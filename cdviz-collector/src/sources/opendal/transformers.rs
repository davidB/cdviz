//TODO add transformer: identity, csv+template -> bunch, jsonl

use crate::errors::Result;
use enum_dispatch::enum_dispatch;
use opendal::{Entry, Operator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "identity")]
    #[default]
    Identity,
}

impl TryFrom<Config> for TransformerEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            Config::Identity => Identity {}.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum TransformerEnum {
    Identity,
}

#[enum_dispatch(TransformerEnum)]
pub(crate) trait Transformer {
    async fn transform(
        &self,
        op: &Operator,
        entry: &Entry,
    ) -> Result<impl Iterator<Item = Vec<u8>>>;
}

#[derive(Debug)]
pub(crate) struct Identity;

impl Transformer for Identity {
    async fn transform(
        &self,
        op: &Operator,
        entry: &Entry,
    ) -> Result<impl Iterator<Item = Vec<u8>>> {
        let input = op.read(entry.path()).await?;
        Ok(Some(input.to_vec()).into_iter())
    }
}
