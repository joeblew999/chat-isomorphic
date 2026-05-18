use std::path::Path;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use chat_isomorphic_core::{AttachmentInfo, Backend, Event, Identity, Inbound, ThreadId};
use futures::channel::oneshot;
use futures::{future, Stream, StreamExt};
use presage::libsignal_service::configuration::SignalServers;
use presage::libsignal_service::content::{Content, ContentBody, Metadata};
use presage::libsignal_service::protocol::{DeviceId, ServiceId};
use presage::model::contacts::Contact;
use presage::model::groups::Group;
use presage::store::ContentsStore;
use presage::libsignal_service::proto::{
    data_message, sync_message, typing_message, DataMessage, EditMessage, ReceiptMessage,
    SyncMessage, TypingMessage,
};
use presage::libsignal_service::websocket::account::DeviceInfo;
use presage::manager::Registered;
use presage::model::messages::Received;
use presage::Manager;
use presage_store_sqlite::{OnNewIdentity, SqliteStore};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SignalError {
    #[error("store error: {0}")]
    Store(String),
    #[error("presage error: {0}")]
    Presage(String),
    #[error("system clock returned a pre-epoch timestamp")]
    Timestamp,
    #[error("not registered — link this machine first (see mise run signal:link)")]
    NotRegistered,
}

pub struct SignalBackend {
    manager: Manager<SqliteStore, Registered>,
}

impl SignalBackend {
    pub async fn open(store_path: &Path) -> Result<Self, SignalError> {
        let url = format!("sqlite://{}", store_path.display());
        let store = SqliteStore::open(&url, OnNewIdentity::Trust)
            .await
            .map_err(|e| SignalError::Store(e.to_string()))?;
        let manager = Manager::load_registered(store)
            .await
            .map_err(|e| match e {
                presage::Error::NotYetRegisteredError => SignalError::NotRegistered,
                other => SignalError::Presage(other.to_string()),
            })?;
        Ok(Self { manager })
    }

    /// Link this machine as a fresh Signal secondary device. `store_path`
    /// must NOT already hold a registered manager — wipe `.data/signal/`
    /// first if relinking. `on_url` is called once with the provisioning
    /// URL; the future returns only when linking completes or fails.
    pub async fn link<F>(
        store_path: &Path,
        device_name: String,
        on_url: F,
    ) -> Result<Self, SignalError>
    where
        F: FnOnce(Url) + Send + 'static,
    {
        let url = format!("sqlite://{}", store_path.display());
        let store = SqliteStore::open(&url, OnNewIdentity::Trust)
            .await
            .map_err(|e| SignalError::Store(e.to_string()))?;

        let (tx, rx) = oneshot::channel::<Url>();
        let link_fut =
            Manager::link_secondary_device(store, SignalServers::Production, device_name, tx);
        let url_fut = async move {
            if let Ok(provision_url) = rx.await {
                on_url(provision_url);
            }
        };
        let (manager, _) = future::join(link_fut, url_fut).await;
        let manager = manager.map_err(|e| SignalError::Presage(e.to_string()))?;
        Ok(Self { manager })
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceInfo>, SignalError> {
        self.manager
            .devices()
            .await
            .map_err(|e| SignalError::Presage(e.to_string()))
    }

    pub fn device_id(&self) -> DeviceId {
        self.manager.device_id()
    }

    pub async fn list_contacts(&self) -> Result<Vec<Contact>, SignalError> {
        let iter = self
            .manager
            .store()
            .contacts()
            .await
            .map_err(|e| SignalError::Store(e.to_string()))?;
        Ok(iter.filter_map(Result::ok).collect())
    }

    pub async fn list_groups(&self) -> Result<Vec<([u8; 32], Group)>, SignalError> {
        let iter = self
            .manager
            .store()
            .groups()
            .await
            .map_err(|e| SignalError::Store(e.to_string()))?;
        Ok(iter.filter_map(Result::ok).collect())
    }

    pub async fn request_contacts(&mut self) -> Result<(), SignalError> {
        self.manager
            .request_contacts()
            .await
            .map_err(|e| SignalError::Presage(e.to_string()))
    }

    /// Note: presage rejects this call from a secondary device. Only callable
    /// from a primary. Surfaces "Unauthorized" if attempted from us.
    pub async fn unlink_secondary(&self, device_id: i64) -> Result<(), SignalError> {
        self.manager
            .unlink_secondary(device_id as u32)
            .await
            .map_err(|e| SignalError::Presage(e.to_string()))
    }
}

#[async_trait(?Send)]
impl Backend for SignalBackend {
    type Error = SignalError;

