use std::{
    cell::{Cell, RefCell},
    i64,
};

use anyhow::anyhow;
use diesel::prelude::*;
use time::Date;

use super::Importance;
use crate::model::{CompletionLevel, Event, Model, Task};

struct SqliteModel(RefCell<SqliteConnection>, Cell<bool>);

impl SqliteModel {
    fn new(sqlite_connection: SqliteConnection) -> Self {
        Self(RefCell::new(sqlite_connection), Cell::new(false))
    }
}

impl Model for SqliteModel {
    fn new_event(&mut self, d: Date, i: usize) -> anyhow::Result<()> {
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

    fn new_task(&mut self, d: Date, i: usize) -> anyhow::Result<()> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

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

    fn delete_event(&mut self, d: Date, i: usize) -> anyhow::Result<()> {
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

    fn delete_task(&mut self, d: Date, i: usize) -> anyhow::Result<()> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

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

    fn get_event(&mut self, d: Date, i: usize) -> anyhow::Result<&dyn Event> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d);

        if i < len {
            let elem = events
                .filter(date.eq(julian_date).and(index.eq(i as i32)))
                .select(SQLEvent::as_select())
                .first(self.0.get_mut())
                .unwrap_or_else(|_| {
                    self.1.set(true);
                    SQLEvent::new(julian_date, i as i32)
                });
            Ok(elem)
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
}

impl<'a> Event<'a> for SQLEvent {
    fn title(&'a self) -> &'a str {
        self.title.as_str()
    }

    fn importance(&self) -> Importance {
        match self.importance {
            0 => Importance::Normal,
            1 => Importance::High,
            _ => panic!("database corrupted"),
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
}

impl<'a> Task<'a> for SQLTask {
    fn title(&'a self) -> &'a str {
        self.title.as_str()
    }

    fn completion_level(&self) -> CompletionLevel {
        match self.completion_level {
            0 => CompletionLevel::None,
            1 => CompletionLevel::Partial,
            2 => CompletionLevel::Full,
            _ => panic!("database corrupted"),
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
