use std::collections::HashMap;

use super::{hbs, EventSourcePipe};
use crate::{
    errors::{Error, Result},
    pipes::{discard_all, log, passthrough},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "passthrough")]
    #[default]
    Passthrough,
    #[serde(alias = "log")]
    Log(log::Config),
    #[serde(alias = "discard_all")]
    DiscardAll,
    #[serde(alias = "hbs")]
    Hbs { template: String },
    // #[serde(alias = "vrl")]
    // Vrl(String),
}

impl Config {
    pub(crate) fn make_transformer(&self, next: EventSourcePipe) -> Result<EventSourcePipe> {
        let out: EventSourcePipe = match &self {
            Config::Passthrough => Box::new(passthrough::Processor::new(next)),
            Config::Log(config) => Box::new(log::Processor::try_from(config, next)?),
            Config::DiscardAll => Box::new(discard_all::Processor::new()),
            Config::Hbs { template } => Box::new(hbs::Processor::new(template, next)?),
        };
        Ok(out)
    }
}

pub fn resolve_transformer_refs(
    transformer_refs: &[String],
    configs: &HashMap<String, Config>,
) -> Result<Vec<Config>> {
    let transformers = transformer_refs
        .iter()
        .map(|name| {
            configs
                .get(name)
                .cloned()
                .ok_or_else(|| Error::ConfigTransformerNotFound(name.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(transformers)
}

// pub struct Identity {
//     next: EventSourcePipe,
// }

// impl Identity {
//     fn new(next: EventSourcePipe) -> Self {
//         Self { next }
//     }
// }

// impl Pipe for Identity {
//     type Input = EventSource;
//     fn send(&mut self, input: Self::Input) -> Result<()> {
//         self.next.send(input)
//     }
// }
