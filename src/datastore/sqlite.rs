use diesel::prelude::*;
use diesel::{Connection, RunQueryDsl};
use thiserror::Error;
use time::Date;
use tokio::sync::mpsc;

use crate::model::{CompletionLevel, Event, Importance, Model, Task, memory};

use super::Message;

const MAX_CHANNEL: usize = 128;

pub struct Controller(mpsc::Sender<Message>);

impl super::Controller for Controller {
    fn send(&mut self, msg: Message) -> Result<(), super::Error> {
        self.0
            .blocking_send(msg)
            .map_err(|_| super::Error::Send("tokio channel failed".into()))
    }

    fn close(&mut self) {
        let _ = self.0.blocking_send(Message::Close);
    }
}

pub struct Datastore {
    connection: SqliteConnection,
}

impl Datastore {
    pub fn new(connection: SqliteConnection) -> Self {
        Self { connection }
    }
}

impl super::Datastore<memory::Model, Storer, Controller> for Datastore {
    fn into_storer_controller(self) -> Result<(Storer, Controller), super::Error> {
        let (sender, receiver) = mpsc::channel(MAX_CHANNEL);
        Ok((
            Storer {
                connection: self.connection,
                receiver,
            },
            Controller(sender),
        ))
    }

    fn load(&mut self) -> Result<memory::Model, super::Error> {
        use tables::events;
        use tables::tasks;

        let events_rows: Vec<SQLEvent> = diesel::RunQueryDsl::load(
            events::dsl::events.order((events::columns::date, events::columns::index)),
            &mut self.connection,
        )
        .map_err(|_| super::Error::Load("sqlite connection failed".into()))?;
        let task_rows: Vec<SQLTask> = diesel::RunQueryDsl::load(
            tasks::dsl::tasks.order((tasks::columns::date, tasks::columns::index)),
            &mut self.connection,
        )
        .map_err(|_| super::Error::Load("sqlite connection failed".into()))?;

        let mut model = memory::Model::new();
        for event in events_rows {
            model
                .new_event(
                    Date::from_julian_day(event.date).expect("db is in sync"),
                    event.index as usize,
                    event.into(),
                )
                .map_err(|_| super::Error::Load("creating model failed".into()))?;
        }
        for task in task_rows {
            model
                .new_task(
                    Date::from_julian_day(task.date).expect("db is in sync"),
                    task.index as usize,
                    task.into(),
                )
                .map_err(|_| super::Error::Load("creating model failed".into()))?;
        }

        Ok(model)
    }
}

pub struct Storer {
    connection: SqliteConnection,
    receiver: mpsc::Receiver<Message>,
}

impl super::Storer for Storer {
    fn run(mut self) {
        while let Some(msg) = self.receiver.blocking_recv() {
            let result = match msg {
                Message::Close => {
                    self.receiver.close();
                    Ok(())
                }
                Message::NewEvent(event, date, index) => self.new_event(event, date, index),
                Message::NewTask(task, date, index) => self.new_task(task, date, index),
                Message::DeleteEvent(date, index) => self.delete_event(date, index),
                Message::DeleteTask(date, index) => self.delete_task(date, index),
                Message::ReplaceEvent(event, date, index) => self.replace_event(event, date, index),
                Message::ReplaceTask(task, date, index) => self.replace_task(task, date, index),
                Message::SwapEvent(date, index1, index2) => self.swap_event(date, index1, index2),
                Message::SwapTask(date, index1, index2) => self.swap_task(date, index1, index2),
            };

            if result.is_err() {
                break;
            }
        }
    }
}

