use std::io::Result;

use anyhow::Error;
use ratatui::DefaultTerminal;
use ratatui::layout::Position;
use ratatui::prelude::*;
use ratatui::text::Span;
use ratatui::text::ToSpan;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, Paragraph, Wrap},
};
use time::{Date, OffsetDateTime};

use crate::model::Model;
use crate::model::{CompletionLevel, Event, Importance, Task};

pub struct View {
    terminal: DefaultTerminal,
    model: Box<dyn Model>,
    date: Date,
    editing: Option<usize>,
    bg_message: Option<String>,
    help_menu: Option<ListState>,
    events_state: ListState,
    task_state: ListState,
}

impl View {
    pub fn new(model: Box<dyn Model>, terminal: DefaultTerminal) -> Self {
        let date = OffsetDateTime::now_local()
            .unwrap_or(OffsetDateTime::now_utc())
            .date();

        let mut events_state = ListState::default();
        let mut task_state = ListState::default();
        if model.events_len(date) > 0 {
            events_state.select(Some(0));
        } else if model.tasks_len(date) > 0 {
            task_state.select(Some(0));
        }

        Self {
            terminal,
            model,
            date,
            bg_message: None,
            help_menu: None,
            editing: None,
            events_state,
            task_state,
        }
    }

    pub fn background_text(mut self, str: String) -> Self {
        self.bg_message = Some(str);
        self
    }

    pub fn render(&mut self) -> Result<()> {
        if let Err(e) = self.model.err() {
            self.render_err(e)
        } else {
            self.render_default()
        }
    }

    fn render_default(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let [_top, middle, _bottom] =
                Layout::vertical([Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)])
                    .flex(Flex::Center)
                    .areas(frame.area());

            let title = Line::from(vec![
                "Jotty".green().bold(),
                " entry on ".bold(),
                self.date.to_string().blue().bold(),
            ]);
            let instructions = Line::from("<q> to quit;  <h> for help".gray());
            let container_block = Block::new()
                .title(title.centered())
                .title_bottom(instructions.centered());

