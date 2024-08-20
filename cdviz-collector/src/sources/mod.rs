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

#[derive(Debug, Deserialize, Serialize)]
pub struct EventPipeline {
    metadata: Json::Value,
    header: HashMap<String, String>,
    body: Json::Value,
}

pub struct SourcePipeline {
    extractor: Extractor,
    transformers: Vec<TransformerEnum>,
}

trait Extractor {
    async fn try_next(&mut self) -> Option<Vec<EventPipeline>>;
}

impl SourcePipeline {
    fn new(extractor: Extractor, transformers: Vec<Transformer>) -> Self {
        Self {
            extractor,
            transformers,
        }
    }
}

impl Source for SourcePipeline {
    // TODO optimize avoid using vector and wave (eg replace by stream pipeline, rx for rust ? (stream/visitor flow are harder to test)
    // TODO avoid crash on Error
    // Poll from extractor or provide a function "push" to extractor?
    async fn run(&mut self, tx: Sender<Message>) -> Result<()> {
        while let Some(event) = self.extractor.try_next().await? {
            let mut events = vec![event];
            for transformer in transformers {
                let mut next_events = vec![];
                for e in events {
                    next_events.push_all(transformer.process(event)?);
                }
                events = next_events;
            }
            for e in events {
                let cdevent: CDEvent = serde_json::from_slice::<CDEvent>(&e.body)?;
                // TODO include headers into message
                tx.send(cdevent.into())?;
            }
        }
    }
}