impl Storer {
    fn new_event(&mut self, e: Event, d: Date, i: usize) -> Result<(), Error> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d)?;

        if i <= len {
            Ok(self.connection.transaction(|conn| {
                diesel::update(events)
                    .filter(
                        date.eq(julian_date)
                            .and(index.eq(i as i32).or(index.gt(i as i32))),
                    )
                    .set(index.eq(index + 1))
                    .execute(conn)?;

                let new_event = SQLEvent::from_event(e, d, i);

                diesel::insert_into(events)
                    .values(&new_event)
                    .execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?)
        } else {
            Err(Error::NewIndexOutOfBounds(i, len))
        }
    }

    fn new_task(&mut self, t: Task, d: Date, i: usize) -> Result<(), Error> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d)?;

        if i <= len {
            Ok(self.connection.transaction(|conn| {
                diesel::update(tasks)
                    .filter(
                        date.eq(julian_date)
                            .and(index.eq(i as i32).or(index.gt(i as i32))),
                    )
                    .set(index.eq(index + 1))
                    .execute(conn)?;

                let new_task = SQLTask::from_task(t, d, i);

                diesel::insert_into(tasks).values(&new_task).execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?)
        } else {
            Err(Error::NewIndexOutOfBounds(i, len))
        }
    }

    fn delete_event(&mut self, d: Date, i: usize) -> Result<(), Error> {
        use tables::events::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.events_len(d)?;

        if i < len {
            self.connection.transaction(|conn| {
                diesel::delete(events)
                    .filter(date.eq(julian_date).and(index.eq(i as i32)))
                    .execute(conn)?;

                diesel::update(events)
                    .filter(date.eq(julian_date).and(index.gt(i as i32)))
                    .set(index.eq(index - 1))
                    .execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?;
            Ok(())
        } else {
            Err(Error::ExistingIndexOutOfBounds(i, len))
        }
    }

    fn delete_task(&mut self, d: Date, i: usize) -> Result<(), Error> {
        use tables::tasks::dsl::*;

        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d)?;

        if i < len {
            self.connection.transaction(|conn| {
                diesel::delete(tasks)
                    .filter(date.eq(julian_date).and(index.eq(i as i32)))
                    .execute(conn)?;

                diesel::update(tasks)
                    .filter(date.eq(julian_date).and(index.gt(i as i32)))
                    .set(index.eq(index - 1))
                    .execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?;
            Ok(())
        } else {
            Err(Error::ExistingIndexOutOfBounds(i, len))
        }
    }

    fn replace_event(&mut self, e: Event, d: Date, i: usize) -> Result<(), Error> {
        use tables::events::dsl::*;
        let event = SQLEvent::from_event(e, d, i);
        let julian_date = d.to_julian_day();
        let len = self.events_len(d)?;

        if i < len {
            self.connection.transaction(|conn| {
                diesel::delete(events)
                    .filter(date.eq(julian_date).and(index.eq(i as i32)))
                    .execute(conn)?;

                diesel::insert_into(events).values(&event).execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?;
            Ok(())
        } else {
            Err(Error::ExistingIndexOutOfBounds(i, len))
        }
    }

    fn replace_task(&mut self, t: Task, d: Date, i: usize) -> Result<(), Error> {
        use tables::tasks::dsl::*;
        let task = SQLTask::from_task(t, d, i);
        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d)?;

        if i < len {
            self.connection.transaction(|conn| {
                diesel::delete(tasks)
                    .filter(date.eq(julian_date).and(index.eq(i as i32)))
                    .execute(conn)?;

                diesel::insert_into(tasks).values(&task).execute(conn)?;

                diesel::result::QueryResult::Ok(())
            })?;
            Ok(())
        } else {
            Err(Error::ExistingIndexOutOfBounds(i, len))
        }
    }

    fn swap_event(&mut self, d: Date, index1: usize, index2: usize) -> Result<(), Error> {
        use tables::events::dsl::*;
        let julian_date = d.to_julian_day();
        let len = self.events_len(d)?;

        if index1 < len {
            if index2 < len {
                self.connection.transaction(|conn| {
                    diesel::update(events)
                        .filter(date.eq(julian_date).and(index.eq(index1 as i32)))
                        .set(index.eq(-1))
                        .execute(conn)?;

                    diesel::update(events)
                        .filter(date.eq(julian_date).and(index.eq(index2 as i32)))
                        .set(index.eq(index1 as i32))
                        .execute(conn)?;

                    diesel::update(events)
                        .filter(date.eq(julian_date).and(index.eq(-1)))
                        .set(index.eq(index2 as i32))
                        .execute(conn)?;

                    diesel::result::QueryResult::Ok(())
                })?;
                Ok(())
            } else {
                Err(Error::ExistingIndexOutOfBounds(index2, len))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index1, len))
        }
    }

    fn swap_task(&mut self, d: Date, index1: usize, index2: usize) -> Result<(), Error> {
        use tables::tasks::dsl::*;
        let julian_date = d.to_julian_day();
        let len = self.tasks_len(d)?;

        if index1 < len {
            if index2 < len {
                self.connection.transaction(|conn| {
                    diesel::update(tasks)
                        .filter(date.eq(julian_date).and(index.eq(index1 as i32)))
                        .set(index.eq(-1))
                        .execute(conn)?;

                    diesel::update(tasks)
                        .filter(date.eq(julian_date).and(index.eq(index2 as i32)))
                        .set(index.eq(index1 as i32))
                        .execute(conn)?;

                    diesel::update(tasks)
                        .filter(date.eq(julian_date).and(index.eq(-1)))
                        .set(index.eq(index2 as i32))
                        .execute(conn)?;

                    diesel::result::QueryResult::Ok(())
                })?;
                Ok(())
            } else {
                Err(Error::ExistingIndexOutOfBounds(index2, len))
            }
        } else {
            Err(Error::ExistingIndexOutOfBounds(index1, len))
        }
    }

    fn events_len(&mut self, d: Date) -> Result<usize, Error> {
        use tables::events::dsl::*;

        Ok(events
            .filter(date.eq(d.to_julian_day()))
            .count()
            .get_result::<i64>(&mut self.connection)? as usize)
    }

    fn tasks_len(&mut self, d: Date) -> Result<usize, Error> {
        use tables::tasks::dsl::*;

        Ok(tasks
            .filter(date.eq(d.to_julian_day()))
            .count()
            .get_result::<i64>(&mut self.connection)? as usize)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlite failed")]
    Sqlite(#[from] diesel::result::Error),
    #[error("index out of bounds for insertion: index {0} is out of bounds for {1} elements")]
    NewIndexOutOfBounds(usize, usize),
    #[error("index out of bounds for modification: index {0} is out of bounds for {1} elements")]
    ExistingIndexOutOfBounds(usize, usize),
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
    fn from_event(event: Event, date: Date, index: usize) -> Self {
        let importance = match event.importance {
            Importance::Normal => 0,
            Importance::High => 1,
            Importance::Extreme => 2,
        };

        Self {
            title: event.title,
            importance,
            date: date.to_julian_day(),
            index: index as i32,
        }
    }
}

impl From<SQLEvent> for Event {
    fn from(value: SQLEvent) -> Self {
        Event {
            title: value.title,
            importance: match value.importance {
                0 => Importance::Normal,
                1 => Importance::High,
                2 => Importance::Extreme,
                _ => panic!("db out of sync"),
            },
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
    fn from_task(task: Task, date: Date, index: usize) -> Self {
        let completion_level = match task.completion_level {
            CompletionLevel::None => 0,
            CompletionLevel::Partial => 1,
            CompletionLevel::Full => 2,
        };

        Self {
            title: task.title,
            completion_level,
            date: date.to_julian_day(),
            index: index as i32,
        }
    }
}

impl From<SQLTask> for Task {
    fn from(value: SQLTask) -> Self {
        Task {
            title: value.title,
            completion_level: match value.completion_level {
                0 => CompletionLevel::None,
                1 => CompletionLevel::Partial,
                2 => CompletionLevel::Full,
                _ => panic!("db out of sync"),
            },
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