    async fn whoami(&self) -> Result<Identity, Self::Error> {
        let reg = self.manager.registration_data();
        let pni = reg.service_ids.pni;
        Ok(Identity {
            aci: reg.service_ids.aci,
            pni: if pni.is_nil() { None } else { Some(pni) },
            e164: Some(reg.phone_number.to_string()),
        })
    }

    async fn send_text(&mut self, to: &ThreadId, body: &str) -> Result<u64, Self::Error> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SignalError::Timestamp)?
            .as_millis() as u64;

        let mut data = DataMessage {
            body: Some(body.to_owned()),
            timestamp: Some(ts),
            ..Default::default()
        };

        match to {
            ThreadId::Contact(uuid) => {
                let service_id = ServiceId::Aci((*uuid).into());
                self.manager
                    .send_message(service_id, data, ts)
                    .await
                    .map_err(|e| SignalError::Presage(e.to_string()))?;
            }
            ThreadId::Group(master_key) => {
                data.group_v2 = Some(presage::libsignal_service::proto::GroupContextV2 {
                    master_key: Some(master_key.to_vec()),
                    revision: None,
                    group_change: None,
                });
                self.manager
                    .send_message_to_group(master_key, data, ts)
                    .await
                    .map_err(|e| SignalError::Presage(e.to_string()))?;
            }
        }
        Ok(ts)
    }

    async fn events(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = Event> + '_>>, Self::Error> {
        let stream = self
            .manager
            .receive_messages()
            .await
            .map_err(|e| SignalError::Presage(e.to_string()))?;

        let mapped = stream.filter_map(|received| async move {
            match received {
                Received::QueueEmpty => Some(Event::BacklogDrained),
                Received::Contacts => Some(Event::Contacts),
                Received::Content(content) => map_content(*content).map(Event::Inbound),
            }
        });

        Ok(Box::pin(mapped))
    }
}

fn service_id_to_uuid(s: &ServiceId) -> Uuid {
    match s {
        ServiceId::Aci(aci) => Uuid::from(*aci),
        ServiceId::Pni(pni) => Uuid::from(*pni),
    }
}

fn map_content(content: Content) -> Option<Inbound> {
    let Content { metadata, body } = content;
    let Metadata { sender, timestamp, .. } = &metadata;
    let from = service_id_to_uuid(sender);
    let ts_ms = *timestamp;
    let thread = thread_from_body(&body, sender);

    match body {
        ContentBody::DataMessage(dm) => map_data_message(dm, from, ts_ms, thread),
        ContentBody::SynchronizeMessage(sm) => map_sync_message(sm, from, ts_ms),
        ContentBody::TypingMessage(tm) => map_typing(tm, from, sender),
        ContentBody::ReceiptMessage(rm) => Some(map_receipt(rm, from)),
        ContentBody::EditMessage(em) => map_edit(em, from, ts_ms, thread),
        ContentBody::CallMessage(_) => Some(Inbound::Other {
            thread,
            from,
            ts_ms,
            kind: "call".into(),
        }),
        _ => Some(Inbound::Other {
            thread,
            from,
            ts_ms,
            kind: "unknown_content_body".into(),
        }),
    }
}

fn thread_from_body(body: &ContentBody, sender: &ServiceId) -> Option<ThreadId> {
    if let ContentBody::DataMessage(dm) = body {
        if let Some(g) = &dm.group_v2 {
            if let Some(mk) = &g.master_key {
                if mk.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(mk);
                    return Some(ThreadId::Group(arr));
                }
            }
        }
    }
    Some(ThreadId::Contact(service_id_to_uuid(sender)))
}

