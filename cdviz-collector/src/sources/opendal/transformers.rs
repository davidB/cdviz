use std::collections::HashMap;

use crate::errors::Result;
use bytes::Buf;
use enum_dispatch::enum_dispatch;
use handlebars::Handlebars;
use opendal::{Entry, Operator};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
        let hbs = new_renderer(template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for MetadataOnlyViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let metadata = extract_metadata(op, entry);
        let data = json!({
            "metadata" : metadata,
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
        let hbs = new_renderer(template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for JsonViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let bytes = op.read(entry.path()).await?;
        let metadata = extract_metadata(op, entry);
        let content: serde_json::Value = serde_json::from_reader(bytes.reader())?;
        let data = json!({
            "metadata" : metadata,
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
        let hbs = new_renderer(template)?;
        Ok(Self { hbs })
    }
}

impl Transformer for CsvRowViaTemplate {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        use csv::Reader;

        let bytes = op.read(entry.path()).await?;
        let mut rdr = Reader::from_reader(bytes.reader());
        let headers = rdr.headers()?.clone();
        let metadata = extract_metadata(op, entry);
        let mut out = Vec::new();
        for record in rdr.records() {
            let record = record?;
            let content = headers
                .iter()
                .zip(record.iter())
                .collect::<HashMap<&str, &str>>();
            let data = json!({
                "metadata" : metadata.clone(),
                "content": content,
            });
            let output = self.hbs.render("tpl", &data)?;
            out.push(output.into_bytes());
        }
        Ok(out)
    }
}

fn new_renderer(template: &str) -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();
    hbs.set_dev_mode(false);
    hbs.set_strict_mode(true);
    hbs.register_template_string("tpl", template)?;
    Ok(hbs)
}

fn extract_metadata(op: &Operator, entry: &Entry) -> Value {
    json!({
        "name": entry.name(),
        "path": entry.path(),
        "root": op.info().root(),
        "last_modified": entry.metadata().last_modified().map(|dt| dt.to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use assert2::{check, let_assert};
    use chrono::prelude::*;
    use futures::TryStreamExt;
    use opendal::Metakey;

    async fn provide_op_entry(prefix: &str) -> (Operator, Entry) {
        // Create fs backend builder.
        let mut builder = opendal::services::Fs::default();
        let root = Path::new("examples/assets/opendal_fs");
        builder.root(&root.to_string_lossy());
        let op: Operator = Operator::new(builder).unwrap().finish();
        let mut entries = op
            .lister_with(prefix)
            .metakey(Metakey::ContentLength | Metakey::LastModified)
            .await
            .unwrap();
        let_assert!(Ok(Some(entry)) = entries.try_next().await);
        (op, entry)
    }

    #[tokio::test]
    async fn extract_metadata_works() {
        let (op, entry) = provide_op_entry("dir1/file").await;
        // Extract the metadata and check that it's what we expect
        let result = extract_metadata(&op, &entry);
        check!(result["name"] == "file01.txt");
        check!(result["path"] == "dir1/file01.txt");
        let_assert!(Some(abs_root) = result["root"].as_str());
        check!(abs_root.ends_with("examples/assets/opendal_fs"));
        let_assert!(
            Ok(_) = result["last_modified"]
                .as_str()
                .unwrap_or_default()
                .parse::<DateTime<Utc>>()
        );
    }

    #[tokio::test]
    async fn csv_row_via_template_works() {
        let (op, entry) = provide_op_entry("cdevents.").await;
        let sut = CsvRowViaTemplate::new(r#"{{content.env}}"#).unwrap();
        let_assert!(Ok(actual) = sut.transform(&op, &entry).await);
        check!(actual.len() == 3);
        check!(actual[0] == "dev".as_bytes());
    }
}
