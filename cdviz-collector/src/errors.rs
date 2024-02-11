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
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    InitTracing(#[from] init_tracing_opentelemetry::Error),
    #[error(transparent)]
    Http(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Opendal(#[from] opendal::Error),
    #[error(transparent)]
    BusSendError(#[from] tokio::sync::broadcast::error::SendError<Message>),
    #[error(transparent)]
    BusRecvError(#[from] tokio::sync::broadcast::error::RecvError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ConfigReaderError(#[from] figment::Error),
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
