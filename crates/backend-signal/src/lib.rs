use std::pin::Pin;

use async_trait::async_trait;
use chat_isomorphic_core::{Backend, Event, Identity, ThreadId};
use futures::{Stream, stream};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignalError {
    #[error("not yet implemented")]
    NotImplemented,
}

pub struct SignalBackend {
    _store_path: std::path::PathBuf,
}

impl SignalBackend {
    pub fn new(store_path: impl Into<std::path::PathBuf>) -> Self {
        Self { _store_path: store_path.into() }
    }
}

#[async_trait(?Send)]
impl Backend for SignalBackend {
    type Error = SignalError;

    async fn whoami(&self) -> Result<Identity, Self::Error> {
        Err(SignalError::NotImplemented)
    }

    async fn send_text(&self, _to: &ThreadId, _body: &str) -> Result<(), Self::Error> {
        Err(SignalError::NotImplemented)
    }

    fn events(&self) -> Pin<Box<dyn Stream<Item = Event> + '_>> {
        Box::pin(stream::empty())
    }
}