            frame.render_widget(container_block, frame.area());
            if let Some(ls) = &mut self.help_menu {
                View::render_help_frame(frame, middle, ls);
            } else if self.model.events_len(self.date) != 0 || self.model.tasks_len(self.date) != 0
            {
                let [events_rect, tasks_rect] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .areas(middle);
                let events_title = Line::from(" Events ".red().bold());
                let events_block = Block::bordered()
                    .title(events_title.centered())
                    .border_set(border::ROUNDED);

                let events_widget = self
                    .model
                    .events_iter(self.date)
                    .map(|x| ListItem::new(format_events(x)))
                    .collect::<List>()
                    .block(events_block)
                    .highlight_style(Style::new().fg(Color::Red));

                let task_title = Line::from(" Tasks ".yellow().bold());
                let task_block = Block::bordered()
                    .title(task_title.centered())
                    .border_set(border::ROUNDED);
                let task_widget = self
                    .model
                    .tasks_iter(self.date)
                    .map(|x| ListItem::new(format_tasks(x)))
                    .collect::<List>()
                    .block(task_block)
                    .highlight_style(Style::new().fg(Color::Yellow));

                frame.render_stateful_widget(events_widget, events_rect, &mut self.events_state);
                frame.render_stateful_widget(task_widget, tasks_rect, &mut self.task_state);
                if let Some(str_offset) = self.editing {
                    let is_events_side = self.events_state.selected().is_some();
                    let (selected, height_offset) = if is_events_side {
                        (
                            self.events_state.selected().unwrap(),
                            self.events_state.offset(),
                        )
                    } else {
                        (
                            self.task_state.selected().unwrap(),
                            self.task_state.offset(),
                        )
                    };
                    let position = if is_events_side {
                        Position::new(
                            events_rect.x + 1 + str_offset as u16,
                            events_rect.y + 1 + selected as u16 - height_offset as u16,
                        )
                    } else {
                        Position::new(
                            tasks_rect.x + 4 + str_offset as u16,
                            tasks_rect.y + 1 + selected as u16 - height_offset as u16,
                        )
                    };
                    frame.set_cursor_position(position);
                }
            } else {
                let [bg_text_area] =
                    Layout::vertical([Constraint::Length(if self.bg_message.is_none() {
                        1
                    } else {
                        2
                    })])
                    .flex(Flex::Center)
                    .areas(middle);

                let background_text = Paragraph::new(if let Some(msg) = &self.bg_message {
                    Text::from_iter([
                        "no entries or tasks yet today".to_span(),
                        msg.as_str().bold().magenta(),
                    ])
                } else {
                    Text::from("no entries or tasks yet today")
                })
                .centered();
                frame.render_widget(background_text, bg_text_area);
            }
        })?;
        Ok(())
    }

    fn render_help_frame(frame: &mut Frame, area: Rect, ls: &mut ListState) {
        let [help_area] = Layout::vertical([Constraint::Length(12)])
            .flex(Flex::Center)
            .areas(area);
        let [key_area, value_area] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(help_area);
        let key_list = List::from_iter(
            [
                "q",
                "h",
                "e",
                "t",
                "n",
                "\' \'",
                "d",
                "ENTER",
                "ARROW",
                "SHIFT + ARROW",
                "c",
            ]
            .map(|x| {
                Line::from(format!("{x} : "))
                    .bold()
                    .alignment(Alignment::Right)
            }),
        )
        .highlight_style(Style::new().fg(Color::Green));
        let value_list = List::from_iter(
            [
                "quit jotty",
                "toggle this help menu",
                "append a new event",
                "append a new task",
                "insert a new entry above the selected entry",
                "cycle the selected entry",
                "delete an entry",
                "toggle editing mode for the selected entry",
                "move the cursor",
                "move between days",
                "jump to today's page",
            ]
            .map(Line::from),
        )
        .highlight_style(Style::new().fg(Color::Blue));

        frame.render_stateful_widget(key_list, key_area, ls);
        frame.render_stateful_widget(value_list, value_area, ls);
    }

    fn render_err(&mut self, err: Error) -> Result<()> {
        self.terminal.draw(|frame| {
            let title = Line::from(" Jotty Error ".red().bold());
            let container_block = Block::new()
                .title(title.centered())
                .border_set(border::ROUNDED);

            let container = Paragraph::new(err.to_string())
                .centered()
                .block(container_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(container, frame.area());
        })?;
        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.model.err().is_ok() {
            self.editing = None;
            if let Some(ls) = &mut self.help_menu {
                ls.select_previous();
            } else if self.events_state.selected().is_some() {
                self.events_state.select_previous();
            } else if self.task_state.selected().is_some() {
                self.task_state.select_previous();
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.model.err().is_ok() {
            self.editing = None;
            if let Some(ls) = &mut self.help_menu {
                ls.select_next();
            } else if self.events_state.selected().is_some() {
                self.events_state.select_next();
            } else if self.task_state.selected().is_some() {
                self.task_state.select_next();
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = None;
            if self.task_state.selected().is_some() && self.model.events_len(self.date) > 0 {
                self.events_state.select(self.task_state.selected());
                self.task_state.select(None);
            }
        }
    }

    pub fn move_right(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = None;
            if self.events_state.selected().is_some() && self.model.tasks_len(self.date) > 0 {
                self.task_state.select(self.events_state.selected());
                self.events_state.select(None);
            }
        }
    }

    pub fn cycle(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = None;
            if let Some(idx) = self.task_state.selected() {
                let cycled_task = self
                    .model
                    .get_task(self.date, idx)
                    .expect("selected cannot be out of bounds")
                    .cycle();
                self.model
                    .replace_task(self.date, idx, cycled_task)
                    .expect("selected cannot be out of bounds");
            } else if let Some(idx) = self.events_state.selected() {
                let cycled_event = self
                    .model
                    .get_event(self.date, idx)
                    .expect("selected cannot be out of bounds")
                    .cycle();
                self.model
                    .replace_event(self.date, idx, cycled_event)
                    .expect("selected cannot be out of bounds");
            }
        }
    }

    pub fn move_to_next(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.move_to(self.date.next_day().expect("we will never reach max date"));
        }
    }

    pub fn move_to_prev(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.move_to(
                self.date
                    .previous_day()
                    .expect("we will never reach minimum date"),
            );
        }
    }

    pub fn move_to_today(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.move_to(
                OffsetDateTime::now_local()
                    .unwrap_or(OffsetDateTime::now_utc())
                    .date(),
            );
        }
    }

    fn move_to(&mut self, date: Date) {
        self.editing = None;
        self.date = date;
        if self.task_state.selected().is_some() && self.model.tasks_len(date) == 0 {
            if self.model.events_len(date) > 0 {
                self.events_state.select(self.task_state.selected());
            }
            self.task_state.select(None);
        } else if self.events_state.selected().is_some() && self.model.events_len(date) == 0 {
            if self.model.tasks_len(date) > 0 {
                self.task_state.select(self.events_state.selected());
            }
            self.events_state.select(None);
        } else if self.task_state.selected().is_none() && self.task_state.selected().is_none() {
            if self.model.events_len(date) > 0 {
                self.events_state.select(Some(0));
            } else if self.model.tasks_len(date) > 0 {
                self.task_state.select(Some(0));
            }
        }
    }

    pub fn enter_editing_mode(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            if let Some(editing_str) = self.get_editing_string() {
                self.editing = Some(editing_str.len());
            }
        }
    }

    pub fn exit_editing_mode(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = None;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = self.editing.map(|x| if x > 0 { x - 1 } else { x });
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            if let Some(len) = self.get_editing_string().map(|x| x.len()) {
                self.editing = self.editing.map(|x| if x < len { x + 1 } else { x });
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            if let Some(idx) = self.editing {
                let mut new_str = self
                    .get_editing_string()
                    .expect("editing has some")
                    .to_string();
                new_str.insert(idx, c);
                self.update_editing_string(new_str);
                self.editing = self.editing.map(|x| x + 1);
            }
        }
    }

    pub fn delete_char(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            if let Some(editing) = self.editing
                && let Some(str) = self.get_editing_string()
            {
                let mut new_str = str.to_string();
                new_str.remove(editing - 1);
                if editing > 0 {
                    self.update_editing_string(new_str);
                    self.editing = self.editing.map(|x| x - 1);
                }
            }
        }
    }

    pub fn append_new_event(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            let idx = self.model.events_len(self.date);
            self.model
                .new_event(self.date, idx)
                .expect("idx was set based on length");
            self.events_state.selected_mut().replace(idx);
            self.task_state.selected_mut().take();
            self.editing = Some(0);
        }
    }

    pub fn append_new_task(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            let idx = self.model.tasks_len(self.date);
            self.model
                .new_task(self.date, idx)
                .expect("idx was set based on length");
            self.task_state.selected_mut().replace(idx);
            self.events_state.selected_mut().take();
            self.editing = Some(0);
        }
    }

    pub fn insert_new_item(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            if let Some(idx) = self.events_state.selected() {
                self.model
                    .new_event(self.date, idx)
                    .expect("idx was set based on selected");
                self.editing = Some(0);
            } else if let Some(idx) = self.task_state.selected() {
                self.model
                    .new_task(self.date, idx)
                    .expect("idx was set based on selected");
                self.editing = Some(0);
            }
        }
    }

    pub fn delete(&mut self) {
        if self.model.err().is_ok() && self.help_menu.is_none() {
            self.editing = None;
            if let Some(idx) = self.events_state.selected() {
                self.model
                    .delete_event(self.date, idx)
                    .expect("the item is selected");
                if self.model.events_len(self.date) == 0 {
                    self.events_state.select(None);
                    if self.model.tasks_len(self.date) > 0 {
                        self.task_state.select(Some(idx));
                    }
                }
            } else if let Some(idx) = self.task_state.selected() {
                self.model
                    .delete_task(self.date, idx)
                    .expect("the item is selected");
                if self.model.tasks_len(self.date) == 0 {
                    self.task_state.select(None);
                    if self.model.events_len(self.date) > 0 {
                        self.events_state.select(Some(idx));
                    }
                }
            }
        }
    }

    pub fn is_editing(&self) -> bool {
        if self.model.err().is_err() {
            return false;
        }
        self.editing.is_some()
    }

    fn get_editing_string(&mut self) -> Option<String> {
        if let Some(row_idx) = self.events_state.selected() {
            return Some(
                self.model
                    .get_event(self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title,
            );
        } else if let Some(row_idx) = self.task_state.selected() {
            return Some(
                self.model
                    .get_task(self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title,
            );
        }
        None
    }

    fn update_editing_string(&mut self, string: String) {
        if let Some(idx) = self.task_state.selected() {
            let mut new_task = self
                .model
                .get_task(self.date, idx)
                .expect("selected cannot be out of bounds");
            new_task.title = string;
            self.model
                .replace_task(self.date, idx, new_task)
                .expect("selected cannot be out of bounds");
        } else if let Some(idx) = self.events_state.selected() {
            let mut new_event = self
                .model
                .get_event(self.date, idx)
                .expect("selected cannot be out of bounds");
            new_event.title = string;
            self.model
                .replace_event(self.date, idx, new_event)
                .expect("selected cannot be out of bounds");
        }
    }

    pub fn toggle_help(&mut self) {
        if self.help_menu.is_some() {
            self.help_menu = None;
        } else {
            self.help_menu = Some(ListState::default().with_selected(Some(0)));
        }
    }
}

fn format_tasks(task: Task) -> String {
    match task.completion_level {
        CompletionLevel::None => {
            format!(" ○ {}", task.title)
        }
        CompletionLevel::Partial => {
            format!(" ◐ {}", task.title)
        }
        CompletionLevel::Full => {
            format!(" ● {}", task.title)
        }
    }
}

fn format_events(event: Event) -> Span<'static> {
    match event.importance {
        Importance::Normal => Span::from(event.title),
        Importance::High => event.title.bold(),
    }
}
