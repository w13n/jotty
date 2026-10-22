pub mod memory;

use time::Date;

use thiserror::Error;

use crate::datastore::{self, Controller};

type Result<R> = std::result::Result<R, Error>;

pub trait Model {
    fn set_datastore(&mut self, datastore: Box<dyn Controller>);
    fn exit(&mut self);

    fn new_event(&mut self, date: Date, index: usize, event: Event) -> Result<()>;
    fn new_task(&mut self, date: Date, index: usize, task: Task) -> Result<()>;

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn delete_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn get_event(&self, date: Date, index: usize) -> Result<Event>;
    fn get_task(&self, date: Date, index: usize) -> Result<Task>;

    fn replace_event(&mut self, date: Date, index: usize, event: Event) -> Result<()>;
    fn replace_task(&mut self, date: Date, index: usize, task: Task) -> Result<()>;

    fn swap_event(&mut self, date: Date, index1: usize, index2: usize) -> Result<()>;
    fn swap_task(&mut self, date: Date, index1: usize, index2: usize) -> Result<()>;

    fn events_len(&self, date: Date) -> usize;
    fn tasks_len(&self, date: Date) -> usize;

    fn events_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Event> + 'a>;
    fn tasks_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Task> + 'a>;
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
    Extreme,
}

impl Importance {
    pub fn cycle(self) -> Self {
        match self {
            Importance::Normal => Importance::High,
            Importance::High => Importance::Extreme,
            Importance::Extreme => Importance::Normal,
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("data store error: {0}")]
    Datastore(#[from] datastore::Error),
    #[error("index out of bounds for insertion: index {0} is out of bounds for {1} elements")]
    NewIndexOutOfBounds(usize, usize),
    #[error("index out of bounds for modification: index {0} is out of bounds for {1} elements")]
    ExistingIndexOutOfBounds(usize, usize),
}
