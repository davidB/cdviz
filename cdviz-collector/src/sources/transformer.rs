use crate::errors::Result;
use enum_dispatch::enum_dispatch;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "format", content = "content")]
pub(crate) enum TransformerConfig {
    #[serde(alias = "hbs")]
    Hbs(String),
    // #[serde(alias = "vrl")]
    // Vrl(String),
}

impl TryFrom<TransformerConfig> for TransformerEnum {
    type Error = crate::errors::Error;

    fn try_from(value: TransformerConfig) -> Result<Self> {
        let out = match value {
            TransformerConfig::Hbs(template) => Hbs::new(&template)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub(crate) enum TransformerEnum {
    Hbs,
}


#[enum_dispatch(TransformerEnum)]
pub(crate) trait Transformer {
    //TODO return a common Iterator or stream type (better than Vec)
    // transform the event
    // return a list of events, empty if event to remove/filter out, severals event for 1-to-many,
    // single for 1-to-1 (and same as input of no transformation to apply)
    fn process(&self, e: EventPipeline) -> Result<Vec<EventPipeline>>;
}

#[derive(Debug)]
pub(crate) struct Hbs {
    hbs: Handlebars<'static>,
}

impl Hbs {
    pub(crate) fn new(template: &str) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_dev_mode(false);
        hbs.set_strict_mode(true);
        hbs.register_template_string("tpl", template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for Hbs {
    fn process(&self, e: EventPipeline) -> Result<Vec<EventPipeline>> {
        let bytes = self.hbs.render("tpl", &e)?.into_bytes();
        serde_json::from_slice::<Vec<EventPipeline>>(&bytes)?;
        Ok()
    }
}
