pub(crate) mod extractors;
mod hbs;
#[cfg(feature = "source_http")]
pub(crate) mod http;
#[cfg(feature = "source_opendal")]
mod opendal;
mod send_cdevents;
pub(crate) mod transformers;

use crate::errors::Result;
use crate::pipes::Pipe;
use crate::{Message, Sender};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::task::JoinHandle;

// #[enum_dispatch]
// #[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
// enum SourceEnum {
//     #[cfg(feature = "source_http")]
//     HttpSource,
//     NoopSource,
//     #[cfg(feature = "source_opendal")]
//     OpendalSource,
// }

// TODO support name/reference for extractor / transformer
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    extractor: extractors::Config,
    transformers: Vec<transformers::Config>,
}

pub(crate) fn start(_name: String, config: Config, tx: Sender<Message>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut pipe: EventSourcePipe = Box::new(send_cdevents::Processor::new(tx));
        let mut tconfigs = config.transformers.clone();
        tconfigs.reverse();
        for tconfig in tconfigs {
            pipe = tconfig.into_transformer(pipe)?
        }
        let mut extractor = config.extractor.into_extractor(pipe)?;
        extractor.run().await?;
        Ok(())
    })
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct EventSource {
    metadata: Value,
    header: HashMap<String, String>,
    body: Value,
}

// TODO explore to use enum_dispatch instead of Box(dyn) on EventSourcePipe (a recursive structure)
type EventSourcePipe = Box<dyn Pipe<Input = EventSource> + Send + Sync>;

#[async_trait]
pub trait Extractor: Send + Sync {
    async fn run(&mut self) -> Result<()>;
}
