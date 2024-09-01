pub(crate) mod extractors;
mod hbs;
#[cfg(feature = "source_http")]
pub(crate) mod http;
#[cfg(feature = "source_opendal")]
mod opendal;
mod send_cdevents;
pub(crate) mod transformers;

use std::collections::HashMap;

use crate::errors::Result;
use crate::pipes::Pipe;
use crate::{Message, Sender};
use enum_dispatch::enum_dispatch;
#[cfg(feature = "source_opendal")]
use opendal::OpendalSource;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct EventSource {
    metadata: Value,
    header: HashMap<String, String>,
    body: Value,
}

// TODO explore to use enum_dispatch instead of Box(dyn) on EventSourcePipe (a recursive structure)
type EventSourcePipe = Box<dyn Pipe<Input = EventSource> + Send>;

pub trait Extractor {
    async fn run(&mut self) -> Result<()>;
}

// pub struct SourcePipeline {
//     extractor: Extractor,
//     transformers: Vec<TransformerEnum>,
// }

// trait Extractor {
//     async fn try_next(&mut self) -> Option<Vec<EventPipeline>>;
// }

// impl SourcePipeline {
//     fn new(extractor: Extractor, transformers: Vec<Transformer>) -> Self {
//         Self { extractor, transformers }
//     }
// }

// impl Source for SourcePipeline {
//     // TODO optimize avoid using vector and wave (eg replace by stream pipeline, rx for rust ? (stream/visitor flow are harder to test)
//     // TODO avoid crash on Error
//     // Poll from extractor or provide a function "push" to extractor?
//     async fn run(&mut self, tx: Sender<Message>) -> Result<()> {
//         while let Some(event) = self.extractor.try_next().await? {
//             let mut events = vec![event];
//             for transformer in transformers {
//                 let mut next_events = vec![];
//                 for e in events {
//                     next_events.push_all(transformer.process(event)?);
//                 }
//                 events = next_events;
//             }
//             for e in events {
//                 let cdevent: CDEvent = serde_json::from_slice::<CDEvent>(&e.body)?;
//                 // TODO include headers into message
//                 tx.send(cdevent.into())?;
//             }
//         }
//     }
// }
