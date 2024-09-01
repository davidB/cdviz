use super::{EventSource, EventSourcePipe};
use crate::errors::Result;
use crate::pipes::Pipe;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    template: String,
}

pub(crate) struct Processor {
    next: EventSourcePipe,
    hbs: Handlebars<'static>,
}

impl Processor {
    pub(crate) fn new(template: &str, next: EventSourcePipe) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_dev_mode(false);
        hbs.set_strict_mode(true);
        hbs.register_template_string("tpl", template)?;
        Ok(Self { next, hbs })
    }

    pub(crate) fn try_from(config: Config, next: EventSourcePipe) -> Result<Self> {
        Self::new(&config.template, next)
    }
}

impl Pipe for Processor {
    type Input = EventSource;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        let res = self.hbs.render("tpl", &input)?;
        let output: EventSource = serde_json::from_str(&res)?;
        self.next.send(output)
    }
}
