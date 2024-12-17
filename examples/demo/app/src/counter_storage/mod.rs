use sails_rs::prelude::*;

pub struct Service<'a> {
    storage: Box<dyn Storage<Item = u128> + 'a>,
}

impl<'a> Service<'a> {
    pub fn new(storage: impl Storage<Item = u128> + 'a) -> Self {
        Self {
            storage: Box::new(storage),
        }
    }

    pub fn from_accessor<T: StorageAccessor<'a, u128>>(accessor: &'a T) -> Self {
        Self {
            storage: accessor.boxed(),
        }
    }
}

#[service(events = Event)]
impl Service<'_> {
    pub fn bump(&mut self) {
        let state = self.storage.get_mut();

        *state = state.saturating_add(1);

        self.notify_on(Event::Bumped).expect("unable to emit event");
    }

    pub fn get(&self) -> u128 {
        *self.storage.get()
    }
}

#[derive(Clone, Debug, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum Event {
    Bumped,
}
