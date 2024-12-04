mod config;
mod errors;
mod pipes;
mod sinks;
mod sources;

use std::path::PathBuf;

use cdevents_sdk::CDEvent;
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use errors::{Error, Result};
use futures::future::TryJoinAll;
use init_tracing_opentelemetry::tracing_subscriber_ext::TracingGuard;
// use time::OffsetDateTime;
use tokio::sync::broadcast;

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

// TODO add options (or subcommand) to `check-configuration` (regardless of enabled), `configuration-dump` (after consolidation (with filter or not enabled) and exit or not),
// TODO add options to overide config from cli arguments (like from env)
#[derive(Debug, Clone, Parser)]
pub(crate) struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// connect sources to sinks
    #[command(arg_required_else_help = true)]
    Connect(ConnectArgs),
}

#[derive(Debug, Clone, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct ConnectArgs {
    /// The configuration file to use.
    #[clap(long = "config", env("CDVIZ_COLLECTOR_CONFIG"))]
    config: Option<PathBuf>,
    /// The directory to use as the working directory.
    #[clap(short = 'C', long = "directory")]
    directory: Option<PathBuf>,
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

fn init_log(verbose: &Verbosity) -> Result<TracingGuard> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .ok()
            .or_else(|| verbose.log_level().map(|level| level.to_string()))
            .unwrap_or_else(|| "off".to_string()),
    );
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().map_err(Error::from)
}

//TODO add garcefull shutdown
//TODO use logfmt
//TODO use verbosity to configure tracing & log, but allow override and finer control with RUST_LOG & CDVIZ_COLLECTOR_LOG (higher priority)
//TODO document the architecture and the configuration
//TODO add transformers ( eg file/event info, into cdevents) for sources
//TODO integrations with cloudevents (sources & sink)
//TODO integrations with kafka / redpanda, nats,
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _guard = init_log(&cli.verbose)?;
    match cli.command {
        Commands::Connect(args) => connect(args).await,
    }
}

async fn connect(args: ConnectArgs) -> Result<()> {
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
    Ok(())
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
