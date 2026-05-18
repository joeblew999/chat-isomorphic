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
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadId {
    Contact(Uuid),
    Group([u8; 32]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub thread: ThreadId,
    pub from: Uuid,
    pub timestamp_ms: u64,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    BacklogDrained,
    Message(InboundMessage),
    Disconnected,
}

#[async_trait(?Send)]
pub trait Backend {
    type Error: std::error::Error + 'static;

    async fn whoami(&self) -> Result<Identity, Self::Error>;

    async fn send_text(&self, to: &ThreadId, body: &str) -> Result<(), Self::Error>;

    fn events(&self) -> Pin<Box<dyn Stream<Item = Event> + '_>>;
}
