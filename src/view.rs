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

use crate::model::{CompletionLevel, Event, Importance, Task};

pub fn view(model: &mut crate::Model, frame: &mut Frame) {
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

    if model.events_len() != 0 || model.tasks_len() != 0 {
        let events_title = Line::from(" Events ".red().bold());
        let events_block = Block::bordered()
            .title(events_title.centered())
            .border_set(border::ROUNDED);

        let events_widget = model
            .events_iter()
            .map(|x| ListItem::new(format_events(*x)))
            .collect::<List>()
            .block(events_block)
            .highlight_style(Style::new().fg(Color::Red));

        let task_title = Line::from(" Tasks ".yellow().bold());
        let task_block = Block::bordered()
            .title(task_title.centered())
            .border_set(border::ROUNDED);
        let task_widget = model
            .tasks_iter()
            .map(|x| ListItem::new(format_tasks(*x)))
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

fn format_tasks<'a>(task: &'a (dyn Task<'a> + 'a)) -> String {
    match task.completion_level() {
        CompletionLevel::None => {
            format!(" ○ {}", &task.title())
        }
        CompletionLevel::Partial => {
            format!(" ◐ {}", &task.title())
        }
        CompletionLevel::Full => {
            format!(" ● {}", &task.title())
        }
    }
}

fn format_events<'a>(event: &'a (dyn Event<'a> + 'a)) -> Span<'static> {
    match event.importance() {
        Importance::Normal => Span::from(event.title().to_owned()),
        Importance::High => event.title().to_owned().clone().bold(),
    }
}
