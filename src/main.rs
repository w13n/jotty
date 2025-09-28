mod controller;
mod model;
mod view;

use std::io;
use std::path::Path;

use crate::controller::Controller;
use crate::model::sqlite_model::establish_connection;
use crate::model::{MemModel, SqliteModel};
use crate::view::View;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let path = Path::new("test.db");
    let view = if let Ok(conn) = establish_connection(path) {
        let model = Box::new(SqliteModel::new(conn));
        View::new(model, terminal)
    } else {
        let model = Box::new(MemModel::default());
        View::new(model, terminal)
            .background_text("entries will not be saved when you quit".to_string())
    };
    let mut controller = Controller::new(view);
    controller.run()?;
    ratatui::restore();
    Ok(())
}
