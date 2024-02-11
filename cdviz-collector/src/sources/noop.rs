use crate::errors::Result;
use crate::{Message, Sender};
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

use super::Source;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {}

impl TryFrom<Config> for NoopSource {
    type Error = crate::errors::Error;

    fn try_from(_value: Config) -> Result<Self> {
        Ok(Self {})
    }
}

pub(crate) struct NoopSource {}

impl Source for NoopSource {
    async fn run(&self, _tx: Sender<Message>) -> Result<()> {
        loop {
            sleep(Duration::MAX).await;
        }
    }
}
