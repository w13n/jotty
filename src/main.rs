mod controller;
mod model;
mod view;

use std::io;

use crate::controller::Controller;
use crate::model::MemModel;
use crate::view::View;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let model = Box::new(MemModel::default());
    let view = View::new(model, terminal);
    let mut controller = Controller::new(view);
    controller.run()?;
    ratatui::restore();
    Ok(())
}
