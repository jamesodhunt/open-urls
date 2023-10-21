use crate::logger;
use crate::url;
use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about = "URL opening tool")]
pub struct AppArgs {
    #[clap(short, long, value_name = "CONFIG-FILE")]
    pub cfg: String,

    #[clap(short, long)]
    pub browser: Option<String>,

    // Without a delay, web browsers can quickly become overwhelmed
    // by a large number of URL open requests.
    #[clap(short, long, default_value = "1s")]
    pub delay: Option<String>,

    #[clap(
        short = 'n',
        long,
        help = "No act mode (just show what would be done)",
        default_value_t = false
    )]
    pub dry_run: bool,

    #[clap(long, help = "force *all* URLs to open in the background")]
    pub background: Option<bool>,

    #[clap(short, long, default_value = Some("info"))]
    pub log_level: Option<String>,

    #[clap(short, long, help = "output log information in JSON format")]
    pub use_json: bool,
}

pub fn handle() -> Result<()> {
    let cli = AppArgs::parse();

    let log_level = cli
        .log_level
        .ok_or("need log level")
        .map_err(|e| anyhow!(e))?;

    logger::setup_logging(&log_level, cli.use_json)?;

    url::handle_urls_file(&cli.cfg, cli.delay, cli.browser, cli.dry_run)
}
