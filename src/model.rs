mod journal;

pub use journal::{BuJo, CompletionLevel, Event, Importance, Task, map_bujo::MapBujo};

use ratatui::widgets::ListState;
use time::{Date, OffsetDateTime};

pub struct Model {
    journal: Box<dyn BuJo>,
    date: Date,
    editing: Option<usize>,
    pub events_state: ListState,
    pub task_state: ListState,
    should_exit: bool,
}

impl Model {
    pub fn new(date: Date, journal: Box<dyn BuJo>) -> Self {
        Self {
            journal,
            date,
            editing: None,
            events_state: ListState::default(),
            task_state: ListState::default(),
            should_exit: false,
        }
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    pub fn move_up(&mut self) {
        self.editing = None;
        if self.events_state.selected().is_some() {
            self.events_state.select_previous();
        } else if self.task_state.selected().is_some() {
            self.task_state.select_previous();
        }
    }

    pub fn move_down(&mut self) {
        self.editing = None;
        if self.events_state.selected().is_some() {
            self.events_state.select_next();
        } else if self.task_state.selected().is_some() {
            self.task_state.select_next();
        }
    }

    pub fn move_left(&mut self) {
        self.editing = None;
        if self.task_state.selected().is_some() && self.journal.events_len(self.date) > 0 {
            self.events_state.select(self.task_state.selected());
            self.task_state.select(None);
        }
    }

    pub fn move_right(&mut self) {
        self.editing = None;
        if self.events_state.selected().is_some() && self.journal.tasks_len(self.date) > 0 {
            self.task_state.select(self.events_state.selected());
            self.events_state.select(None);
        }
    }

    pub fn cycle(&mut self) {
        self.editing = None;
        if let Some(idx) = self.task_state.selected() {
            self.journal
                .cycle_task(self.date, idx)
                .expect("selected cannot be out of range");
        } else if let Some(idx) = self.events_state.selected() {
            self.journal
                .cycle_event(self.date, idx)
                .expect("selected cannot be out of range");
        }
    }

    pub fn move_to_next(&mut self) {
        self.move_to(self.date.next_day().expect("we will never reach max date"));
    }

    pub fn move_to_prev(&mut self) {
        self.move_to(
            self.date
                .previous_day()
                .expect("we will never reach minimum date"),
        );
    }

    pub fn move_to_today(&mut self) {
        self.move_to(
            OffsetDateTime::now_local()
                .unwrap_or(OffsetDateTime::now_utc())
                .date(),
        );
    }

    fn move_to(&mut self, date: Date) {
        self.editing = None;
        self.date = date;
        if self.task_state.selected().is_some() && self.journal.tasks_len(date) == 0 {
            if self.journal.events_len(date) > 0 {
                self.events_state.select(self.task_state.selected());
            }
            self.task_state.select(None);
        } else if self.events_state.selected().is_some() && self.journal.events_len(date) == 0 {
            if self.journal.tasks_len(date) > 0 {
                self.task_state.select(self.events_state.selected());
            }
            self.events_state.select(None);
        } else if self.task_state.selected().is_none() && self.task_state.selected().is_none() {
            if self.journal.events_len(date) > 0 {
                self.events_state.select(Some(0));
            } else if self.journal.tasks_len(date) > 0 {
                self.task_state.select(Some(0));
            }
        }
    }

    pub fn enter_editing_mode(&mut self) {
        if let Some(editing_str) = self.get_editing_string() {
            self.editing = Some(editing_str.len());
        }
    }

    pub fn exit_editing_mode(&mut self) {
        self.editing = None;
    }

    pub fn move_cursor_left(&mut self) {
        self.editing = self.editing.map(|x| if x > 0 { x - 1 } else { x });
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(len) = self.get_editing_string().map(str::len) {
            self.editing = self.editing.map(|x| if x < len { x + 1 } else { x });
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(idx) = self.editing() {
            let mut new_str = self
                .get_editing_string()
                .expect("editing has some")
                .to_string();
            new_str.insert(idx, c);
            self.update_editing_string(&new_str);
            self.editing = self.editing.map(|x| x + 1);
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(editing) = self.editing {
            if let Some(str) = self.get_editing_string() {
                let mut new_str = str.to_string();
                new_str.remove(editing - 1);
                if editing > 0 {
                    self.update_editing_string(&new_str);
                    self.editing = self.editing.map(|x| x - 1);
                }
            }
        }
    }

    pub fn append_new_event(&mut self) {
        let idx = self.journal.events_len(self.date);
        self.journal
            .new_event(self.date, idx)
            .expect("idx was set based on length");
        self.events_state.selected_mut().replace(idx);
        self.task_state.selected_mut().take();
        self.editing = Some(0);
    }

    pub fn append_new_task(&mut self) {
        let idx = self.journal.tasks_len(self.date);
        self.journal
            .new_task(self.date, idx)
            .expect("idx was set based on length");
        self.task_state.selected_mut().replace(idx);
        self.events_state.selected_mut().take();
        self.editing = Some(0);
    }

    pub fn insert_new_item(&mut self) {
        if let Some(idx) = self.events_state.selected() {
            self.journal
                .new_event(self.date, idx)
                .expect("idx was set based on selected");
            self.editing = Some(0);
        } else if let Some(idx) = self.task_state.selected() {
            self.journal
                .new_task(self.date, idx)
                .expect("idx was set based on selected");
            self.editing = Some(0);
        }
    }

    pub fn delete(&mut self) {
        self.editing = None;
        if let Some(idx) = self.events_state.selected() {
            self.journal
                .delete_event(self.date, idx)
                .expect("the item is selected");
            if self.journal.events_len(self.date) == 0 && self.journal.tasks_len(self.date) > 0 {
                self.task_state.select(Some(idx));
            }
        } else if let Some(idx) = self.task_state.selected() {
            self.journal
                .delete_task(self.date, idx)
                .expect("the item is selected");
            if self.journal.tasks_len(self.date) == 0 && self.journal.events_len(self.date) > 0 {
                self.events_state.select(Some(idx));
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn editing(&self) -> Option<usize> {
        self.editing
    }

    pub fn date(&self) -> &Date {
        &self.date
    }

    pub fn tasks_len(&self) -> usize {
        self.journal.tasks_len(self.date)
    }

    pub fn events_len(&self) -> usize {
        self.journal.events_len(self.date)
    }

    pub fn tasks_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Box<&'a dyn Task<'a>>> + 'a> {
        self.journal.tasks_iter(self.date)
    }

    pub fn events_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Box<&'a dyn Event<'a>>> + 'a> {
        self.journal.events_iter(self.date)
    }

    fn get_editing_string(&mut self) -> Option<&str> {
        if let Some(row_idx) = self.events_state.selected() {
            return Some(
                self.journal
                    .get_event(self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title(),
            );
        } else if let Some(row_idx) = self.task_state.selected() {
            return Some(
                self.journal
                    .get_task(self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title(),
            );
        }
        None
    }

    fn update_editing_string(&mut self, string: &str) {
        if let Some(row_idx) = self.events_state.selected() {
            self.journal
                .update_event_title(self.date, row_idx, &string)
                .expect("index cannot be out of bounds");
        } else if let Some(row_idx) = self.task_state.selected() {
            self.journal
                .update_task_title(self.date, row_idx, &string)
                .expect("index cannot be out of bounds");
        }
    }
}
