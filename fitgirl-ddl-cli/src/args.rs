use std::path::PathBuf;

use palc::Parser;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "extract direct download links from fitgirl repacks"
)]
pub struct CliArgs {
    /// number of workers to spawn
    #[arg(long, default_value_t = 2)]
    pub workers: usize,

    /// directory to save generated aria2 input files
    #[arg(long, default_value_t = PathBuf::from("."))]
    pub save_dir: PathBuf,

    /// url of the game, format is like:
    ///
    /// https://fitgirl-repacks.site/the-bards-tale-iv-directors-cut/
    ///
    /// you can type multiple game urls as different arguments
    #[arg(value_name = "GAME_URL")]
    pub game_urls: Vec<String>,
}
