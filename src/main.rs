use std::io;

use crate::app::{CompletionLevel, Entry, Importance, Model, Task};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::Color;
use ratatui::widgets::ListItem;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, ListDirection, Paragraph, Widget},
};

mod app;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut model = Model::new();

    let mut entry = Entry::new();
    entry.push_event(app::Event::new("NAMI HERE", Importance::Extreme));
    entry.push_event(app::Event::new("birthday!!", Importance::High));
    entry.push_event(app::Event::new("Work", Importance::Normal));
    entry.push_event(app::Event::new("cook dinner", Importance::Low));

    entry.push_task(Task::new("code stuff today", CompletionLevel::None));
    entry.push_task(Task::new("call nami", CompletionLevel::Partial));
    entry.push_task(Task::new("play stardew", CompletionLevel::Full));

    model.journal.insert_today(entry);

    while !model.should_exit {
        terminal.draw(|frame| view(&mut model, frame))?;
        update(&mut model)?;
    }
    ratatui::restore();
    Ok(())
}

fn update(app: &mut Model) -> io::Result<()> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Up => app.up(),
            KeyCode::Down => app.down(),
            KeyCode::Right => app.right(),
            KeyCode::Left => app.left(),
            _ => {}
        },
        _ => {}
    };
    Ok(())
}

fn view(model: &mut Model, frame: &mut Frame) {
    let [schedule_rect, tasks_rect] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(frame.area());

    let entry = model.journal.0.iter().next().unwrap().1;
    let events = &entry.events;
    let tasks = &entry.tasks;

    let schedule_title = Line::from(" Schedule ".red());
    let schedule_block = Block::bordered()
        .title(schedule_title.centered())
        .border_set(border::ROUNDED);
    let schedule_items = events
        .iter()
        .map(|x| ListItem::new(format!("{}", &x.title)));
    let schedule_widget = List::from_iter(schedule_items)
        .block(schedule_block)
        .highlight_style(Style::new().fg(Color::Red));

    let task_title = Line::from(" Tasks ".yellow());
    let task_block = Block::bordered()
        .title(task_title.centered())
        .border_set(border::ROUNDED);
    let task_items = tasks.iter().map(|x| ListItem::new(format_tasks(x)));
    let task_widget = List::from_iter(task_items)
        .block(task_block)
        .highlight_style(Style::new().fg(Color::Yellow));

    frame.render_stateful_widget(schedule_widget, schedule_rect, &mut model.left_state);
    frame.render_stateful_widget(task_widget, tasks_rect, &mut model.right_state);
}

fn format_tasks(task: &Task) -> String {
    match task.completion_level {
        CompletionLevel::None => {
            format!(" ○ {}", &task.title)
        }
        CompletionLevel::Partial => {
            format!(" ◐ {}", &task.title)
        }
        CompletionLevel::Full => {
            format!(" ● {}", &task.title)
        }
    }
}
