mod app;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.exit != true {
        terminal.draw(|frame| view(&model, frame))?;
        update(&mut model)?;
    }
    ratatui::restore();
    Ok(())
}

#[derive(Debug, Default)]
pub struct Model {
    counter: u8,
    exit: bool,
}

impl Model {
    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

fn update(app: &mut Model) -> io::Result<()> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Left => app.decrement_counter(),
            KeyCode::Right => app.increment_counter(),
            _ => {}
        },
        _ => {}
    };
    Ok(())
}

fn view(app: &Model, frame: &mut Frame) {
    let title = Line::from(" Counter App Tutorial ".bold());
    let instructions = Line::from(vec![
        " Decrement ".into(),
        "<Left>".blue().bold(),
        " Increment ".into(),
        "<Right>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let counter_text = Text::from(vec![Line::from(vec![
        "Value: ".into(),
        app.counter.to_string().yellow(),
    ])]);

    let widget = Paragraph::new(counter_text).centered().block(block);

    frame.render_widget(widget, frame.area());
}
