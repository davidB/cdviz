use std::collections::HashMap;

use crate::{
    errors::Result,
    sources::{EventSource, EventSourcePipe},
};
use bytes::Buf;
use enum_dispatch::enum_dispatch;
use opendal::{Entry, Operator};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum Config {
    #[serde(alias = "json")]
    Json,
    #[serde(alias = "metadata")]
    Metadata,
    #[serde(alias = "csv_row")]
    CsvRow,
}

impl Config {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn make_parser(&self, next: EventSourcePipe) -> Result<ParserEnum> {
        let out = match self {
            Config::Json => JsonParser::new(next).into(),
            Config::Metadata => MetadataParser::new(next).into(),
            Config::CsvRow => CsvRowParser::new(next).into(),
        };
        Ok(out)
    }
}

#[enum_dispatch]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ParserEnum {
    JsonParser,
    MetadataParser,
    CsvRowParser,
}

#[enum_dispatch(ParserEnum)]
pub(crate) trait Parser {
    async fn parse(&mut self, op: &Operator, entry: &Entry) -> Result<()>;
}

pub(crate) struct MetadataParser {
    next: EventSourcePipe,
}

impl MetadataParser {
    fn new(next: EventSourcePipe) -> Self {
        Self { next }
    }
}

impl Parser for MetadataParser {
    async fn parse(&mut self, op: &Operator, entry: &Entry) -> Result<()> {
        let metadata = extract_metadata(op, entry);
        let event = EventSource { metadata, ..Default::default() };
        self.next.send(event)
    }
}

pub(crate) struct JsonParser {
    next: EventSourcePipe,
}

impl JsonParser {
    fn new(next: EventSourcePipe) -> Self {
        Self { next }
    }
}

impl Parser for JsonParser {
    async fn parse(&mut self, op: &Operator, entry: &Entry) -> Result<()> {
        let bytes = op.read(entry.path()).await?;
        let metadata = extract_metadata(op, entry);
        let body: serde_json::Value = serde_json::from_reader(bytes.reader())?;
        let event = EventSource { metadata, body, ..Default::default() };
        self.next.send(event)
    }
}

pub(crate) struct CsvRowParser {
    next: EventSourcePipe,
}

impl CsvRowParser {
    fn new(next: EventSourcePipe) -> Self {
        Self { next }
    }
}

impl Parser for CsvRowParser {
    async fn parse(&mut self, op: &Operator, entry: &Entry) -> Result<()> {
        use csv::Reader;

        let bytes = op.read(entry.path()).await?;
        let mut rdr = Reader::from_reader(bytes.reader());
        let headers = rdr.headers()?.clone();
        let metadata = extract_metadata(op, entry);
        for record in rdr.records() {
            let record = record?;
            let body = json!(headers.iter().zip(record.iter()).collect::<HashMap<&str, &str>>());
            let event = EventSource { metadata: metadata.clone(), body, ..Default::default() };
            self.next.send(event)?;
        }
        Ok(())
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
    use super::*;
    use assert2::{check, let_assert};
    use chrono::prelude::*;
    use futures::TryStreamExt;
    use opendal::Metakey;
    use std::path::Path;

    async fn provide_op_entry(prefix: &str) -> (Operator, Entry) {
        // Create fs backend builder.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/assets/inputs");
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
        check!(abs_root.ends_with("examples/assets/inputs"));
        let_assert!(
            Ok(_) = result["last_modified"].as_str().unwrap_or_default().parse::<DateTime<Utc>>()
        );
    }

    // TODO
    // #[tokio::test]
    // async fn csv_row_via_template_works() {
    //     let (op, entry) = provide_op_entry("cdevents.").await;
    //     let dest = collect_to_vec::Processor::new();
    //     let collector = dest.collector();
    //     let sut = CsvRowParser::new(Box::new(collector));
    //     assert!(Ok(()) == sut.parse(&op, &entry).await);
    //     check!(collector.len() == 3);
    //     // TODO check!(collector[0]. == "dev".as_bytes());
    // }
}
