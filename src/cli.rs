use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about="a bullet journal for your terminal", long_about = None)]
pub struct Cli {
    /// the path to a sqlite database to use instead of the default
    #[arg(short, long, conflicts_with = "ephemeral")]
    pub database_path: Option<PathBuf>,
    /// use an in-memory model rather than a database backed model
    #[arg(short, long, default_value_t = false)]
    pub ephemeral: bool,
}
