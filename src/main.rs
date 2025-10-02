mod cli;
mod controller;
mod model;
mod view;

use std::fs::create_dir_all;
use std::io;

use clap::Parser;
use ratatui::DefaultTerminal;

use crate::controller::Controller;
use crate::model::sqlite_model::establish_connection;
use crate::model::{MemModel, SqliteModel};
use crate::view::View;

fn main() -> io::Result<()> {
    let args = cli::Cli::parse();
    let terminal = ratatui::init();
    let mut controller = get_controller(args, terminal);
    controller.run()?;
    ratatui::restore();
    Ok(())
}

fn get_controller(args: cli::Cli, terminal: DefaultTerminal) -> Controller {
    let path = if args.ephemeral {
        None
    } else if let Some(path) = args.database_path {
        Some(path)
    } else if let Some(mut path) = directories_next::ProjectDirs::from("com", "w13n", "jotty")
        .map(|x| x.data_dir().to_path_buf())
    {
        if create_dir_all(&path).is_ok() {
            path.push("v1.db");
            Some(path)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(conn) = path.and_then(|x| establish_connection(x.as_path()).ok()) {
        Controller::new(View::new(Box::new(SqliteModel::new(conn)), terminal))
    } else {
        Controller::new(
            View::new(Box::new(MemModel::default()), terminal)
                .background_text("entries will not be saved when you quit".to_string()),
        )
    }
}
