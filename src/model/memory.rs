use std::collections::HashMap;
use std::iter;
use time::Date;

use super::{Error, Event, Result, Task, datastore};

#[derive(Default)]
pub struct Model(HashMap<Date, Entry>, Option<Box<dyn datastore::Controller>>);

impl Model {
    pub fn new() -> Self {
        Self(HashMap::new(), None)
    }

    fn send(&mut self, msg: datastore::Message) -> Result<()> {
        if let Some(ds) = &mut self.1 {
            ds.send(msg)?;
        }
        Ok(())
    }
}

impl super::Model for Model {
    fn set_datastore(&mut self, datastore: Box<dyn datastore::Controller>) {
        self.1 = Some(datastore);
    }

    fn exit(&mut self) {
        if let Some(controller) = &mut self.1 {
            let _ = controller.send(datastore::Message::Close);
            controller.close();
        }
    }

    fn new_event(&mut self, date: Date, index: usize, event: Event) -> Result<()> {
        let events = &mut self.0.entry(date).or_default().events;

        if index <= events.len() {
            events.insert(index, event.clone());
            Ok(self.send(datastore::Message::NewEvent(event, date, index))?)
        } else {
            Err(Error::NewIndexOutOfBounds(index, events.len()))
        }
    }

    fn new_task(&mut self, date: Date, index: usize, task: Task) -> Result<()> {
        let tasks = &mut self.0.entry(date).or_default().tasks;

        if index <= tasks.len() {
            tasks.insert(index, task.clone());
            Ok(self.send(datastore::Message::NewTask(task, date, index))?)
        } else {
            Err(Error::NewIndexOutOfBounds(index, tasks.len()))
        }
    }

    fn delete_event(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.events.len() {
                entry.events.remove(index);
                Ok(self.send(datastore::Message::DeleteEvent(date, index))?)
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.events.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn delete_task(&mut self, date: Date, index: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.tasks.len() {
                entry.tasks.remove(index);
                Ok(self.send(datastore::Message::DeleteTask(date, index))?)
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.tasks.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn get_event(&self, date: Date, index: usize) -> Result<Event> {
        if let Some(entry) = self.0.get(&date) {
            if index < entry.events.len() {
                Ok(entry
                    .events
                    .get(index)
                    .expect("element is in bounds")
                    .clone())
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.events.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn get_task(&self, date: Date, index: usize) -> Result<Task> {
        if let Some(entry) = self.0.get(&date) {
            if index < entry.tasks.len() {
                Ok(entry
                    .tasks
                    .get(index)
                    .expect("element is in bounds")
                    .clone())
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.tasks.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn replace_event(&mut self, date: Date, index: usize, event: Event) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.events.len() {
                entry.events[index] = event.clone();
                Ok(self.send(datastore::Message::ReplaceEvent(event, date, index))?)
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.events.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn replace_task(&mut self, date: Date, index: usize, task: Task) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index < entry.tasks.len() {
                entry.tasks[index] = task.clone();
                Ok(self.send(datastore::Message::ReplaceTask(task, date, index))?)
            } else {
                Err(Error::ExistingIndexOutOfBounds(index, entry.tasks.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index, 0))
        }
    }

    fn swap_event(&mut self, date: Date, index1: usize, index2: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index1 < entry.events.len() {
                if index2 < entry.events.len() {
                    entry.events.swap(index1, index2);
                    Ok(self.send(datastore::Message::SwapEvent(date, index1, index2))?)
                } else {
                    Err(Error::ExistingIndexOutOfBounds(index2, entry.tasks.len()))
                }
            } else {
                Err(Error::ExistingIndexOutOfBounds(index1, entry.tasks.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index2, 0))
        }
    }

    fn swap_task(&mut self, date: Date, index1: usize, index2: usize) -> Result<()> {
        if let Some(entry) = self.0.get_mut(&date) {
            if index1 < entry.tasks.len() {
                if index2 < entry.tasks.len() {
                    entry.tasks.swap(index1, index2);
                    Ok(self.send(datastore::Message::SwapTask(date, index1, index2))?)
                } else {
                    Err(Error::ExistingIndexOutOfBounds(index2, entry.tasks.len()))
                }
            } else {
                Err(Error::ExistingIndexOutOfBounds(index1, entry.tasks.len()))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index2, 0))
        }
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
}

#[derive(Default)]
struct Entry {
    events: Vec<Event>,
    tasks: Vec<Task>,
}
