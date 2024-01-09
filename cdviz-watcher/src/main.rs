mod errors;
mod http_sink;
mod settings;
mod watch_directory;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use errors::Result;
use http_sink::HttpSink;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CDEvent {
    json: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = settings::Settings::parse();

    // very opinionated init of tracing, look as is source to make your own
    //TODO use logfmt format (with traceid,...) see [tracing-logfmt-otel](https://github.com/elkowar/tracing-logfmt-otel)
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    let (tx, mut rx) = mpsc::channel::<CDEvent>(32);

    let mut watchers_count = 0;
    let _watch_directory_guard = if let Some(directory) = settings.watch_directory {
        let w = watch_directory::watch(tx, directory).await?;
        watchers_count += 1;
        Some(w)
    } else {
        None
    };
    if watchers_count < 1 {
        tracing::error!("no watcher configured or started");
        return Err(errors::Error::NoWatcher);
    }

    let mut sinks = vec![];

    if settings.sink_debug {
        sinks.push(SinkEnum::from(DebugSink {}));
    }

    if let Some(url) = settings.sink_http {
        sinks.push(SinkEnum::from(HttpSink::new(url)));
    }

    if sinks.len() < 1 {
        tracing::error!("no sink configured or started");
        return Err(errors::Error::NoSink);
    }

    while let Some(message) = rx.recv().await {
        for sink in sinks.iter() {
            if let Err(e) = sink.send(&message).await {
                tracing::warn!(?e, ?sink, "failed to send to sink");
            }
        }
    }
    Ok(())
}

#[enum_dispatch(SinkEnum)]
trait Sink {
    async fn send(&self, cdevent: &CDEvent) -> Result<()>;
}

#[enum_dispatch]
#[derive(Debug)]
enum SinkEnum {
    DebugSink,
    HttpSink,
}

#[derive(Debug)]
struct DebugSink;

impl Sink for DebugSink {
    async fn send(&self, cdevent: &CDEvent) -> Result<()> {
        tracing::debug!(?cdevent, "sending");
        Ok(())
    }
}