fn map_data_message(
    dm: DataMessage,
    from: Uuid,
    ts_ms: u64,
    thread: Option<ThreadId>,
) -> Option<Inbound> {
    let thread = thread?;

    if let Some(reaction) = dm.reaction {
        return Some(Inbound::Reaction {
            thread,
            from,
            ts_ms,
            emoji: reaction.emoji.unwrap_or_default(),
            target_ts_ms: reaction.target_sent_timestamp.unwrap_or_default(),
            target_author: reaction
                .target_author_aci
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            remove: reaction.remove.unwrap_or(false),
        });
    }

    if let Some(delete) = dm.delete {
        return Some(Inbound::DeleteForEveryone {
            thread,
            from,
            ts_ms,
            target_ts_ms: delete.target_sent_timestamp.unwrap_or_default(),
        });
    }

    if let Some(sticker) = dm.sticker {
        return Some(Inbound::Sticker {
            thread,
            from,
            ts_ms,
            emoji: sticker.emoji,
        });
    }

    if !dm.attachments.is_empty() {
        let attachments = dm
            .attachments
            .iter()
            .map(|a| AttachmentInfo {
                content_type: a.content_type.clone(),
                size: a.size,
                filename: a.file_name.clone(),
            })
            .collect();
        return Some(Inbound::Attachment {
            thread,
            from,
            ts_ms,
            attachments,
            caption: dm.body.clone(),
        });
    }

    if let Some(body) = dm.body {
        return Some(Inbound::Text {
            thread,
            from,
            ts_ms,
            body,
        });
    }

    Some(Inbound::Other {
        thread: Some(thread),
        from,
        ts_ms,
        kind: "empty_data_message".into(),
    })
}

fn map_sync_message(sm: SyncMessage, from: Uuid, ts_ms: u64) -> Option<Inbound> {
    use sync_message::Sent;

    if let Some(Sent { message: Some(dm), destination_service_id, .. }) = sm.sent {
        let thread = destination_service_id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(ThreadId::Contact);
        return map_data_message(dm, from, ts_ms, thread);
    }
    Some(Inbound::Other {
        thread: None,
        from,
        ts_ms,
        kind: "sync_message_other".into(),
    })
}

fn map_typing(tm: TypingMessage, from: Uuid, sender: &ServiceId) -> Option<Inbound> {
    let started = matches!(
        tm.action.and_then(|a| typing_message::Action::try_from(a).ok()),
        Some(typing_message::Action::Started)
    );
    let ts_ms = tm.timestamp.unwrap_or_default();
    let thread = match tm.group_id {
        Some(_) => None, // we don't have the master key here, only the group_id hash
        None => Some(ThreadId::Contact(service_id_to_uuid(sender))),
    };
    thread.map(|thread| Inbound::Typing {
        thread,
        from,
        ts_ms,
        started,
    })
}

fn map_receipt(rm: ReceiptMessage, from: Uuid) -> Inbound {
    let kind = rm
        .r#type
        .and_then(|t| presage::libsignal_service::proto::receipt_message::Type::try_from(t).ok())
        .map(|t| format!("{t:?}").to_lowercase())
        .unwrap_or_else(|| "unknown".into());
    Inbound::Receipt {
        from,
        target_ts_ms: rm.timestamp,
        kind,
    }
}

fn map_edit(em: EditMessage, from: Uuid, ts_ms: u64, thread: Option<ThreadId>) -> Option<Inbound> {
    let thread = thread?;
    Some(Inbound::Edit {
        thread,
        from,
        ts_ms,
        target_ts_ms: em.target_sent_timestamp.unwrap_or_default(),
        new_body: em.data_message.and_then(|dm| dm.body),
    })
}

// Silence unused-import warnings for items reached only via fully-qualified
// path in match arms above. Keeps the explicit imports at the top for clarity.
#[allow(dead_code)]
const _: fn() = || {
    let _ = data_message::Reaction::default();
};
