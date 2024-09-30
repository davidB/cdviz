#[cfg(feature = "sink_db")]
pub(crate) mod db;
pub(crate) mod debug;
pub(crate) mod http;

use crate::errors::Result;
use crate::{Message, Receiver};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[cfg(feature = "sink_db")]
use db::DbSink;
use debug::DebugSink;
use http::HttpSink;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[cfg(feature = "sink_db")]
    #[serde(alias = "db")]
    Db(db::Config),
    #[serde(alias = "debug")]
    Debug(debug::Config),
    #[serde(alias = "http")]
    Http(http::Config),
}

impl Default for Config {
    fn default() -> Self {
        Self::Debug(debug::Config { enabled: true })
    }
}

impl Config {
    pub(crate) fn is_enabled(&self) -> bool {
        match self {
            Self::Db(db::Config { enabled, .. }) => *enabled,
            Self::Debug(debug::Config { enabled, .. }) => *enabled,
            Self::Http(http::Config { enabled, .. }) => *enabled,
        }
    }
}

impl TryFrom<Config> for SinkEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            #[cfg(feature = "sink_db")]
            Config::Db(config) => DbSink::try_from(config)?.into(),
            Config::Debug(config) => DebugSink::try_from(config)?.into(),
            Config::Http(config) => HttpSink::try_from(config)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[allow(clippy::enum_variant_names)]
enum SinkEnum {
    #[cfg(feature = "sink_db")]
    DbSink,
    DebugSink,
    HttpSink,
}

#[enum_dispatch(SinkEnum)]
trait Sink {
    async fn send(&self, msg: &Message) -> Result<()>;
}

pub(crate) fn start(name: String, config: Config, rx: Receiver<Message>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let sink = SinkEnum::try_from(config)?;
        let mut rx = rx;
        while let Ok(msg) = rx.recv().await {
            tracing::debug!(name, event_id = ?msg.cdevent.id(), "sending");
            if let Err(err) = sink.send(&msg).await {
                tracing::warn!(name, ?err, "fail during sending of event");
            }
        }
        Ok(())
    })
}
