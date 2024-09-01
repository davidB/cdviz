use super::{hbs, EventSourcePipe};
use crate::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub(crate) enum Config {
    #[serde(alias = "hbs")]
    Hbs { template: String },
    // #[serde(alias = "vrl")]
    // Vrl(String),
}

impl Config {
    fn into_transformer(&self, next: EventSourcePipe) -> Result<EventSourcePipe> {
        let out = match &self {
            Config::Hbs { template } => Box::new(hbs::Processor::new(&template, next)?),
        };
        Ok(out)
    }
}
