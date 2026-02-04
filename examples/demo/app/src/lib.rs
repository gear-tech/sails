#![no_std]

use demo_walker as walker;
use sails_rs::{cell::RefCell, prelude::*};

mod chaos;
mod counter;
mod dog;
mod inheritance;
mod mammal;
mod ping;
mod references;
mod this_that;
mod value_fee;

// Dog data is stored as a global variable. However, it has exactly the same lifetime
// the Counter data incapsulated in the program itself, i.e. there are no any benefits
// of using a global variable here. It is just a demonstration of how to use global variables.
static mut DOG_DATA: Option<RefCell<walker::WalkerData>> = None;

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
    ref_data: u8,
}

#[program(payable)]
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
            ref_data: 42,
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
            ref_data: 42,
        })
    }

    // Exposing service with overriden route
    #[export(route = "ping_pong", unwrap_result)]
    pub fn ping(&self) -> Result<ping::PingService, String> {
        Ok(ping::PingService::default())
    }

    // Exposing another service
    pub fn counter(&self) -> counter::CounterService<'_> {
        counter::CounterService::new(&self.counter_data)
    }

    // Exposing yet another service
    pub fn dog(&self) -> dog::DogService {
        dog::DogService::new(walker::WalkerService::new(dog_data()))
    }

    pub fn references(&mut self) -> references::ReferenceService<'_> {
        references::ReferenceService::new(&mut self.ref_data, "demo")
    }

    pub fn this_that(&self) -> this_that::MyService {
        this_that::MyService::default()
    }

    pub fn value_fee(&self) -> value_fee::FeeService {
        value_fee::FeeService::new(10_000_000_000_000)
    }

    pub fn chaos(&self) -> chaos::ChaosService {
        chaos::ChaosService
    }

    pub fn chain(&self) -> inheritance::ChainService {
        inheritance::ChainService::new(dog::DogService::new(walker::WalkerService::new(dog_data())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sails_rs::gstd::services::Exposure;

    // Test program constructor and exposed service
    // Mock `Syscall` to simulate the environment
    #[tokio::test]
    async fn program_service_exposure() {
        // Arrange
        let program = DemoProgram::new(Some(42), None).unwrap();

        // First call
        let message_value = 100_000_000_000_000;
        Syscall::with_message_value(message_value);
        Syscall::with_message_id(MessageId::from(1));

        let mut service_exposure = program.value_fee();
        let (data, value) = service_exposure.do_something_and_take_fee().to_tuple();

        // Assert
        assert_eq!(6, service_exposure.route_idx());
        assert!(data);
        assert_eq!(value, message_value - 10_000_000_000_000);

        // Next call
        Syscall::with_message_value(0);
        Syscall::with_message_id(MessageId::from(2));

        let mut service_exposure = program.counter();
        let mut emitter = service_exposure.emitter();
        let data = service_exposure.add(10);

        // Assert
        assert_eq!(2, service_exposure.route_idx());
        assert_eq!(52, data);
        let events = emitter.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], counter::CounterEvents::Added(10));
    }
}
