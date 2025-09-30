use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// some special input argument
    #[arg(short, long, conflicts_with = "ephemeral")]
    pub database_path: Option<PathBuf>,

    #[arg(short, long, default_value_t = false)]
    pub ephemeral: bool,
}
