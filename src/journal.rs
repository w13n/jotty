use anyhow::{Result, anyhow};
use std::collections::HashMap;
use time::Date;

#[derive(Default)]
pub struct Journal(HashMap<Date, Entry>);

impl Journal {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn new_entry(&mut self, date: Date) -> bool {
        self.0.insert(date, Entry::new()).is_some()
    }

    pub fn new_task(&mut self, date: &Date, index: usize) -> Result<()> {
        let tasks = &mut self
            .0
            .get_mut(date)
            .ok_or(anyhow!("date does not exist in journal"))?
            .tasks;

        if tasks.len() < index {
            return Err(anyhow!("index out of bounds"));
        }

        tasks.insert(index, Task::new());
        Ok(())
    }

    pub fn new_event(&mut self, date: &Date, index: usize) -> Result<()> {
        let events = &mut self
            .0
            .get_mut(date)
            .ok_or(anyhow!("date does not exist in journal"))?
            .events;

        if events.len() < index {
            return Err(anyhow!("index out of bounds"));
        }

        events.insert(index, Event::new());
        Ok(())
    }

    pub fn contains_day(&self, date: &Date) -> bool {
        self.0.contains_key(date)
    }

    pub fn get_task(&mut self, date: &Date, index: usize) -> Option<&Task> {
        if let Some(entry) = self.0.get(date) {
            return entry.tasks.get(index);
        }
        None
    }

    pub fn get_task_mut(&mut self, date: &Date, index: usize) -> Option<&mut Task> {
        if let Some(entry) = self.0.get_mut(date) {
            return entry.tasks.get_mut(index);
        }
        None
    }

    pub fn get_event(&mut self, date: &Date, index: usize) -> Option<&Event> {
        if let Some(entry) = self.0.get(date) {
            return entry.events.get(index);
        }
        None
    }

    pub fn get_event_mut(&mut self, date: &Date, index: usize) -> Option<&mut Event> {
        if let Some(entry) = self.0.get_mut(date) {
            return entry.events.get_mut(index);
        }
        None
    }

    pub fn tasks_len(&self, date: &Date) -> Option<usize> {
        self.0.get(date).map(|x| x.tasks.len())
    }

    pub fn events_len(&self, date: &Date) -> Option<usize> {
        self.0.get(date).map(|x| x.events.len())
    }

    pub fn tasks_iter(&self, date: &Date) -> Option<std::slice::Iter<'_, Task>> {
        self.0.get(date).map(|x| x.tasks.iter())
    }

    pub fn events_iter(&self, date: &Date) -> Option<std::slice::Iter<'_, Event>> {
        self.0.get(date).map(|x| x.events.iter())
    }
}

struct Entry {
    events: Vec<Event>,
    tasks: Vec<Task>,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            tasks: Vec::new(),
        }
    }
}

pub struct Event {
    pub title: String,
    pub importance: Importance,
}

impl Event {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            importance: Importance::Normal,
        }
    }

    pub fn cycle(&mut self) {
        self.importance = self.importance.cycle()
    }
}

pub enum Importance {
    Normal,
    High,
}

impl Importance {
    fn cycle(&self) -> Self {
        match self {
            Importance::High => Importance::Normal,
            Importance::Normal => Importance::High,
        }
    }
}

pub struct Task {
    pub title: String,
    pub completion_level: CompletionLevel,
}

impl Task {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            completion_level: CompletionLevel::None,
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
    fn cycle(&self) -> Self {
        match self {
            CompletionLevel::None => CompletionLevel::Partial,
            CompletionLevel::Partial => CompletionLevel::Full,
            CompletionLevel::Full => CompletionLevel::None,
        }
    }
}
