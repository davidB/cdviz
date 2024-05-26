mod errors;
mod sinks;
mod sources;

use std::{collections::HashMap, path::PathBuf};

use cdevents_sdk::CDEvent;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use errors::{Error, Result};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use futures::future::TryJoinAll;
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;
use tokio::sync::broadcast;

#[derive(Debug, Clone, clap::Parser)]
pub(crate) struct Cli {
    #[clap(
        long = "config",
        env("CDVIZ_COLLECTOR_CONFIG"),
        default_value = "cdviz-collector.toml"
    )]
    config: PathBuf,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    sources: HashMap<String, sources::Config>,
    sinks: HashMap<String, sinks::Config>,
}

type Sender<T> = tokio::sync::broadcast::Sender<T>;
type Receiver<T> = tokio::sync::broadcast::Receiver<T>;

#[derive(Clone, Debug)]
struct Message {
    // received_at: OffsetDateTime,
    cdevent: CDEvent,
    //raw: serde_json::Value,
}

impl From<CDEvent> for Message {
    fn from(value: CDEvent) -> Self {
        Self {
            // received_at: OffsetDateTime::now_utc(),
            cdevent: value,
        }
    }
}

fn init_log(verbose: Verbosity) -> Result<()> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .ok()
            .or_else(|| verbose.log_level().map(|l| l.to_string()))
            .unwrap_or_else(|| "off".to_string()),
    );
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    Ok(())
}

//TODO add garcefull shutdown
//TODO use logfmt
//TODO use verbosity to configure tracing & log, but allow override and finer control with RUST_LOG & CDVIZ_COLLECTOR_LOG (higher priority)
//TODO add a `enabled: bool` field as part of the config of each sources & sinks
//TODO provide default config, and default values for some config fields
//TODO document the architecture and the configuration
//TODO add transformers ( eg file/event info, into cdevents) for sources
//TODO integrations with cloudevents (sources & sink)
//TODO integrations with kafka / redpanda, nats,
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_log(cli.verbose)?;

    if !cli.config.exists() {
        return Err(errors::Error::ConfigNotFound {
            path: cli.config.to_string_lossy().to_string(),
        });
    }
    if let Some(dir) = cli.config.parent() {
        std::env::set_current_dir(dir)?;
    }
    let config: Config = Figment::new()
        .merge(Toml::file(cli.config.as_path()))
        .merge(Env::prefixed("CDVIZ_COLLECTOR_"))
        .extract()?;

    let (tx, _) = broadcast::channel::<Message>(100);

    let sinks = config
        .sinks
        .into_iter()
        .inspect(|(name, _config)| tracing::info!(kind = "sink", name, "starting"))
        .map(|(name, config)| sinks::start(name, config, tx.subscribe()))
        .collect::<Vec<_>>();

    if sinks.is_empty() {
        tracing::error!("no sink configured or started");
        return Err(errors::Error::NoSink);
    }

    let sources = config
        .sources
        .into_iter()
        .inspect(|(name, _config)| tracing::info!(kind = "source", name, "starting"))
        .map(|(name, config)| sources::start(name, config, tx.clone()))
        .collect::<Vec<_>>();

    if sources.is_empty() {
        tracing::error!("no source configured or started");
        return Err(errors::Error::NoSource);
    }

    //TODO use tokio JoinSet?
    sinks
        .into_iter()
        .chain(sources)
        .collect::<TryJoinAll<_>>()
        .await
        .map_err(|err| Error::from(err.to_string()))?;
    // handlers.append(&mut sinks);
    // handlers.append(&mut sources);
    //tokio::try_join!(handlers).await?;
    //futures::try_join!(handlers);
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    impl proptest::arbitrary::Arbitrary for Message {
        type Parameters = ();
        type Strategy = proptest::strategy::BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use proptest::prelude::*;
            (any::<CDEvent>()).prop_map(Message::from).boxed()
        }
    }

    #[rstest]
    fn read_samples_config(#[files("../**/cdviz-collector.toml")] path: PathBuf) {
        assert!(path.exists());
        let _config: Config = Figment::new().merge(Toml::file(path)).extract().unwrap();
    }
}
