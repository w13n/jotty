mod mem_model;
mod sqlite_model;

pub use mem_model::MemModel;

use anyhow::Result;
use time::Date;

pub trait Model {
    fn new_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn new_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn delete_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn get_event(&mut self, date: Date, index: usize) -> Result<&dyn Event>;
    fn get_task(&self, date: Date, index: usize) -> Result<&dyn Task>;

    fn update_event_title(&mut self, date: Date, index: usize, title: &dyn ToString) -> Result<()>;
    fn update_task_title(&mut self, date: Date, index: usize, title: &dyn ToString) -> Result<()>;

    fn cycle_event(&mut self, date: Date, index: usize) -> Result<()>;
    fn cycle_task(&mut self, date: Date, index: usize) -> Result<()>;

    fn events_len(&self, date: Date) -> usize;
    fn tasks_len(&self, date: Date) -> usize;

    fn events_iter<'a>(
        &'a self,
        date: Date,
    ) -> Box<dyn Iterator<Item = Box<&'a dyn Event<'a>>> + 'a>;
    fn tasks_iter<'a>(&'a self, date: Date)
    -> Box<dyn Iterator<Item = Box<&'a dyn Task<'a>>> + 'a>;
}

pub trait Task<'a> {
    fn title(&'a self) -> &'a str;
    fn completion_level(&self) -> CompletionLevel;
}

pub trait Event<'a> {
    fn title(&'a self) -> &'a str;
    fn importance(&self) -> Importance;
}

#[derive(Copy, Clone, Default)]
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

#[derive(Copy, Clone, Default)]
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
