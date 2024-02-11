pub(crate) mod http;
pub(crate) mod opendal;

use crate::errors::Result;
use crate::{Message, Sender};
use enum_dispatch::enum_dispatch;
use http::HttpSource;
use opendal::OpendalSource;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "http")]
    Http(http::Config),
    #[serde(alias = "opendal")]
    Opendal(opendal::Config),
}

impl TryFrom<Config> for SourceEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            Config::Opendal(config) => OpendalSource::try_from(config)?.into(),
            Config::Http(config) => HttpSource::try_from(config)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
enum SourceEnum {
    HttpSource,
    OpendalSource,
}

#[enum_dispatch(SourceEnum)]
trait Source {
    async fn run(&self, tx: Sender<Message>) -> Result<()>;
}

pub(crate) fn start(_name: String, config: Config, tx: Sender<Message>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let source = SourceEnum::try_from(config)?;
        source.run(tx).await?;
        Ok(())
    })
}
