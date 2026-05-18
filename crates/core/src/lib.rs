use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub aci: Uuid,
    pub pni: Option<Uuid>,
    pub e164: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadId {
    Contact(Uuid),
    Group([u8; 32]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub content_type: Option<String>,
    pub size: Option<u32>,
    pub filename: Option<String>,
}

/// Every meaningful variant of an inbound DataMessage / TypingMessage /
/// ReceiptMessage, surfaced explicitly. The whole point of chat-isomorphic
/// is that we never silently swallow a variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Inbound {
    Text {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        body: String,
    },
    Reaction {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        emoji: String,
        target_ts_ms: u64,
        target_author: Option<Uuid>,
        remove: bool,
    },
    Attachment {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        attachments: Vec<AttachmentInfo>,
        caption: Option<String>,
    },
    Sticker {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        emoji: Option<String>,
    },
    Edit {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        target_ts_ms: u64,
        new_body: Option<String>,
    },
    DeleteForEveryone {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        target_ts_ms: u64,
    },
    Typing {
        thread: ThreadId,
        from: Uuid,
        ts_ms: u64,
        started: bool,
    },
    Receipt {
        from: Uuid,
        target_ts_ms: Vec<u64>,
        kind: String,
    },
    /// Anything we recognized but haven't given a dedicated variant yet.
    /// Carries enough breadcrumbs to triage what we're missing.
    Other {
        thread: Option<ThreadId>,
        from: Uuid,
        ts_ms: u64,
        kind: String,
    },
}

#[derive(Debug, Clone)]
pub enum Event {
    BacklogDrained,
    Contacts,
    Inbound(Inbound),
}

#[async_trait(?Send)]
pub trait Backend {
    type Error: std::error::Error + 'static;

    async fn whoami(&self) -> Result<Identity, Self::Error>;

    async fn send_text(&mut self, to: &ThreadId, body: &str) -> Result<u64, Self::Error>;

    async fn events(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = Event> + '_>>, Self::Error>;
}
