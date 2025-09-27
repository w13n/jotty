mod mem_model;
mod sqlite_model;

pub use mem_model::MemModel;
pub use sqlite_model::SqliteModel;

use anyhow::Result;
use time::Date;

pub trait Model {
    fn new_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn new_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn delete_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn get_event(&self, date: Date, index: usize) -> Result<Event>;
    fn get_task(&self, date: Date, index: usize) -> Result<Task>;

    fn replace_event(&mut self, date: Date, index: usize, event: Event) -> Result<()>;
    fn replace_task(&mut self, date: Date, index: usize, task: Task) -> Result<()>;

    fn events_len(&self, date: Date) -> usize;
    fn tasks_len(&self, date: Date) -> usize;

    fn events_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Event> + 'a>;
    fn tasks_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Task> + 'a>;

    fn err(&self) -> Result<()>;
}

#[derive(Default, Debug, Hash, Clone)]
pub struct Task {
    pub title: String,
    pub completion_level: CompletionLevel,
}

impl Task {
    pub fn cycle(self) -> Self {
        Self {
            title: self.title,
            completion_level: self.completion_level.cycle(),
        }
    }
}

#[derive(Default, Debug, Hash, Clone)]
pub struct Event {
    pub title: String,
    pub importance: Importance,
}

impl Event {
    pub fn cycle(self) -> Self {
        Self {
            title: self.title,
            importance: self.importance.cycle(),
        }
    }
}

#[derive(Default, Debug, Hash, Clone)]
pub enum Importance {
    #[default]
    Normal,
    High,
}

impl Importance {
    pub fn cycle(self) -> Self {
        match self {
            Importance::High => Importance::Normal,
            Importance::Normal => Importance::High,
        }
    }
}

#[derive(Default, Debug, Hash, Clone)]
pub enum CompletionLevel {
    #[default]
    None,
    Partial,
    Full,
}

impl CompletionLevel {
    pub fn cycle(self) -> Self {
        match self {
            CompletionLevel::None => CompletionLevel::Partial,
            CompletionLevel::Partial => CompletionLevel::Full,
            CompletionLevel::Full => CompletionLevel::None,
        }
    }
}
