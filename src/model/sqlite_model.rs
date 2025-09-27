use std::{
    cell::{Cell, RefCell},
    iter,
};

use anyhow::{Result, anyhow};
use diesel::prelude::*;
use time::Date;

use super::Importance;
use crate::model::{CompletionLevel, Event, Model, Task};

pub struct SqliteModel(RefCell<SqliteConnection>, Cell<bool>);

impl SqliteModel {
    pub fn new(sqlite_connection: SqliteConnection) -> Self {
        Self(RefCell::new(sqlite_connection), Cell::new(false))
    }
}

impl Model for SqliteModel {
    fn new_event(&mut self, d: Date, i: usize) -> Result<()> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

        if i <= len {
            diesel::update(events)
                .filter(
                    date.eq(julian_date)
                        .and(index.eq(i as i32).or(index.gt(i as i32))),
                )
                .set(index.eq(index + 1))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            let new_event = SQLEvent::new(julian_date, i as i32);

            diesel::insert_into(events)
                .values(&new_event)
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
        }

        Ok(())
    }

    fn new_task(&mut self, d: Date, i: usize) -> Result<()> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d);

        if i <= len {
            diesel::update(tasks)
                .filter(
                    date.eq(julian_date)
                        .and(index.eq(i as i32).or(index.gt(i as i32))),
                )
                .set(index.eq(index + 1))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            let new_task = SQLTask::new(julian_date, i as i32);

            diesel::insert_into(tasks)
                .values(&new_task)
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
            Ok(())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn delete_event(&mut self, d: Date, i: usize) -> Result<()> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

        if i < len {
            diesel::delete(events)
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            diesel::update(events)
                .filter(date.eq(julian_date).and(index.gt(i as i32)))
                .set(index.eq(index - 1))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
            Ok(())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn delete_task(&mut self, d: Date, i: usize) -> Result<()> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d);

        if i < len {
            diesel::delete(tasks)
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            diesel::update(tasks)
                .filter(date.eq(julian_date).and(index.gt(i as i32)))
                .set(index.eq(index - 1))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
            Ok(())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn get_event(&self, d: Date, i: usize) -> Result<Event> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

        if i < len {
            let elem = events
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .select(SQLEvent::as_select())
                .first(&mut *self.0.borrow_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    SQLEvent::new(julian_date, i as i32)
                });
            Ok(elem.to())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn get_task(&self, d: Date, i: usize) -> Result<Task> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d);

        if i < len {
            let elem = tasks
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .select(SQLTask::as_select())
                .first(&mut *self.0.borrow_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    SQLTask::new(julian_date, i as i32)
                });
            Ok(elem.to())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn replace_event(&mut self, d: Date, i: usize, e: Event) -> Result<()> {
        use tables::events::dsl::*;
        let event = SQLEvent::from(e, d, i);
        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

        if i < len {
            diesel::delete(events)
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            diesel::insert_into(events)
                .values(&event)
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
            Ok(())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn replace_task(&mut self, d: Date, i: usize, t: Task) -> Result<()> {
        use tables::tasks::dsl::*;
        let task = SQLTask::from(t, d, i);
        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d);

        if i < len {
            diesel::delete(tasks)
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });

            diesel::insert_into(tasks)
                .values(&task)
                .execute(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    0
                });
            Ok(())
        } else {
            Err(anyhow!("index out of bounds"))
        }
    }

    fn tasks_len(&self, d: Date) -> usize {
        use tables::tasks::dsl::*;

        let result: Result<i64, _> = tasks
            .filter(date.eq(d.to_julian_day()))
            .count()
            .get_result(&mut *self.0.borrow_mut());
        if let Ok(len) = result {
            return len as usize;
        };

        self.1.set(true);
        0
    }

    fn events_len(&self, d: Date) -> usize {
        use tables::events::dsl::*;

        let result: Result<i64, _> = events
            .filter(date.eq(d.to_julian_day()))
            .count()
            .get_result(&mut *self.0.borrow_mut());
        if let Ok(len) = result {
            return len as usize;
        };

        self.1.set(true);
        0
    }

    fn events_iter<'a>(&'a self, d: Date) -> Box<dyn Iterator<Item = Event> + 'a> {
        use tables::events::dsl::*;
        events
            .filter(date.eq(d.to_julian_day()))
            .select(SQLEvent::as_select())
            .order(index.asc())
            .load(&mut *self.0.borrow_mut())
            .map(|vec| {
                Box::new(vec.into_iter().map(|elem| elem.to()))
                    as Box<dyn Iterator<Item = Event> + 'a>
            })
            .unwrap_or_else(|_| {
                self.1.set(true);
                Box::new(iter::empty())
            })
    }

    fn tasks_iter<'a>(&'a self, d: Date) -> Box<dyn Iterator<Item = Task> + 'a> {
        use tables::tasks::dsl::*;
        tasks
            .filter(date.eq(d.to_julian_day()))
            .select(SQLTask::as_select())
            .order(index.asc())
            .load(&mut *self.0.borrow_mut())
            .map(|vec| {
                Box::new(vec.into_iter().map(|elem| elem.to()))
                    as Box<dyn Iterator<Item = Task> + 'a>
            })
            .unwrap_or_else(|_| {
                self.1.set(true);
                Box::new(iter::empty())
            })
    }

    fn err(&self) -> Result<()> {
        if self.1.get() {
            return Err(anyhow!(
                "The database has encountered an unrecoverable error. This can occur when the database is deleted or the permissions are changed while this program is running. You must quit the app now."
            ));
        }
        Ok(())
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = tables::events)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct SQLEvent {
    date: i32,
    index: i32,
    title: String,
    importance: i32,
}

impl SQLEvent {
    fn new(date: i32, index: i32) -> Self {
        Self {
            date,
            index,
            title: String::new(),
            importance: 0,
        }
    }

    fn to(self) -> Event {
        Event {
            title: self.title,
            importance: match self.importance {
                0 => Importance::Normal,
                1 => Importance::High,
                _ => panic!("db out of sync"),
            },
        }
    }

    fn from(e: Event, d: Date, i: usize) -> Self {
        let importance = match e.importance {
            Importance::Normal => 0,
            Importance::High => 1,
        };

        Self {
            title: e.title,
            importance,
            date: d.to_julian_day(),
            index: i as i32,
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = tables::tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct SQLTask {
    date: i32,
    index: i32,
    title: String,
    completion_level: i32,
}

impl SQLTask {
    fn new(date: i32, index: i32) -> Self {
        Self {
            date,
            index,
            title: String::new(),
            completion_level: 0,
        }
    }

    fn to(self) -> Task {
        Task {
            title: self.title,
            completion_level: match self.completion_level {
                0 => CompletionLevel::None,
                1 => CompletionLevel::Partial,
                2 => CompletionLevel::Full,
                _ => panic!("db out of sync"),
            },
        }
    }

    fn from(e: Task, d: Date, i: usize) -> Self {
        let completion_level = match e.completion_level {
            CompletionLevel::None => 0,
            CompletionLevel::Partial => 1,
            CompletionLevel::Full => 2,
        };

        Self {
            title: e.title,
            completion_level,
            date: d.to_julian_day(),
            index: i as i32,
        }
    }
}

mod tables {
    diesel::table! {
        events (date, index) {
            date -> Integer,
            index -> Integer,
            title -> Text,
            importance -> Integer,
        }
    }

    diesel::table! {
        tasks (date, index) {
            date -> Integer,
            index -> Integer,
            title -> Text,
            completion_level -> Integer,
        }
    }
}
