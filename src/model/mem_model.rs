use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::iter;
use time::Date;

use super::{Event, Model, Task};

#[derive(Default)]
pub struct MemModel(HashMap<Date, Entry>);

impl Model for MemModel {
    fn new_event(&mut self, date: Date, index: usize) -> Result<()> {
        let events = &mut self.0.entry(date).or_default().events;

        if index > events.len() {
            return Err(anyhow!("index out of bounds"));
        }

        events.insert(index, Event::default());
        Ok(())
    }

    fn new_task(&mut self, date: Date, index: usize) -> Result<()> {
        let tasks = &mut self.0.entry(date).or_default().tasks;

        if index > tasks.len() {
            return Err(anyhow!("index out of bounds"));
        }

        tasks.insert(index, Task::default());
        Ok(())
    }

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date)
            && index < entry.events.len()
        {
            entry.events.remove(index);
            return Ok(());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn delete_task(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date)
            && index < entry.tasks.len()
        {
            entry.tasks.remove(index);
            return Ok(());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn get_event(&self, date: Date, index: usize) -> Result<Event> {
        if let Some(entry) = self.0.get(&date)
            && index < entry.events.len()
        {
            return Ok(entry
                .events
                .get(index)
                .expect("element is in bounds")
                .clone());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn get_task(&self, date: Date, index: usize) -> Result<Task> {
        if let Some(entry) = self.0.get(&date)
            && index < entry.tasks.len()
        {
            return Ok(entry
                .tasks
                .get(index)
                .expect("element is in bounds")
                .clone());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn replace_event(&mut self, date: Date, index: usize, event: Event) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date)
            && index < entry.events.len()
        {
            entry.events[index] = event;
            return Ok(());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn replace_task(&mut self, date: Date, index: usize, task: Task) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date)
            && index < entry.tasks.len()
        {
            entry.tasks[index] = task;
            return Ok(());
        }

        Err(anyhow!("index out of bounds"))
    }

    fn events_len(&self, date: Date) -> usize {
        self.0
            .get(&date)
            .map(|x| x.events.len())
            .unwrap_or_default()
    }
    fn tasks_len(&self, date: Date) -> usize {
        self.0.get(&date).map(|x| x.tasks.len()).unwrap_or_default()
    }

    fn events_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Event> + 'a> {
        self.0.get(&date).map_or(Box::new(iter::empty()), |x| {
            Box::new(x.events.iter().cloned()) as Box<dyn Iterator<Item = Event> + 'a>
        })
    }

    fn tasks_iter<'a>(&'a self, date: Date) -> Box<dyn Iterator<Item = Task> + 'a> {
        self.0.get(&date).map_or(Box::new(iter::empty()), |x| {
            Box::new(x.tasks.iter().cloned()) as Box<dyn Iterator<Item = Task> + 'a>
        })
    }

    fn err(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct Entry {
    events: Vec<Event>,
    tasks: Vec<Task>,
}
