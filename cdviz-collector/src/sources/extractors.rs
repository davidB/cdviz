use super::{http, opendal, EventSourcePipe, Extractor};
use crate::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub(crate) enum Config {
    #[cfg(feature = "source_http")]
    #[serde(alias = "http")]
    Http(http::Config),
    #[cfg(feature = "source_opendal")]
    #[serde(alias = "opendal")]
    Opendal(opendal::Config),
}

impl Config {
    pub(crate) fn into_extractor(&self, next: EventSourcePipe) -> Result<Box<dyn Extractor>> {
        let out: Box<dyn Extractor> = match self {
            #[cfg(feature = "source_http")]
            Config::Http(config) => Box::new(http::HttpExtractor::try_from(config, next)?),
            #[cfg(feature = "source_opendal")]
            Config::Opendal(config) => Box::new(opendal::OpendalExtractor::try_from(config, next)?),
        };
        Ok(out)
    }
}
