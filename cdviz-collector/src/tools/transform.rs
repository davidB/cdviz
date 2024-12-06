use crate::{
    config,
    errors::Result,
    pipes::Pipe,
    sources::{opendal as source_opendal, transformers, EventSource, EventSourcePipe},
};
use clap::{Args, ValueEnum};
use opendal::Scheme;
use std::{collections::HashMap, path::PathBuf};

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

pub(crate) async fn transform(args: TransformArgs) -> Result<bool> {
    let config = config::Config::from_file(args.config)?;

    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }

    let mut pipe: EventSourcePipe = Box::new(OutputToJsonFile { directory: args.output.clone() });
    let mut tconfigs =
        transformers::resolve_transformer_refs(&args.transformer_refs, &config.transformers)?;
    tconfigs.reverse();
    for tconfig in tconfigs {
        pipe = tconfig.make_transformer(pipe)?;
    }
    let config_extractor = source_opendal::Config {
        polling_interval: std::time::Duration::ZERO,
        kind: Scheme::Fs,
        parameters: HashMap::from([("root".to_string(), args.input.to_string_lossy().to_string())]),
        recursive: false,
        path_patterns: vec![
            "**/*.json".to_string(),
            "!**/*.out.json".to_string(),
            "!**/*.new.json".to_string(),
        ],
        parser: source_opendal::parsers::Config::Json,
    };
    source_opendal::OpendalExtractor::try_from(&config_extractor, pipe)?.run_once().await?;
    // TODO process .new.json vs .out.json using the self.mode strategy
    let res = match args.mode {
        TransformMode::Review => review(&args.output),
        TransformMode::Check => check(&args.output),
        TransformMode::Overwrite => overwrite(&args.output),
    };
    remove_new_files(&args.output)?;
    res
}

struct OutputToJsonFile {
    directory: PathBuf,
}

impl Pipe for OutputToJsonFile {
    type Input = EventSource;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        let filename = input.metadata["name"].as_str().unwrap();
        let filename = filename.replace(".json", ".new.json");
        let path = self.directory.join(filename);
        std::fs::write(path, serde_json::to_string_pretty(&input)?)?;
        Ok(())
    }
}

fn overwrite(output: &PathBuf) -> Result<bool> {
    let mut count = 0;
    for entry in std::fs::read_dir(output)? {
        let path = entry?.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        if filename.ends_with(".new.json") {
            let out_filename = filename.replace(".new.json", ".out.json");
            let out_path = path.with_file_name(out_filename);
            std::fs::rename(path, out_path)?;
            count += 1;
        }
    }
    println!("Overwritten {} files.", count);
    Ok(true)
}

fn check(output: &PathBuf) -> Result<bool> {
    let differences = crate::tools::difference::search_new_vs_out(output)?;
    if !differences.is_empty() {
        println!("Differences found:");
        for (comparison, diff) in differences {
            diff.show(&comparison);
        }
        Ok(false)
    } else {
        println!("NO differences found.");
        Ok(true)
    }
}

fn review(output: &PathBuf) -> Result<bool> {
    let differences = crate::tools::difference::search_new_vs_out(output)?;
    if !differences.is_empty() {
        println!("Differences found:");
        let mut no_differences = true;
        for (comparison, diff) in differences {
            no_differences = diff.review(&comparison)? && no_differences;
        }
        Ok(no_differences)
    } else {
        println!("NO differences found.");
        Ok(true)
    }
}

fn remove_new_files(output: &PathBuf) -> Result<()> {
    for entry in std::fs::read_dir(output)? {
        let path = entry?.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        if filename.ends_with(".new.json") {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}
