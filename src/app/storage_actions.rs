use std::sync::mpsc;

use diesel::prelude::*;
use time::Date;

use crate::journal::{Event, Task};

pub(super) enum StorageAction {
    InsertTask(Date, usize, Task),
    UpdateTask(Date, usize, Task),
    DeleteTask(Date, usize),
    InsertEvent(Date, usize, Event),
    UpdateEvent(Date, usize, Event),
    DeleteEvent(Date, usize),
}

fn run_tasks(messages: mpsc::Receiver<StorageAction>) {
    let mut path = directories_next::ProjectDirs::from("com", "w13n", "jotty")
        .expect("Thread is usless if directory cannot be found")
        .data_dir()
        .to_path_buf();

    path.push("v1.db");

    let conn = SqliteConnection::establish(
        path.as_os_str()
            .to_str()
            .expect("thread is useless if we cannot connect to db"),
    )
    .expect("thread is useless if we cannot connect to db");

    while let Ok(msg) = messages.recv() {
        match msg {
            StorageAction::InsertTask(date, index, task) => {}
            StorageAction::UpdateTask(date, index, task) => {}
            StorageAction::DeleteTask(date, index) => {}
            StorageAction::InsertEvent(date, index, event) => {}
            StorageAction::UpdateEvent(date, index, event) => {}
            StorageAction::DeleteEvent(date, index) => {}
        }
    }
}
