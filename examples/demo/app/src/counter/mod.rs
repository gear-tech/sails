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
#[event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum CounterEvents {
    /// Emitted when a new value is added to the counter
    Added(u32),
    /// Emitted when a value is subtracted from the counter
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
impl CounterService<'_> {
    /// Add a value to the counter
    pub fn add(&mut self, value: u32) -> u32 {
        let mut data_mut = self.data.borrow_mut();
        data_mut.counter += value;
        // Emit event right before the method returns via
        // the generated `emit_event` method.
        self.emit_event(CounterEvents::Added(value)).unwrap();
        data_mut.counter
    }

    /// Substract a value from the counter
    pub fn sub(&mut self, value: u32) -> u32 {
        let mut data_mut = self.data.borrow_mut();
        data_mut.counter -= value;
        // Emit event right before the method returns via
        // the generated `emit_event` method.
        self.emit_event(CounterEvents::Subtracted(value)).unwrap();
        data_mut.counter
    }

    /// Get the current value
    pub fn value(&self) -> u32 {
        self.data.borrow().counter
    }
}
