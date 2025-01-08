use sails_rs::{prelude::*, static_storage};

#[derive(Default)]
pub struct Data(pub u128);
static_storage!(Data, Data(0u128));

pub struct Service<'a> {
    storage: Box<dyn Storage<Item = Data> + 'a>,
}

impl<'a> Service<'a> {
    pub fn new(storage: impl Storage<Item = Data> + 'a) -> Self {
        Self {
            storage: Box::new(storage),
        }
    }

    pub fn from_accessor<T: StorageAccessor<'a, Data>>(accessor: &'a T) -> Self {
        Self {
            storage: accessor.boxed(),
        }
    }
}

#[service(events = Event)]
impl Service<'_> {
    pub fn bump(&mut self) {
        let state = self.storage.get_mut();
        state.0 = state.0.saturating_add(1);

        self.notify_on(Event::Bumped).expect("unable to emit event");
    }

    pub fn get(&self) -> u128 {
        self.storage.get().0
    }
}

#[derive(Clone, Debug, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum Event {
    Bumped,
}
