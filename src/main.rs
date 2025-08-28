use std::io;

use crate::app::{CompletionLevel, Entry, Importance, Model, Task};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Position;
use ratatui::prelude::Color;
use ratatui::widgets::ListItem;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, List, Paragraph},
};
use time::{Date, Month};

mod app;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let date = Date::from_calendar_date(2025, Month::August, 15).unwrap();
    let mut model = Model::new(date);

    let mut entry = Entry::new();
    entry.push_event(app::Event::new("NAMI HERE", Importance::Extreme));
    entry.push_event(app::Event::new("birthday!!", Importance::High));
    entry.push_event(app::Event::new("Work", Importance::Normal));
    entry.push_event(app::Event::new("cook dinner", Importance::Low));

    entry.push_task(Task::new("code stuff today", CompletionLevel::None));
    entry.push_task(Task::new("call nami", CompletionLevel::Partial));
    entry.push_task(Task::new("play stardew", CompletionLevel::Full));

    model.journal.insert_with(date, entry);

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
            KeyCode::Up => app.move_up(),
            KeyCode::Down => app.move_down(),
            KeyCode::Left => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    app.move_to_prev()
                } else if app.editing().is_some() {
                    app.move_cursor_left()
                } else {
                    app.move_left()
                }
            }
            KeyCode::Right => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    app.move_to_next()
                } else if app.editing().is_some() {
                    app.move_cursor_right()
                } else {
                    app.move_right()
                }
            }
            KeyCode::Backspace => app.delete_char(),
            KeyCode::Enter => {
                if app.editing().is_some() {
                    app.exit_editing_mode()
                } else {
                    app.enter_editing_mode()
                }
            }
            KeyCode::Esc => app.exit_editing_mode(),
            KeyCode::Char(c) => {
                if app.editing().is_some() {
                    app.insert_char(c)
                } else {
                    match c {
                        'q' => app.exit(),
                        ' ' => app.cycle_task(),
                        'n' => app.create_new_entry(),
                        't' => app.move_to_today(),
                        'e' => app.enter_editing_mode(),
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    };
    Ok(())
}

fn view(model: &mut Model, frame: &mut Frame) {
    let [_top, middle, _bottom] =
        Layout::vertical([Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)])
            .areas(frame.area());
    let [events_rect, tasks_rect] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(middle);

    let title = Line::from(vec![
        "Jotty".green().bold(),
        " entry on ".bold(),
        model.date.to_string().blue().bold(),
    ]);
    let instructions =
        Line::from("<q> to quit; <←↑↓→> to navigate; <SPACE> to cycle; <ENTER> to type".gray());
    let container_block = Block::new()
        .title(title.centered())
        .title_bottom(instructions.centered());

    let container = Paragraph::new("no entry for this date")
        .centered()
        .block(container_block);

    frame.render_widget(container, frame.area());

    if let Some(entry) = model.journal.0.get(&model.date) {
        let events = &entry.events;
        let tasks = &entry.tasks;

        let events_title = Line::from(" Events ".red().bold());
        let events_block = Block::bordered()
            .title(events_title.centered())
            .border_set(border::ROUNDED);
        let events_items = events.iter().map(|x| ListItem::new(x.title.to_string()));
        let events_widget = List::from_iter(events_items)
            .block(events_block)
            .highlight_style(Style::new().fg(Color::Red));

        let task_title = Line::from(" Tasks ".yellow().bold());
        let task_block = Block::bordered()
            .title(task_title.centered())
            .border_set(border::ROUNDED);
        let task_items = tasks.iter().map(|x| ListItem::new(format_tasks(x)));
        let task_widget = List::from_iter(task_items)
            .block(task_block)
            .highlight_style(Style::new().fg(Color::Yellow));

        frame.render_stateful_widget(events_widget, events_rect, &mut model.events_state);
        frame.render_stateful_widget(task_widget, tasks_rect, &mut model.task_state);
        if let Some(offset) = model.editing() {
            let is_events_side = model.events_state.selected().is_some();
            let selected = if is_events_side {
                model.events_state.selected().unwrap()
            } else {
                model.task_state.selected().unwrap()
            };
            let position = if is_events_side {
                Position::new(
                    events_rect.x + 1 + offset,
                    events_rect.y + 1 + selected as u16,
                )
            } else {
                Position::new(
                    tasks_rect.x + 4 + offset,
                    tasks_rect.y + 1 + selected as u16,
                )
            };
            frame.set_cursor_position(position);
        }
    }
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
