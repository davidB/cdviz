use std::path::PathBuf;

use crate::{
    config,
    errors::{self, Error, Result},
    sinks, sources,
};
use cdevents_sdk::CDEvent;
use clap::Args;
use futures::future::TryJoinAll;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Args)]
#[command(args_conflicts_with_subcommands = true,flatten_help = true, about, long_about = None)]
pub(crate) struct ConnectArgs {
    /// The configuration file to use.
    #[clap(long = "config", env("CDVIZ_COLLECTOR_CONFIG"))]
    config: Option<PathBuf>,

    /// The directory to use as the working directory.
    #[clap(short = 'C', long = "directory")]
    directory: Option<PathBuf>,
}

pub(crate) type Sender<T> = tokio::sync::broadcast::Sender<T>;
pub(crate) type Receiver<T> = tokio::sync::broadcast::Receiver<T>;

#[derive(Clone, Debug)]
pub(crate) struct Message {
    // received_at: OffsetDateTime,
    pub(crate) cdevent: CDEvent,
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

//TODO add garcefull shutdown
//TODO add transformers ( eg file/event info, into cdevents) for sources
//TODO integrations with cloudevents (sources & sink)
//TODO integrations with kafka / redpanda, nats,
/// retuns true if the connection service ran successfully
pub(crate) async fn connect(args: ConnectArgs) -> Result<bool> {
    let config = config::Config::from_file(args.config)?;

    if let Some(dir) = args.directory {
        std::env::set_current_dir(dir)?;
    }

    let (tx, _) = broadcast::channel::<Message>(100);

    let sinks = config
        .sinks
        .into_iter()
        .filter(|(_name, config)| config.is_enabled())
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
        .filter(|(_name, config)| config.is_enabled())
        .inspect(|(name, _config)| tracing::info!(kind = "source", name, "starting"))
        .map(|(name, config)| sources::start(&name, config, tx.clone()))
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
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl proptest::arbitrary::Arbitrary for Message {
        type Parameters = ();
        type Strategy = proptest::strategy::BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use proptest::prelude::*;
            (any::<CDEvent>()).prop_map(Message::from).boxed()
        }
    }
}
