use config::ConfigError;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{
    fs::{OpenOptions, create_dir_all},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    db_path: Option<PathBuf>,
    view: ViewConfig,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ViewConfig {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub event_color: Color,
    pub task_color: Color,
    pub strike_completed: bool,
}

impl Config {
    pub fn new(config_path: Option<PathBuf>) -> Config {
        let config_file = config_path.or(directories::ProjectDirs::from("com", "w13n", "jotty")
            .map(|x| {
                let mut path = x.config_dir().to_path_buf();
                path.push("config.toml");
                path
            }));
        let default = Self::default();
        match config_file {
            Some(config_file) => {
                let _ = create_file_if_not_exists(&config_file);
                match Self::build(&default, config_file) {
                    Ok(val) => val,
                    Err(_err) => default,
                }
            }
            None => Self::default(),
        }
        .create_db()
    }

    fn build(default: &Self, config_file: PathBuf) -> Result<Self, ConfigError> {
        let defaults = config::Config::try_from(&default).expect("default is a valid config");
        let config = config::Config::builder()
            .add_source(defaults)
            .add_source(config::File::from(config_file));
        config.build()?.try_deserialize()
    }

    fn create_db(mut self) -> Self {
        if let Some(db_path) = &self.db_path
            && create_file_if_not_exists(db_path).is_ok()
        {
            return self;
        }
        self.db_path = None;
        self
    }

    pub fn db_path(&self) -> &Option<PathBuf> {
        &self.db_path
    }

    pub fn view_config(&self) -> ViewConfig {
        self.view
    }
}

impl Default for Config {
    fn default() -> Self {
        let db_path = directories::ProjectDirs::from("com", "w13n", "jotty").map(|x| {
            let mut path = x.data_dir().to_path_buf();
            path.push("v1.db");
            path
        });
        Self {
            db_path,
            view: ViewConfig::default(),
        }
    }
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            primary_color: Color::Green,
            secondary_color: Color::Blue,
            event_color: Color::Red,
            task_color: Color::Yellow,
            strike_completed: true,
        }
    }
}

fn create_file_if_not_exists(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false) // leave existing contents alone
        .open(path)?;

    Ok(())
}
