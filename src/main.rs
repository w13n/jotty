mod controller;
mod model;
mod view;

use std::io;
use time::OffsetDateTime;

use crate::controller::update;
use crate::model::{MapBujo, Model};
use crate::view::view;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let date = OffsetDateTime::now_local()
        .unwrap_or(OffsetDateTime::now_utc())
        .date();
    let mut model = Model::new(date, Box::new(MapBujo::default()));

    while !model.should_exit() {
        terminal.draw(|frame| view(&mut model, frame))?;
        update(&mut model)?;
    }
    ratatui::restore();
    Ok(())
}
