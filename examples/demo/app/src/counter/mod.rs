use sails_rs::{cell::RefCell, prelude::*};

// Model of the service's data. Only service knows what is the data
// and how to manipulate it.
pub struct CounterData {
    counter: u32,
}

impl CounterData {
    // The only method exposed publicly for creating a new instance of the data.
    pub const fn new(counter: u32) -> Self {
        Self { counter }
    }
}

// Service event type definition.
#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum CounterEvents {
    Added(u32),
    Subtracted(u32),
}

pub struct CounterService<'a> {
    data: &'a RefCell<CounterData>,
}

impl<'a> CounterService<'a> {
    // Service constrctor demands a reference to the data to be passed
    // from the outside.
    pub fn new(data: &'a RefCell<CounterData>) -> Self {
        Self { data }
    }
}

// Declare the service can emit events of type CounterEvents.
#[service(events = CounterEvents)]
impl<'a> CounterService<'a> {
    pub fn add(&mut self, value: u32) -> u32 {
        let mut data_mut = self.data.borrow_mut();
        data_mut.counter += value;
        // Emit event right before the method returns via
        // the generated `notify_on` method.
        self.notify_on(CounterEvents::Added(value)).unwrap();
        data_mut.counter
    }

    pub fn sub(&mut self, value: u32) -> u32 {
        let mut data_mut = self.data.borrow_mut();
        data_mut.counter -= value;
        // Emit event right before the method returns via
        // the generated `notify_on` method.
        self.notify_on(CounterEvents::Subtracted(value)).unwrap();
        data_mut.counter
    }

    pub fn value(&self) -> u32 {
        self.data.borrow().counter
    }
}
