use std::path::PathBuf;

use reqwest::Url;

#[derive(Debug, Clone, clap::Parser)]
pub struct Settings {
    /// watch a local file system directory
    /// (create on event per valid cdevents json file)
    #[clap(long)]
    pub watch_directory: Option<PathBuf>,

    /// push cdevents as json to this url
    #[clap(long)]
    pub sink_http: Option<Url>,

    /// push cdevents to log
    #[clap(long)]
    pub sink_debug: bool,
}
