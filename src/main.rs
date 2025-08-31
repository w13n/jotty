use std::io;

use crate::app::Model;
use crate::journal::{CompletionLevel, Event, Importance, Task};
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Position;
use ratatui::prelude::Color;
use ratatui::text::Span;
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
use time::{Date, Month, OffsetDateTime};

mod app;
mod journal;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let date = OffsetDateTime::now_local()
        .unwrap_or(OffsetDateTime::now_utc())
        .date();
    let mut model = Model::new(date);

    while !model.should_exit() {
        terminal.draw(|frame| view(&mut model, frame))?;
        update(&mut model)?;
    }
    ratatui::restore();
    Ok(())
}

fn update(app: &mut Model) -> io::Result<()> {
    match event::read()? {
        event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Left => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        app.move_to_prev();
                    } else if app.editing().is_some() {
                        app.move_cursor_left();
                    } else {
                        app.move_left();
                    }
                }
                KeyCode::Right => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        app.move_to_next();
                    } else if app.editing().is_some() {
                        app.move_cursor_right();
                    } else {
                        app.move_right();
                    }
                }
                KeyCode::Backspace => app.delete_char(),
                KeyCode::Enter => {
                    if app.editing().is_some() {
                        app.exit_editing_mode();
                    } else {
                        app.enter_editing_mode();
                    }
                }
                KeyCode::Esc => app.exit_editing_mode(),
                KeyCode::Char(c) => {
                    if app.editing().is_some() {
                        app.insert_char(c);
                    } else {
                        match c {
                            'q' => app.exit(),
                            ' ' => app.cycle(),
                            'c' => app.move_to_today(),
                            'n' => {
                                if app.has_entry() {
                                    app.insert_new_item();
                                } else {
                                    app.create_new_entry();
                                }
                            }
                            'e' => app.append_new_event(),
                            't' => app.append_new_task(),
                            'd' => app.delete(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
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
        model.date().to_string().blue().bold(),
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

    if model.has_entry() {
        let events_title = Line::from(" Events ".red().bold());
        let events_block = Block::bordered()
            .title(events_title.centered())
            .border_set(border::ROUNDED);
        let events_items = model
            .events_iter()
            .expect("checked that model has entry")
            .map(|x| ListItem::new(format_events(x)));
        let events_widget = events_items
            .collect::<List>()
            .block(events_block)
            .highlight_style(Style::new().fg(Color::Red));

        let task_title = Line::from(" Tasks ".yellow().bold());
        let task_block = Block::bordered()
            .title(task_title.centered())
            .border_set(border::ROUNDED);
        let task_items = model
            .tasks_iter()
            .expect("checked that model has entry")
            .map(|x| ListItem::new(format_tasks(x)));
        let task_widget = task_items
            .collect::<List>()
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
                    events_rect.x + 1 + offset as u16,
                    events_rect.y + 1 + selected as u16,
                )
            } else {
                Position::new(
                    tasks_rect.x + 4 + offset as u16,
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

fn format_events(event: &Event) -> Span<'static> {
    match event.importance {
        Importance::Normal => Span::from(event.title.clone()),
        Importance::High => event.title.clone().bold(),
    }
}
