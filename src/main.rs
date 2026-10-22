mod config;
mod controller;
mod datastore;
mod model;
mod view;

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::Parser;
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use ratatui::DefaultTerminal;
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::controller::Controller;
use crate::datastore::{Datastore, Storer, sqlite};
use crate::model::{Model, memory};
use crate::view::View;

fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Config::new(args.config_path);
    let terminal = ratatui::init();

    let rt = Runtime::new()?;
    let (mut controller, storer) = get_controller(config, terminal, args.ephemeral);
    let jh = storer.map(|x| rt.spawn_blocking(|| x.run()));
    controller.run()?;
    jh.map(async |x| rt.block_on(x));
    ratatui::restore();
    Ok(())
}

fn get_controller(
    config: Config,
    terminal: DefaultTerminal,
    ephemeral: bool,
) -> (Controller, Option<sqlite::Storer>) {
    if !ephemeral
        && let Some(db_path) = config.db_path()
        && let Ok(conn) = establish_connection(db_path)
    {
        let mut datastore = sqlite::Datastore::new(conn);
        if let Ok(mut model) = datastore.load()
            && let Ok((storer, controller)) = datastore.into_storer_controller()
        {
            model.set_datastore(Box::new(controller));

            return (
                Controller::new(View::new(Box::new(model), terminal, config.view_config())),
                Some(storer),
            );
        }
    }
    (
        Controller::new(
            View::new(
                Box::new(memory::Model::default()),
                terminal,
                config.view_config(),
            )
            .background_text("entries will not be saved when you quit".to_string()),
        ),
        None,
    )
}

#[derive(Parser)]
#[command(version, about="a bullet journal for your terminal", long_about = None)]
pub struct Cli {
    /// the path to a config toml to use instead of the default
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,
    /// use an in-memory model rather than a database backed model
    #[arg(short, long, default_value_t = false)]
    pub ephemeral: bool,
}

fn establish_connection(path: &Path) -> Result<SqliteConnection> {
    let mut sqlite_conn = SqliteConnection::establish(
        path.to_str()
            .ok_or(anyhow!("cannot create a db connection to path"))?,
    )?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS `events` (
	`date` INT NOT NULL,
	`index` INT NOT NULL,
	`title` TEXT NOT NULL,
	`importance` INT NOT NULL)",
    )
    .execute(&mut sqlite_conn)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS `tasks` (
	`date` INT NOT NULL,
	`index` INT NOT NULL,
	`title` TEXT NOT NULL,
	`completion_level` INT NOT NULL)",
    )
    .execute(&mut sqlite_conn)?;

    Ok(sqlite_conn)
}
