use super::{hbs, EventSourcePipe};
use crate::{errors::Result, pipes::discard_all, pipes::log, pipes::passthrough};
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
