pub(crate) mod extractors;
#[cfg(feature = "source_http")]
pub(crate) mod http;
#[cfg(feature = "source_opendal")]
pub(crate) mod opendal;
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
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct Config {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    extractor: extractors::Config,
    #[serde(default)]
    transformer_refs: Vec<String>,
    #[serde(default)]
    transformers: Vec<transformers::Config>,
}

impl Config {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn resolve_transformers(
        &mut self,
        configs: &HashMap<String, transformers::Config>,
    ) -> Result<()> {
        let mut tconfigs = transformers::resolve_transformer_refs(&self.transformer_refs, configs)?;
        self.transformers.append(&mut tconfigs);
        Ok(())
    }
}

pub(crate) fn start(_name: &str, config: Config, tx: Sender<Message>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut pipe: EventSourcePipe = Box::new(send_cdevents::Processor::new(tx));
        let mut tconfigs = config.transformers.clone();
        tconfigs.reverse();
        for tconfig in tconfigs {
            pipe = tconfig.make_transformer(pipe)?;
        }
        let mut extractor = config.extractor.make_extractor(pipe)?;
        extractor.run().await?;
        Ok(())
    })
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct EventSource {
    pub metadata: Value,
    pub header: HashMap<String, String>,
    pub body: Value,
}

// TODO explore to use enum_dispatch instead of Box(dyn) on EventSourcePipe (a recursive structure)
pub type EventSourcePipe = Box<dyn Pipe<Input = EventSource> + Send + Sync>;

#[async_trait]
pub trait Extractor: Send + Sync {
    async fn run(&mut self) -> Result<()>;
}
