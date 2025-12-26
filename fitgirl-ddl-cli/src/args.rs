use std::{num::NonZeroUsize, path::PathBuf};

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help"))]
/// fitgirl-repacks helper.
pub struct Cli {
    #[argh(subcommand)]
    pub command: Commands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Commands {
    Search(Search),
    Fetch(Fetch),
}

/// search games from fitgirl-repacks
#[derive(FromArgs)]
#[argh(subcommand, name = "search")]
pub struct Search {
    /// search keyword
    #[argh(option)]
    pub query: String,

    /// result page, cannot be zero
    #[argh(option)]
    pub page: NonZeroUsize,
}

/// extract direct download links from fitgirl-repacks.site
#[derive(FromArgs)]
#[argh(subcommand, name = "fetch")]
pub struct Fetch {
    /// number of workers to spawn
    #[argh(option, default = "3")]
    pub workers: usize,

    /// directory to save generated aria2 input files
    #[argh(option, default = "PathBuf::from(\".\")")]
    pub save_dir: PathBuf,

    /// url of the game, format is like:
    ///
    /// https://fitgirl-repacks.site/the-bards-tale-iv-directors-cut/
    ///
    /// you can type multiple game urls as different arguments
    #[argh(positional)]
    pub game_urls: Vec<String>,
}
