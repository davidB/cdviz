//TODO add transformer: identity, csv+template -> bunch, jsonl

use std::collections::HashMap;

use crate::errors::Result;
use bytes::Buf;
use enum_dispatch::enum_dispatch;
use handlebars::Handlebars;
use opendal::{Entry, Operator};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(tag = "type")]
pub(crate) enum Config {
    #[serde(alias = "json_content_as_is")]
    #[default]
    JsonContentAsIs,
    #[serde(alias = "metadata_only_via_template")]
    MetadataOnlyViaTemplate { template: String },
    #[serde(alias = "json_via_template")]
    JsonViaTemplate { template: String },
    #[serde(alias = "csv_row_via_template")]
    CsvRowViaTemplate { template: String },
}

impl TryFrom<Config> for TransformerEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            Config::JsonContentAsIs => JsonContentAsIs {}.into(),
            Config::MetadataOnlyViaTemplate { template } => {
                MetadataOnlyViaTemplate::new(&template)?.into()
            }
            Config::JsonViaTemplate { template } => JsonViaTemplate::new(&template)?.into(),
            Config::CsvRowViaTemplate { template } => CsvRowViaTemplate::new(&template)?.into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub(crate) enum TransformerEnum {
    JsonContentAsIs,
    MetadataOnlyViaTemplate,
    JsonViaTemplate,
    CsvRowViaTemplate,
}

#[enum_dispatch(TransformerEnum)]
pub(crate) trait Transformer {
    //TODO return a common Iterator type (better than Vec)
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>>;
}

#[derive(Debug)]
pub(crate) struct JsonContentAsIs;

impl Transformer for JsonContentAsIs {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let input = op.read(entry.path()).await?;
        Ok(vec![input.to_vec()])
    }
}

#[derive(Debug)]
pub(crate) struct MetadataOnlyViaTemplate {
    hbs: Handlebars<'static>,
}

impl MetadataOnlyViaTemplate {
    fn new(template: &str) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.register_template_string("tpl", template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for MetadataOnlyViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let data = json!({
            "metadata" : {
                "name": entry.name(),
                "path": entry.path(),
                "root": op.info().root(),
                "last_modified": entry.metadata().last_modified(),
            }
        });
        let output = self.hbs.render("tpl", &data)?;
        Ok(vec![output.into_bytes()])
    }
}

#[derive(Debug)]
pub(crate) struct JsonViaTemplate {
    hbs: Handlebars<'static>,
}

impl JsonViaTemplate {
    fn new(template: &str) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.register_template_string("tpl", template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for JsonViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let bytes = op.read(entry.path()).await?;
        let content: serde_json::Value = serde_json::from_reader(bytes.reader())?;
        let data = json!({
            "metadata" : {
                "name": entry.name(),
                "path": entry.path(),
                "root": op.info().root(),
                "last_modified": entry.metadata().last_modified(),
            },
            "content": content,
        });
        let output = self.hbs.render("tpl", &data)?;
        Ok(vec![output.into_bytes()])
    }
}

#[derive(Debug)]
pub(crate) struct CsvRowViaTemplate {
    hbs: Handlebars<'static>,
}

impl CsvRowViaTemplate {
    fn new(template: &str) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.register_template_string("tpl", template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for CsvRowViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        use csv::Reader;

        let bytes = op.read(entry.path()).await?;
        let mut rdr = Reader::from_reader(bytes.reader());
        let headers = rdr.headers()?.clone();

        let mut out = Vec::new();
        for record in rdr.records() {
            let record = record?;
            let content = headers
                .iter()
                .zip(record.iter())
                .collect::<HashMap<&str, &str>>();
            let data = json!({
                "metadata" : {
                    "name": entry.name(),
                    "path": entry.path(),
                    "root": op.info().root(),
                    "last_modified": entry.metadata().last_modified(),
                },
                "content": content,
            });
            let output = self.hbs.render("tpl", &data)?;
            out.push(output.into_bytes());
        }
        Ok(out)
    }
}
