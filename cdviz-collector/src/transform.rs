use std::path::PathBuf;

use crate::errors::Result;
use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Args)]
#[command(args_conflicts_with_subcommands = true,flatten_help = true, about, long_about = None)]
pub(crate) struct TransformArgs {
    /// The configuration file to use.
    #[clap(long = "config", env("CDVIZ_COLLECTOR_CONFIG"))]
    config: Option<PathBuf>,

    /// Names of transformers to chain (comma separated)
    #[clap(short = 't', long = "transformer-refs", default_value = "passthrough")]
    transformer_refs: Vec<String>,

    /// The input directory with json files.
    #[clap(short = 'i', long = "input")]
    input: PathBuf,

    /// The output directory with json files.
    #[clap(short = 'o', long = "output")]
    output: PathBuf,

    /// How to handle new vs existing output files
    #[clap(short = 'm', long = "mode", value_enum, default_value_t = TransformMode::Review)]
    mode: TransformMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
enum TransformMode {
    /// interactive review generated against existing output
    #[default]
    Review,
    /// overwrite existing output files without checking
    Overwrite,
    /// check generated against existing output and failed on difference
    Check,
}

pub(crate) async fn transform(_args: TransformArgs) -> Result<()> {
    todo!()
}
