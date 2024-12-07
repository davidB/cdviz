use crate::Message;

pub(crate) type Result<T> = std::result::Result<T, Error>;

// Nightly requires enabling this feature:
// #![feature(error_generic_member_access)]
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
#[non_exhaustive]
pub(crate) enum Error {
    #[from(ignore)]
    #[display("config file not found: {path}")]
    ConfigNotFound {
        path: String,
    },
    #[from(ignore)]
    #[display("config of transformer not found: {}", _0)]
    ConfigTransformerNotFound(#[error(ignore)] String),
    #[display("no source found (configured or started)")]
    NoSource,
    #[display("no sink found (configured or started)")]
    NoSink,
    // #[error(transparent)]
    // WatchDirectory(#[error(backtrace, source)] notify::Error),
    #[cfg(feature = "sink_db")]
    Db(sqlx::Error),
    InitTracing(init_tracing_opentelemetry::Error),
    Http(reqwest_middleware::Error),
    HttpReqwest(reqwest::Error),
    Json(serde_json::Error),
    #[cfg(feature = "source_opendal")]
    Opendal(opendal::Error),
    #[cfg(feature = "source_opendal")]
    GlobPattern(globset::Error),
    #[cfg(feature = "transformer_hbs")]
    HandlebarsRender(handlebars::RenderError),
    #[cfg(feature = "transformer_hbs")]
    HandlebarsTemplate(handlebars::TemplateError),
    #[cfg(feature = "source_opendal")]
    Csv(csv::Error),
    BusSend(tokio::sync::broadcast::error::SendError<Message>),
    BusRecv(tokio::sync::broadcast::error::RecvError),
    Io(std::io::Error),
    ConfigReader(figment::Error),
    CloudEventBuilder(cloudevents::event::EventBuilderError),
    CloudEventMessage(cloudevents::message::Error),
    // MutexPoisoned(std::sync::PoisonError<std::sync::MutexGuard<'static, Message>>),
    // MutexPoisoned<T>(std::sync::PoisonError<T>),
    // ConfigTomlError(toml::de::Error),
    MultiHash(multihash::Error),
    #[display("{txt}")]
    Custom {
        txt: String,
    },
    // Other(#[error(backtrace, source)] anyhow::Error),
}

pub(crate) fn to_err<T>(txt: T) -> Error
where
    T: Into<String>,
{
    Error::Custom { txt: txt.into() }
}

// impl From<String> for Error {
//     fn from(value: String) -> Self {
//         to_err(value)
//     }
// }

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        to_err(value)
    }
}

impl std::str::FromStr for Error {
    type Err = std::convert::Infallible;
    fn from_str(txt: &str) -> std::result::Result<Self, Self::Err> {
        Ok(to_err(txt))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use assert2::let_assert;

    use super::*;

    #[test]
    fn string_to_error() {
        let_assert!(Ok(Error::Custom { txt }) = Error::from_str("test0"));
        assert!(txt == "test0");

        let_assert!(Error::Custom { txt } = Error::from("test1"));
        assert!(txt == "test1");

        let_assert!(Error::Custom { txt } = Error::from("test2".to_string()));
        assert!(txt == "test2");
    }
}
