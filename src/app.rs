use crate::journal::{Event, Journal, Task};
use ratatui::widgets::ListState;
use time::{Date, OffsetDateTime};

pub struct Model {
    journal: Journal,
    date: Date,
    editing: Option<u16>,
    pub events_state: ListState,
    pub task_state: ListState,
    should_exit: bool,
}

impl Model {
    pub fn new(date: Date) -> Self {
        Self {
            journal: Journal::new(),
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
            self.events_state.select_previous()
        } else if self.task_state.selected().is_some() {
            self.task_state.select_previous()
        }
    }

    pub fn move_down(&mut self) {
        self.editing = None;
        if self.events_state.selected().is_some() {
            self.events_state.select_next()
        } else if self.task_state.selected().is_some() {
            self.task_state.select_next()
        }
    }

    pub fn move_left(&mut self) {
        self.editing = None;
        if self.task_state.selected().is_some()
            && self
                .journal
                .events_len(&self.date)
                .expect("task state has something selected")
                > 0
        {
            self.events_state.select(self.task_state.selected());
            self.task_state.select(None);
        }
    }

    pub fn move_right(&mut self) {
        self.editing = None;
        if self.events_state.selected().is_some()
            && self
                .journal
                .tasks_len(&self.date)
                .expect("event state has something selected")
                > 0
        {
            self.task_state.select(self.events_state.selected());
            self.events_state.select(None);
        }
    }

    pub fn cycle(&mut self) {
        self.editing = None;
        if let Some(idx) = self.task_state.selected() {
            self.journal
                .get_task_mut(&self.date, idx)
                .expect("selected cannot be out of range")
                .cycle();
        } else if let Some(idx) = self.events_state.selected() {
            self.journal
                .get_event_mut(&self.date, idx)
                .expect("selected cannot be out of range")
                .cycle();
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
        if self.task_state.selected().is_some() && self.journal.tasks_len(&date).unwrap_or(0) == 0 {
            if self.journal.events_len(&date).unwrap_or(0) > 0 {
                self.events_state.select(self.task_state.selected());
            }
            self.task_state.select(None);
        } else if self.events_state.selected().is_some()
            && self.journal.events_len(&date).unwrap_or(0) == 0
        {
            if self.journal.tasks_len(&date).unwrap_or(0) > 0 {
                self.task_state.select(self.events_state.selected());
            }
            self.events_state.select(None);
        } else if self.task_state.selected().is_none() && self.task_state.selected().is_none() {
            if self.journal.events_len(&date).unwrap_or(0) > 0 {
                self.events_state.select(Some(0));
            } else if self.journal.tasks_len(&date).unwrap_or(0) > 0 {
                self.task_state.select(Some(0));
            }
        }
    }

    pub fn create_new_entry(&mut self) {
        self.editing = None;
        if !self.journal.contains_day(&self.date) {
            self.journal.new_entry(self.date);
        }
    }

    pub fn enter_editing_mode(&mut self) {
        if let Some(editing_str) = self.get_editing_string() {
            self.editing = Some(editing_str.len() as u16)
        }
    }

    pub fn exit_editing_mode(&mut self) {
        self.editing = None;
    }

    pub fn move_cursor_left(&mut self) {
        self.editing = self.editing.map(|x| if x > 0 { x - 1 } else { x });
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(len) = self.get_editing_string().map(|x| x.len() as u16) {
            self.editing = self.editing.map(|x| if x < len { x + 1 } else { x });
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(idx) = self.editing() {
            let str = self.get_editing_string().expect("editing has some");
            str.insert(idx as usize, c);
            self.editing = self.editing.map(|x| x + 1);
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(editing) = self.editing {
            if let Some(x) = self.get_editing_string() {
                if editing > 0 {
                    x.remove(editing as usize - 1);
                    self.editing = self.editing.map(|x| x - 1);
                }
            }
        }
    }

    pub fn append_new_event(&mut self) {
        if self.has_entry() {
            let idx = self.journal.events_len(&self.date).expect("self has entry");
            self.journal
                .new_event(&self.date, idx)
                .expect("idx was set based on length");
            self.events_state.selected_mut().replace(idx);
            self.task_state.selected_mut().take();
            self.editing = Some(0);
        }
    }

    pub fn append_new_task(&mut self) {
        if self.has_entry() {
            let idx = self.journal.tasks_len(&self.date).expect("self has entry");
            self.journal
                .new_task(&self.date, idx)
                .expect("idx was set based on length");
            self.task_state.selected_mut().replace(idx);
            self.events_state.selected_mut().take();
            self.editing = Some(0);
        }
    }

    pub fn insert_new_item(&mut self) {
        if self.has_entry() {
            if let Some(idx) = self.events_state.selected() {
                self.journal
                    .new_event(&self.date, idx)
                    .expect("idx was set based on selected");
                self.editing = Some(0);
            } else if let Some(idx) = self.task_state.selected() {
                self.journal
                    .new_task(&self.date, idx)
                    .expect("idx was set based on selected");
                self.editing = Some(0);
            }
        }
    }

    pub fn delete(&mut self) {
        self.editing = None;
        if let Some(idx) = self.events_state.selected() {
            self.journal
                .delete_event(&self.date, idx)
                .expect("the item is selected");
            if self
                .journal
                .events_len(&self.date)
                .expect("an item was selected")
                == 0
                && self
                    .journal
                    .tasks_len(&self.date)
                    .expect("an item was selected")
                    > 0
            {
                self.task_state.select(Some(idx));
            }
        } else if let Some(idx) = self.task_state.selected() {
            self.journal
                .delete_task(&self.date, idx)
                .expect("the item is selected");
            if self
                .journal
                .tasks_len(&self.date)
                .expect("an item was selected")
                == 0
                && self
                    .journal
                    .events_len(&self.date)
                    .expect("an item was selected")
                    > 0
            {
                self.events_state.select(Some(idx));
            }
        } else if self.has_entry() {
            self.journal
                .delete_entry(&self.date)
                .expect("we have the entry");
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn editing(&self) -> Option<u16> {
        self.editing
    }

    pub fn date(&self) -> &Date {
        &self.date
    }

    pub fn has_entry(&self) -> bool {
        self.journal.contains_day(&self.date)
    }

    pub fn tasks_iter(&self) -> Option<std::slice::Iter<'_, Task>> {
        self.journal.tasks_iter(&self.date)
    }

    pub fn events_iter(&self) -> Option<std::slice::Iter<'_, Event>> {
        self.journal.events_iter(&self.date)
    }

    fn get_editing_string(&mut self) -> Option<&mut String> {
        if let Some(row_idx) = self.events_state.selected() {
            return Some(
                &mut self
                    .journal
                    .get_event_mut(&self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title,
            );
        } else if let Some(row_idx) = self.task_state.selected() {
            return Some(
                &mut self
                    .journal
                    .get_task_mut(&self.date, row_idx)
                    .expect("selected cannot be out of range")
                    .title,
            );
        }
        None
    }
}
