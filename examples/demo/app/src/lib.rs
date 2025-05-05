#![no_std]

use demo_walker as walker;
use sails_rs::{cell::RefCell, prelude::*};

mod counter;
mod dog;
mod mammal;
mod ping;
mod references;
mod this_that;
mod value_fee;

// Dog data is stored as a global variable. However, it has exactly the same lifetime
// the Counter data incapsulated in the program itself, i.e. there are no any benefits
// of using a global variable here. It is just a demonstration of how to use global variables.
static mut DOG_DATA: Option<RefCell<walker::WalkerData>> = None;
static mut REF_DATA: u8 = 42;

#[allow(static_mut_refs)]
fn dog_data() -> &'static RefCell<walker::WalkerData> {
    unsafe {
        DOG_DATA
            .as_ref()
            .unwrap_or_else(|| panic!("`Dog` data should be initialized first"))
    }
}

pub struct DemoProgram {
    // Counter data has the same lifetime as the program itself, i.e. it will
    // live as long as the program is available on the network.
    counter_data: RefCell<counter::CounterData>,
}

#[program(accept_transfers)]
impl DemoProgram {
    #[allow(clippy::should_implement_trait)]
    /// Program constructor (called once at the very beginning of the program lifetime)
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

    /// Another program constructor (called once at the very beginning of the program lifetime)
    #[export(unwrap_result)]
    pub fn new(counter: Option<u32>, dog_position: Option<(i32, i32)>) -> Result<Self, String> {
        unsafe {
            let dog_position = dog_position.unwrap_or_default();
            DOG_DATA = Some(RefCell::new(walker::WalkerData::new(
                dog_position.0,
                dog_position.1,
            )));
        }
        Ok(Self {
            counter_data: RefCell::new(counter::CounterData::new(counter.unwrap_or_default())),
        })
    }

    // Exposing service with overriden route
    #[export(route = "ping_pong", unwrap_result)]
    pub fn ping(&self) -> Result<ping::PingService, String> {
        Ok(ping::PingService::default())
    }

    // Exposing another service
    pub fn counter(&self) -> counter::CounterService {
        counter::CounterService::new(&self.counter_data)
    }

    // Exposing yet another service
    pub fn dog(&self) -> dog::DogService {
        dog::DogService::new(walker::WalkerService::new(dog_data()))
    }

    pub fn references(&self) -> references::ReferenceService {
        #[allow(static_mut_refs)]
        unsafe {
            references::ReferenceService::new(&mut REF_DATA, "demo")
        }
    }

    pub fn this_that(&self) -> this_that::MyService {
        this_that::MyService::default()
    }

    pub fn value_fee(&self) -> value_fee::FeeService {
        value_fee::FeeService::new(10_000_000_000_000)
    }
}
