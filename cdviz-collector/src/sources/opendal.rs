use crate::errors::{Error, Result};
use crate::{Message, Sender};
use cdevents_sdk::CDEvent;
use chrono::DateTime;
use chrono::Utc;
use futures::TryStreamExt;
use opendal::Entry;
use opendal::EntryMode;
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

use super::Source;

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    #[serde(with = "humantime_serde")]
    polling_interval: Duration,
    #[serde_as(as = "DisplayFromStr")]
    kind: Scheme,
    parameters: HashMap<String, String>,
}

impl TryFrom<Config> for OpendalSource {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let op: Operator = Operator::via_map(value.kind, value.parameters)?;
        Ok(Self {
            op,
            polling_interval: value.polling_interval,
        })
    }
}

pub(crate) struct OpendalSource {
    op: Operator,
    polling_interval: Duration,
}

// impl OpendalSource {
//     pub(crate) fn from_local_path<P>(p: P) -> Result<Self>
//     where
//         P: AsRef<Path> + std::fmt::Debug,
//     {
//         let mut builder = Fs::default();
//         builder.root(
//             p.as_ref()
//                 .to_str()
//                 .ok_or_else(|| Error::from(format!("failed to convert into str: {:?}", p)))?,
//         );
//         let op: Operator = Operator::new(builder)?.finish();

//         Ok(Self {
//             op,
//             polling_interval: Duration::from_secs(5),
//         })
//     }
// }

impl Source for OpendalSource {
    async fn run(&self, tx: Sender<Message>) -> Result<()> {
        let mut after = DateTime::<Utc>::MIN_UTC;
        loop {
            let before = Utc::now();
            if let Err(err) = run_once(&tx, &self.op, (&after, &before)).await {
                tracing::warn!(?err, after =? after, before =? before, scheme =? self.op.info().scheme(), root =? self.op.info().root(), "fail during scanning");
            }
            after = before;
            sleep(self.polling_interval).await;
        }
    }
}

#[instrument]
pub(crate) async fn run_once(
    tx: &Sender<Message>,
    op: &Operator,
    ts_beetween: (&DateTime<Utc>, &DateTime<Utc>),
) -> Result<()> {
    let (after, before) = ts_beetween;
    // TODO convert into arg of instrument
    tracing::debug!(after =? after, before =? before, scheme =? op.info().scheme(), root =? op.info().root(), "scanning");
    let mut lister = op
        .lister_with("")
        // Make sure content-length and last-modified been fetched.
        .metakey(Metakey::ContentLength | Metakey::LastModified)
        .await?;
    while let Some(entry) = lister.try_next().await? {
        let meta = entry.metadata();
        if meta.mode() == EntryMode::FILE {
            if let Some(last) = meta.last_modified() {
                if &last > after && &last <= before && meta.content_length() > 0 {
                    if let Err(err) = process_entry(tx, op, &entry).await {
                        tracing::warn!(?err, path = entry.path(), "fail to process, skip")
                    }
                }
            } else {
                tracing::warn!(
                    path = entry.path(),
                    "can not read last modified timestamp, skip"
                )
            }
        }
    }
    Ok(())
}

async fn process_entry(tx: &Sender<Message>, op: &Operator, entry: &Entry) -> Result<usize> {
    let read = op.read(entry.path()).await?;
    let cdevent: CDEvent = serde_json::from_slice::<CDEvent>(&read)?;
    tx.send(cdevent.into()).map_err(Error::from)
}
