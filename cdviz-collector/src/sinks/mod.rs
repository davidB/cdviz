#[cfg(feature = "sink_db")]
pub(crate) mod db;
pub(crate) mod debug;
pub(crate) mod http;

use crate::errors::Result;
use crate::{Message, Receiver};
use debug::DebugSink;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use db::DbSink;
use http::HttpSink;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "postgresql")]
    Db(db::Config),
    #[serde(alias = "debug")]
    Debug(debug::Config),
    #[serde(alias = "http")]
    Http(http::Config),
}

impl TryFrom<Config> for SinkEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            Config::Db(config) => DbSink::try_from(config)?.into(),
            Config::Debug(config) => DebugSink::try_from(config)?.into(),
            Config::Http(config) => HttpSink::try_from(config)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
enum SinkEnum {
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
            if let Err(err) = sink.send(&msg).await {
                tracing::warn!(name, ?err, "fail during sending of event");
            }
        }
        Ok(())
    })
}
