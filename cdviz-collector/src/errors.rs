use std::{convert::Infallible, str::FromStr};

use thiserror::Error;

use crate::Message;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("no source found (configured or started)")]
    NoSource,
    #[error("no sink found (configured or started)")]
    NoSink,
    // #[error(transparent)]
    // WatchDirectory(#[from] notify::Error),
    #[cfg(feature = "sink_db")]
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    InitTracing(#[from] init_tracing_opentelemetry::Error),
    #[error(transparent)]
    Http(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    HttpReqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "source_opendal")]
    #[error(transparent)]
    Opendal(#[from] opendal::Error),
    #[cfg(feature = "source_opendal")]
    #[error(transparent)]
    GlobPattern(#[from] globset::Error),
    #[error(transparent)]
    BusSend(#[from] tokio::sync::broadcast::error::SendError<Message>),
    #[error(transparent)]
    BusRecv(#[from] tokio::sync::broadcast::error::RecvError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ConfigReader(#[from] figment::Error),
    #[error(transparent)]
    CloudEventBuilder(#[from] cloudevents::event::EventBuilderError),
    #[error(transparent)]
    CloudEventMessage(#[from] cloudevents::message::Error),
    // #[error(transparent)]
    // ConfigTomlError(#[from] toml::de::Error),
    #[error("{txt}")]
    Custom { txt: String },
    // #[error(transparent)]
    // Other(#[from] anyhow::Error),
}

pub(crate) fn to_err<T>(txt: T) -> Error
where
    T: Into<String>,
{
    Error::Custom { txt: txt.into() }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        to_err(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        to_err(value)
    }
}

impl FromStr for Error {
    type Err = Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(to_err(s))
    }
}
