use super::{http, opendal, EventSourcePipe, Extractor};
use crate::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "noop")]
    #[default]
    Sleep,
    #[cfg(feature = "source_http")]
    #[serde(alias = "http")]
    Http(http::Config),
    #[cfg(feature = "source_opendal")]
    #[serde(alias = "opendal")]
    Opendal(opendal::Config),
}

impl Config {
    //TODO include some metadata into the extractor like the source name
    pub(crate) fn make_extractor(&self, next: EventSourcePipe) -> Result<Box<dyn Extractor>> {
        let out: Box<dyn Extractor> = match self {
            Config::Sleep => Box::new(SleepExtractor {}),
            #[cfg(feature = "source_http")]
            Config::Http(config) => Box::new(http::HttpExtractor::try_from(config, next)?),
            #[cfg(feature = "source_opendal")]
            Config::Opendal(config) => Box::new(opendal::OpendalExtractor::try_from(config, next)?),
        };
        Ok(out)
    }
}

struct SleepExtractor {}

#[async_trait::async_trait]
impl Extractor for SleepExtractor {
    async fn run(&mut self) -> Result<()> {
        use std::future;

        let future = future::pending();
        let () = future.await;
        unreachable!()
    }
}
