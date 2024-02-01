use crate::{errors::Result, CDEvent};
use notify::{
    event::{AccessKind, AccessMode},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::sync::mpsc::Sender;

// based on https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs
pub(crate) async fn watch<P: AsRef<Path>>(
    tx: Sender<CDEvent>,
    path: P,
) -> Result<Box<dyn Watcher>> {
    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                //dbg!(&res);
                if let Ok(event) = res {
                    if let Some(cdevents) = maybe_to_cdevents(event) {
                        for cdevent in cdevents {
                            let sent = tx.send(cdevent).await;
                            if sent.is_err() {
                                tracing::warn!(?sent);
                            }
                        }
                    }
                } else {
                    tracing::warn!(?res);
                }
            })
        },
        Config::default(),
    )?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    tracing::info!(path = ?path.as_ref(), "start watching directory");
    Ok(Box::new(watcher))
}

fn maybe_to_cdevents(event: Event) -> Option<Vec<CDEvent>> {
    // Access is called after creation or modification of a file
    if event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)) {
        let v: Vec<CDEvent> = event
            .paths
            .into_iter()
            .filter(|p| p.is_file() && (p.extension().unwrap_or_default() == "json"))
            .filter_map(maybe_to_cdevent)
            .collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    } else {
        None
    }
}

fn maybe_to_cdevent(p: PathBuf) -> Option<CDEvent> {
    fs::read_to_string(p)
        .map_err(|error| tracing::warn!(?error))
        .ok()
        .and_then(|txt| {
            serde_json::from_str(&txt)
                .map_err(|error| tracing::warn!(?error))
                .ok()
        })
        .map(|json| CDEvent { json })
}
