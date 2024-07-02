#![no_std]

use demo_walker as walker;
use sails_rtl::{
    cell::RefCell,
    gstd::{gprogram, groute},
};

mod counter;
mod dog;
mod mammal;
mod ping;
mod references;
mod this_that;

static mut DOG_DATA: Option<RefCell<walker::WalkerData>> = None;

fn dog_data() -> &'static RefCell<walker::WalkerData> {
    unsafe {
        DOG_DATA
            .as_ref()
            .unwrap_or_else(|| panic!("`Dog` data should be initialized first"))
    }
}

pub struct DemoProgram {
    counter_data: RefCell<counter::CounterData>,
}

#[gprogram]
impl DemoProgram {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        unsafe {
            DOG_DATA = Some(RefCell::new(walker::WalkerData::new(
                Default::default(),
                Default::default(),
            )));
        }
        Self {
            counter_data: RefCell::new(counter::CounterData::new(Default::default())),
        }
    }

    pub fn new(counter: Option<u32>, dog_position: Option<(i32, i32)>) -> Self {
        unsafe {
            let dog_position = dog_position.unwrap_or_default();
            DOG_DATA = Some(RefCell::new(walker::WalkerData::new(
                dog_position.0,
                dog_position.1,
            )));
        }
        Self {
            counter_data: RefCell::new(counter::CounterData::new(counter.unwrap_or_default())),
        }
    }

    #[groute("ping_pong")]
    pub fn ping(&self) -> ping::PingService {
        ping::PingService::default()
    }

    pub fn counter(&self) -> counter::CounterService {
        counter::CounterService::new(&self.counter_data)
    }

    pub fn dog(&self) -> dog::DogService {
        dog::DogService::new(walker::WalkerService::new(dog_data()))
    }

    pub fn references(&self) -> references::ReferenceService {
        references::ReferenceService::default()
    }

    pub fn this_that(&self) -> this_that::MyService {
        this_that::MyService::default()
    }
}
