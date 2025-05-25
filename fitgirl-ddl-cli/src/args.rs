use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "extract direct download links from fitgirl-repacks.site")]
pub struct Args {
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
