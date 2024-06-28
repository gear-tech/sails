use sails_rtl::{gstd::gservice, prelude::*};

pub struct CounterData {
    counter: u32,
}

impl CounterData {
    pub const fn new(counter: u32) -> Self {
        Self { counter }
    }
}

#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
enum CounterEvents {
    Added(u32),
    Subtracted(u32),
}

pub struct CounterService<'a> {
    data: &'a mut CounterData,
}

impl<'a> CounterService<'a> {
    pub fn new(data: &'a mut CounterData) -> Self {
        Self { data }
    }
}

#[gservice(events = CounterEvents)]
impl<'a> CounterService<'a> {
    pub fn add(&mut self, value: u32) -> u32 {
        self.data.counter += value;
        self.notify_on(CounterEvents::Added(value)).unwrap();
        self.data.counter
    }

    pub fn sub(&mut self, value: u32) -> u32 {
        self.data.counter -= value;
        self.notify_on(CounterEvents::Subtracted(value)).unwrap();
        self.data.counter
    }

    pub fn value(&self) -> u32 {
        self.data.counter
    }
}
