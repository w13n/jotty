mod cli;
mod config;
mod controller;
mod model;
mod view;

use anyhow::Result;

use clap::Parser;
use ratatui::DefaultTerminal;

use crate::config::Config;
use crate::controller::Controller;
use crate::model::sqlite_model::establish_connection;
use crate::model::{MemModel, SqliteModel};
use crate::view::View;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    let config = Config::new(args.config_path);
    println!("{:#?}", &config);
    let terminal = ratatui::init();
    let mut controller = get_controller(config, terminal, args.ephemeral);
    controller.run()?;
    ratatui::restore();
    Ok(())
}

fn get_controller(config: Config, terminal: DefaultTerminal, ephemeral: bool) -> Controller {
    if !ephemeral
        && let Some(db_path) = config.db_path()
        && let Some(conn) = establish_connection(db_path).ok()
    {
        Controller::new(View::new(
            Box::new(SqliteModel::new(conn)),
            terminal,
            config.view_config(),
        ))
    } else {
        Controller::new(
            View::new(
                Box::new(MemModel::default()),
                terminal,
                config.view_config(),
            )
            .background_text("entries will not be saved when you quit".to_string()),
        )
    }
}
