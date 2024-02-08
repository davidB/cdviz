use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::errors::{Error, Result};
use cdevents_sdk::CDEvent;
use chrono::DateTime;
use chrono::Utc;
use futures::TryStreamExt;
use opendal::services::Fs;
use opendal::Entry;
use opendal::EntryMode;
use opendal::Metakey;
use opendal::Operator;
use opendal::Scheme;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::instrument;

pub struct Source {
    op: Operator,
    poll_interval: Duration,
}

impl Source {
    pub fn from_local_path<P>(p: P) -> Result<Self>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let mut builder = Fs::default();
        builder.root(
            p.as_ref()
                .to_str()
                .ok_or_else(|| Error::from(format!("failed to convert into str: {:?}", p)))?,
        );
        let op: Operator = Operator::new(builder)?.finish();

        Ok(Self {
            op,
            poll_interval: Duration::from_secs(5),
        })
    }
    pub fn from_config() -> Result<Self> {
        let map = HashMap::from([
            // Set the root for fs, all operations will happen under this root.
            //
            // NOTE: the root must be absolute path.
            ("root".to_string(), "/tmp".to_string()),
        ]);

        // Build an `Operator` to start operating the storage.
        let op: Operator = Operator::via_map(Scheme::Fs, map)?;

        Ok(Self {
            op,
            poll_interval: Duration::from_secs(5),
        })
    }

    pub async fn start(self, tx: Sender<CDEvent>) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            // Process each socket concurrently.
            run_in_loop(tx, self.op, self.poll_interval).await
        })
    }
}

pub async fn run_in_loop(tx: Sender<CDEvent>, op: Operator, poll_interval: Duration) -> Result<()> {
    let mut after = DateTime::<Utc>::MIN_UTC;
    loop {
        let before = Utc::now();
        if let Err(err) = run_once(&tx, &op, (&after, &before)).await {
            tracing::warn!(?err, after =? after, before =? before, scheme =? op.info().scheme(), root =? op.info().root(), "fail on scanning");
        }
        after = before;
        sleep(poll_interval).await;
    }
}

#[instrument]
pub async fn run_once(
    tx: &Sender<CDEvent>,
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
            }
        }
    }
    Ok(())
}

async fn process_entry(tx: &Sender<CDEvent>, op: &Operator, entry: &Entry) -> Result<()> {
    let read = op.read(entry.path()).await?;
    let cdevent: CDEvent = serde_json::from_slice::<CDEvent>(&read)?;
    tx.send(cdevent).await.map_err(Error::from)
}
