//TODO add persistance for state (time window to not reprocess same file after restart)

mod filter;
pub(crate) mod parsers;

use self::filter::{FilePatternMatcher, Filter};
use self::parsers::{Parser, ParserEnum};
use super::{EventSourcePipe, Extractor};
use crate::errors::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use opendal::Metakey;
use opendal::Operator;
use opendal::Scheme;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    #[serde(with = "humantime_serde")]
    pub(crate) polling_interval: Duration,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) kind: Scheme,
    pub(crate) parameters: HashMap<String, String>,
    pub(crate) recursive: bool,
    pub(crate) path_patterns: Vec<String>,
    pub(crate) parser: parsers::Config,
}

pub(crate) struct OpendalExtractor {
    op: Operator,
    polling_interval: Duration,
    recursive: bool,
    filter: Filter,
    parser: ParserEnum,
}

impl OpendalExtractor {
    pub(crate) fn try_from(value: &Config, next: EventSourcePipe) -> Result<Self> {
        let op: Operator = Operator::via_iter(value.kind, value.parameters.clone())?;
        let filter = Filter::from_patterns(FilePatternMatcher::from(&value.path_patterns)?);
        let parser = value.parser.make_parser(next)?;
        Ok(Self {
            op,
            polling_interval: value.polling_interval,
            recursive: value.recursive,
            filter,
            parser,
        })
    }

    #[instrument(skip(self))]
    pub(crate) async fn run_once(&mut self) -> Result<()> {
        let op = &self.op;
        let filter = &self.filter;
        let recursive = self.recursive;
        let parser = &mut self.parser;
        // TODO convert into arg of instrument
        tracing::debug!(filter=? filter, scheme =? op.info().scheme(), root =? op.info().root(), "scanning");
        let mut lister = op
        .lister_with("")
        .recursive(recursive)
        // Make sure content-length and last-modified been fetched.
        .metakey(Metakey::ContentLength | Metakey::LastModified)
        .await?;
        while let Some(entry) = lister.try_next().await? {
            if filter.accept(&entry) {
                if let Err(err) = parser.parse(op, &entry).await {
                    tracing::warn!(?err, path = entry.path(), "fail to process, skip");
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Extractor for OpendalExtractor {
    async fn run(&mut self) -> Result<()> {
        loop {
            if let Err(err) = self.run_once().await {
                tracing::warn!(?err, filter = ?self.filter, scheme =? self.op.info().scheme(), root =? self.op.info().root(), "fail during scanning");
            }
            sleep(self.polling_interval).await;
            self.filter.jump_to_next_ts_window();
        }
    }
}
