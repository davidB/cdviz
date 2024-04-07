#[cfg(feature = "source_http")]
pub(crate) mod http;
pub(crate) mod noop;
#[cfg(feature = "source_opendal")]
pub(crate) mod opendal;

use crate::errors::Result;
use crate::{Message, Sender};
use enum_dispatch::enum_dispatch;
#[cfg(feature = "source_http")]
use http::HttpSource;
use noop::NoopSource;
#[cfg(feature = "source_opendal")]
use opendal::OpendalSource;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[cfg(feature = "source_http")]
    #[serde(alias = "http")]
    Http(http::Config),
    #[serde(alias = "noop")]
    Noop(noop::Config),
    #[cfg(feature = "source_opendal")]
    #[serde(alias = "opendal")]
    Opendal(opendal::Config),
}

impl TryFrom<Config> for SourceEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            #[cfg(feature = "source_http")]
            Config::Http(config) => HttpSource::try_from(config)?.into(),
            Config::Noop(config) => NoopSource::try_from(config)?.into(),
            #[cfg(feature = "source_opendal")]
            Config::Opendal(config) => OpendalSource::try_from(config)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
enum SourceEnum {
    #[cfg(feature = "source_http")]
    HttpSource,
    NoopSource,
    #[cfg(feature = "source_opendal")]
    OpendalSource,
}

#[enum_dispatch(SourceEnum)]
trait Source {
    async fn run(&mut self, tx: Sender<Message>) -> Result<()>;
}

pub(crate) fn start(_name: String, config: Config, tx: Sender<Message>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut source = SourceEnum::try_from(config)?;
        source.run(tx).await?;
        Ok(())
    })
}
