use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no watcher found (configured or started)")]
    NoWatcher,
    #[error("no sink found (configured or started)")]
    NoSink,
    #[error(transparent)]
    WatchDirectory(#[from] notify::Error),
    #[error(transparent)]
    InitTracing(#[from] init_tracing_opentelemetry::Error),
    #[error(transparent)]
    Http(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    // #[error(transparent)]
    // Other(#[from] anyhow::Error),
}
