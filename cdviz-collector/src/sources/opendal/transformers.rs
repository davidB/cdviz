use std::collections::HashMap;

use crate::errors::Result;
use bytes::Buf;
use enum_dispatch::enum_dispatch;
use opendal::{Entry, Operator};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::texecutors::{TExecutor, TExecutorConfig, TExecutorEnum};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "extractor")]
pub(crate) enum Config {
    #[serde(alias = "json")]
    Json { transform: Option<TExecutorConfig> },
    #[serde(alias = "metadata")]
    Metadata { transform: TExecutorConfig },
    #[serde(alias = "csv_row")]
    CsvRow { transform: TExecutorConfig },
}

impl TryFrom<Config> for TransformerEnum {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let out = match value {
            Config::Json { transform } => match transform {
                None => JsonContentAsIs {}.into(),
                Some(te) => JsonExtractor::new(te.try_into()?).into(),
            },
            Config::Metadata { transform } => MetadataExtractor::new(transform.try_into()?).into(),
            Config::CsvRow { transform } => CsvRowExtractor::new(transform.try_into()?).into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub(crate) enum TransformerEnum {
    JsonContentAsIs,
    JsonExtractor,
    MetadataExtractor,
    CsvRowExtractor,
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
pub(crate) struct MetadataExtractor {
    texecutor: TExecutorEnum,
}

impl MetadataExtractor {
    fn new(texecutor: TExecutorEnum) -> Self {
        Self { texecutor }
    }
}

impl Transformer for MetadataExtractor {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let metadata = extract_metadata(op, entry);
        let data = json!({
            "metadata" : metadata,
        });
        let output = self.texecutor.execute(data)?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
pub(crate) struct JsonExtractor {
    texecutor: TExecutorEnum,
}

impl JsonExtractor {
    fn new(texecutor: TExecutorEnum) -> Self {
        Self { texecutor }
    }
}

impl Transformer for JsonExtractor {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        let bytes = op.read(entry.path()).await?;
        let metadata = extract_metadata(op, entry);
        let content: serde_json::Value = serde_json::from_reader(bytes.reader())?;
        let data = json!({
            "metadata" : metadata,
            "content": content,
        });
        let output = self.texecutor.execute(data)?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
pub(crate) struct CsvRowExtractor {
    texecutor: TExecutorEnum,
}

impl CsvRowExtractor {
    fn new(texecutor: TExecutorEnum) -> Self {
        Self { texecutor }
    }
}

impl Transformer for CsvRowExtractor {
    async fn transform(&self, op: &Operator, entry: &Entry) -> Result<Vec<Vec<u8>>> {
        use csv::Reader;

        let bytes = op.read(entry.path()).await?;
        let mut rdr = Reader::from_reader(bytes.reader());
        let headers = rdr.headers()?.clone();
        let metadata = extract_metadata(op, entry);
        let mut out = Vec::new();
        for record in rdr.records() {
            let record = record?;
            let content = headers.iter().zip(record.iter()).collect::<HashMap<&str, &str>>();
            let data = json!({
                "metadata" : metadata.clone(),
                "content": content,
            });
            let output = self.texecutor.execute(data)?;
            out.push(output);
        }
        Ok(out)
    }
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

    use crate::sources::opendal::texecutors::Hbs;

    use super::*;
    use assert2::{check, let_assert};
    use chrono::prelude::*;
    use futures::TryStreamExt;
    use opendal::Metakey;

    async fn provide_op_entry(prefix: &str) -> (Operator, Entry) {
        // Create fs backend builder.
        let root = Path::new("examples/assets/opendal_fs");
        let builder = opendal::services::Fs::default().root(&root.to_string_lossy());
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
            Ok(_) = result["last_modified"].as_str().unwrap_or_default().parse::<DateTime<Utc>>()
        );
    }

    #[tokio::test]
    async fn csv_row_via_template_works() {
        let (op, entry) = provide_op_entry("cdevents.").await;
        let sut =
            CsvRowExtractor::new(TExecutorEnum::from(Hbs::new(r#"{{content.env}}"#).unwrap()));
        let_assert!(Ok(actual) = sut.transform(&op, &entry).await);
        check!(actual.len() == 3);
        check!(actual[0] == "dev".as_bytes());
    }
}
