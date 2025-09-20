use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::iter;
use time::Date;

use super::{BuJo, CompletionLevel, Event, Importance, Task};

#[derive(Default)]
pub struct MapBujo(HashMap<Date, Entry>);

impl BuJo for MapBujo {
    fn new_event(&mut self, date: Date, index: usize) -> Result<()> {
        let events = &mut self.0.entry(date).or_default().events;

        if index > events.len() {
            return Err(anyhow!("index out of bounds"));
        }

        events.insert(index, MemEvent::default());
        Ok(())
    }

    fn new_task(&mut self, date: Date, index: usize) -> Result<()> {
        let tasks = &mut self.0.entry(date).or_default().tasks;

        if index > tasks.len() {
            return Err(anyhow!("index out of bounds"));
        }

        tasks.insert(index, MemTask::default());
        Ok(())
    }

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.events.len() {
                entry.events.remove(index);
                return Ok(());
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn delete_task(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.tasks.len() {
                entry.tasks.remove(index);
                return Ok(());
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn get_event(&mut self, date: Date, index: usize) -> Result<&dyn Event> {
        if let Some(entry) = self.0.get(&date) {
            if index < entry.events.len() {
                return Ok(entry.events.get(index).expect("element is in bounds"));
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn get_task(&self, date: Date, index: usize) -> Result<&dyn Task> {
        if let Some(entry) = self.0.get(&date) {
            if index < entry.tasks.len() {
                return Ok(entry.tasks.get(index).expect("element is in bounds"));
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn update_event_title(&mut self, date: Date, index: usize, title: &dyn ToString) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.events.len() {
                entry
                    .events
                    .get_mut(index)
                    .expect("index is in bounds")
                    .title = title.to_string();
                return Ok(());
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn update_task_title(&mut self, date: Date, index: usize, title: &dyn ToString) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.tasks.len() {
                entry
                    .tasks
                    .get_mut(index)
                    .expect("index is in bounds")
                    .title = title.to_string();
                return Ok(());
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn cycle_event(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.events.len() {
                entry
                    .events
                    .get_mut(index)
                    .expect("index is in bounds")
                    .cycle();
                return Ok(());
            }
        }

        Err(anyhow!("index out of bounds"))
    }

    fn cycle_task(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.tasks.len() {
                entry
                    .tasks
                    .get_mut(index)
                    .expect("index is in bounds")
                    .cycle();
                return Ok(());
            }
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

    fn events_iter<'a>(
        &'a self,
        date: Date,
    ) -> Box<dyn Iterator<Item = Box<&'a dyn Event<'a>>> + 'a> {
        self.0.get(&date).map_or(Box::new(iter::empty()), |x| {
            Box::new(x.events.iter().map(|e| Box::new(e as &'a dyn Event)))
                as Box<dyn Iterator<Item = Box<&dyn Event>>>
        })
    }

    fn tasks_iter<'a>(
        &'a self,
        date: Date,
    ) -> Box<dyn Iterator<Item = Box<&'a dyn Task<'a>>> + 'a> {
        self.0.get(&date).map_or(Box::new(iter::empty()), |x| {
            Box::new(x.tasks.iter().map(|t| Box::new(t as &dyn Task)))
                as Box<dyn Iterator<Item = Box<&dyn Task>>>
        })
    }
}

#[derive(Default)]
struct Entry {
    events: Vec<MemEvent>,
    tasks: Vec<MemTask>,
}

#[derive(Default)]
pub struct MemEvent {
    title: String,
    importance: Importance,
}

impl MemEvent {
    fn cycle(&mut self) {
        self.importance = self.importance.cycle();
    }
}

impl<'a> Event<'a> for MemEvent {
    fn title(&'a self) -> &'a str {
        self.title.as_str()
    }

    fn importance(&self) -> Importance {
        self.importance
    }
}

#[derive(Default)]
struct MemTask {
    title: String,
    completion_level: CompletionLevel,
}

impl MemTask {
    fn cycle(&mut self) {
        self.completion_level = self.completion_level.cycle();
    }
}

impl<'a> Task<'a> for MemTask {
    fn title(&'a self) -> &'a str {
        self.title.as_str()
    }

    fn completion_level(&self) -> CompletionLevel {
        self.completion_level
    }
}
