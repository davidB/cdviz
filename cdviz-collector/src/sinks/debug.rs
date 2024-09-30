use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::Message;

use super::Sink;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct Config {
    /// Is the sink is enabled?
    pub(crate) enabled: bool,
}

impl TryFrom<Config> for DebugSink {
    type Error = crate::errors::Error;

    fn try_from(_value: Config) -> Result<Self> {
        Ok(DebugSink {})
    }
}

pub(crate) struct DebugSink {}

impl Sink for DebugSink {
    async fn send(&self, msg: &Message) -> Result<()> {
        tracing::info!(cdevent=?msg.cdevent, "mock sending");
        Ok(())
    }
}
