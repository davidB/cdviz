use crate::errors::Result;
use enum_dispatch::enum_dispatch;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "format", content = "content")]
pub(crate) enum TExecutorConfig {
    #[serde(alias = "hbs")]
    Hbs(String),
    // #[serde(alias = "vrl")]
    // Vrl(String),
}

impl TryFrom<TExecutorConfig> for TExecutorEnum {
    type Error = crate::errors::Error;

    fn try_from(value: TExecutorConfig) -> Result<Self> {
        let out = match value {
            TExecutorConfig::Hbs(template) => Hbs::new(&template)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub(crate) enum TExecutorEnum {
    Hbs,
}

#[enum_dispatch(TExecutorEnum)]
pub(crate) trait TExecutor {
    //TODO return a common Iterator or stream type (better than Vec)
    fn execute(&self, json: Value) -> Result<Vec<u8>>;
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

impl TExecutor for Hbs {
    fn execute(&self, data: Value) -> Result<Vec<u8>> {
        Ok(self.hbs.render("tpl", &data)?.into_bytes())
    }
}
