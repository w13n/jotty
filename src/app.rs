use std::collections::HashMap;
use time::{Date, OffsetDateTime};

#[derive(Default)]
pub struct Model {
    pub journal: Journal,
    pub should_exit: bool,
}

impl Model {
    pub fn new() -> Self {
        Self {
            journal: Journal::new(),
            should_exit: false,
        }
    }
}

#[derive(Default)]
pub struct Journal(pub HashMap<Date, Entry>);

impl Journal {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_today(&mut self, entry: Entry) {
        let today = OffsetDateTime::now_local()
            .unwrap_or(OffsetDateTime::now_utc())
            .date();
        self.0.insert(today, entry);
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
}

pub enum CompletionLevel {
    None,
    Partial,
    Full,
}
