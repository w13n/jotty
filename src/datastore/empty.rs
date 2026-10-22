use crate::model::memory;

use super::Error;

#[allow(unused)]
pub struct Datastore {}
#[allow(unused)]
pub struct Storer {}
#[allow(unused)]
pub struct Controller {}

impl super::Datastore<memory::Model, Storer, Controller> for Datastore {
    fn load(&mut self) -> Result<memory::Model, Error> {
        Ok(memory::Model::new())
    }

    fn into_storer_controller(self) -> Result<(Storer, Controller), Error> {
        Ok((Storer {}, Controller {}))
    }
}

impl super::Storer for Storer {
    fn run(self) {}
}

impl super::Controller for Controller {
    fn send(&mut self, _: super::Message) -> Result<(), Error> {
        Ok(())
    }

    fn close(&mut self) {}
}
