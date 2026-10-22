pub mod empty;
pub mod sqlite;

use time::Date;

use thiserror::Error;

use crate::model::{Event, Model, Task};

pub trait Datastore<M: Model, S: Storer, C: Controller>: Sized {
    fn load(&mut self) -> Result<M, Error>;
    fn into_storer_controller(self) -> Result<(S, C), Error>;
}

pub trait Storer: Sized {
    fn run(self);
}

pub trait Controller {
    fn send(&mut self, msg: Message) -> Result<(), Error>;
    fn close(&mut self);
}

#[derive(Debug)]
pub enum Message {
    Close,
    NewEvent(Event, Date, usize),
    NewTask(Task, Date, usize),
    DeleteEvent(Date, usize),
    DeleteTask(Date, usize),
    ReplaceEvent(Event, Date, usize),
    ReplaceTask(Task, Date, usize),
    SwapEvent(Date, usize, usize),
    SwapTask(Date, usize, usize),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("loading failed: {0}")]
    Load(String),
    #[error("sending failed: {0}")]
    Send(String),
}
