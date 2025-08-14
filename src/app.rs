use anyhow::{Result, anyhow};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use time::{Date, OffsetDateTime};

pub struct Model {
    pub journal: Journal,
    pub date: Date,
    pub events_state: ListState,
    pub task_state: ListState,
    pub should_exit: bool,
}

impl Model {
    pub fn new(date: Date) -> Self {
        Self {
            journal: Journal::new(),
            date,
            events_state: ListState::default().with_selected(Some(0)),
            task_state: ListState::default(),
            should_exit: false,
        }
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    pub fn up(&mut self) {
        if self.events_state.selected().is_some() {
            self.events_state.select_previous()
        } else {
            self.task_state.select_previous()
        }
    }

    pub fn down(&mut self) {
        if self.events_state.selected().is_some() {
            self.events_state.select_next()
        } else {
            self.task_state.select_next()
        }
    }

    pub fn left(&mut self) {
        if self.task_state.selected().is_some() {
            self.events_state.select(self.task_state.selected());
            self.task_state.select(None);
        }
    }

    pub fn right(&mut self) {
        if self.events_state.selected().is_some() {
            self.task_state.select(self.events_state.selected());
            self.events_state.select(None);
        }
    }

    pub fn cycle_task(&mut self) {
        if let Some(pos) = self.task_state.selected() {
            self.journal
                .cycle_task(&self.date, pos)
                .expect("values extracted from app state so they cannot be invalid")
        }
    }

    pub fn next_day(&mut self) {
        self.date = self.date.next_day().expect("we will never reach max date")
    }

    pub fn prev_day(&mut self) {
        self.date = self
            .date
            .previous_day()
            .expect("we will never reach minimum date")
    }

    pub fn today(&mut self) {
        self.date = OffsetDateTime::now_local()
            .unwrap_or(OffsetDateTime::now_utc())
            .date();
    }

    pub fn new_entry(&mut self) {
        if !self.journal.contains(&self.date) {
            self.journal.insert_with(self.date, Entry::new());
        }
    }
}

#[derive(Default)]
pub struct Journal(pub HashMap<Date, Entry>);

impl Journal {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_with(&mut self, date: Date, entry: Entry) -> bool {
        self.0.insert(date, entry).is_some()
    }

    pub fn contains(&self, date: &Date) -> bool {
        self.0.contains_key(date)
    }

    pub fn cycle_task(&mut self, date: &Date, index: usize) -> Result<()> {
        self.0
            .get_mut(date)
            .ok_or(anyhow!("date does not exist in Journal for cycling"))?
            .cycle_task(index)?;
        Ok(())
    }
}

pub struct Entry {
    pub events: Vec<Event>,
    pub tasks: Vec<Task>,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn push_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn cycle_task(&mut self, index: usize) -> Result<()> {
        self.tasks
            .get_mut(index)
            .ok_or(anyhow!("Task does not exist"))?
            .cycle();
        Ok(())
    }
}

pub struct Event {
    pub title: String,
    pub importance: Importance,
}

impl Event {
    pub fn new(title: &str, importance: Importance) -> Self {
        Self {
            title: title.to_string(),
            importance,
        }
    }
}

pub enum Importance {
    Low,
    Normal,
    High,
    Extreme,
}

pub struct Task {
    pub title: String,
    pub completion_level: CompletionLevel,
}

impl Task {
    pub fn new(title: &str, completion_level: CompletionLevel) -> Self {
        Self {
            title: title.to_string(),
            completion_level,
        }
    }

    pub fn cycle(&mut self) {
        self.completion_level = self.completion_level.cycle()
    }
}

pub enum CompletionLevel {
    None,
    Partial,
    Full,
}

impl CompletionLevel {
    pub fn cycle(&self) -> Self {
        match self {
            CompletionLevel::None => CompletionLevel::Partial,
            CompletionLevel::Partial => CompletionLevel::Full,
            CompletionLevel::Full => CompletionLevel::None,
        }
    }
}
