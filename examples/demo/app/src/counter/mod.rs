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
#[sails_type]
#[derive(Clone, Debug, PartialEq)]
pub enum CounterEvents {
    /// Emitted when a new value is added to the counter
    Added(u32),
    /// Emitted when a value is subtracted from the counter
    Subtracted(u32),
}

// `CounterService` is generic over its state handle `S: StateMut`. The impl
// stays agnostic of how the data is stored — any type that implements
// `StateMut<Item = CounterData, Error = Infallible>` works:
// a `&RefCell<CounterData>` borrowed from program state (production default),
// an owned `RefCell<CounterData>`, an `Rc<RefCell<CounterData>>`, a
// `&mut RefCell<CounterData>` (via the `&mut S` blanket in `state.rs`), or a
// custom wrapper (pausing, metering, etc.).
pub struct CounterService<
    S: StateMut<Item = CounterData, Error = Infallible> = RefCell<CounterData>,
> {
    data: S,
}

impl<S: StateMut<Item = CounterData, Error = Infallible>> CounterService<S> {
    // Service constructor demands the state handle to be passed from outside.
    pub fn new(data: S) -> Self {
        Self { data }
    }
}

// Declare the service can emit events of type CounterEvents.
#[service(events = CounterEvents)]
impl<S: StateMut<Item = CounterData, Error = Infallible>> CounterService<S> {
    /// Add a value to the counter
    #[export]
    pub fn add(&mut self, value: u32) -> u32 {
        let counter = {
            // Access mutable state
            let mut data_mut = self.data.get_mut();
            data_mut.counter += value;
            data_mut.counter
        };

        // Emit event right before the method returns via
        // the generated `emit_event` method.
        self.emit_event(CounterEvents::Added(value)).unwrap();
        counter
    }

    /// Subtract a value from the counter
    #[export]
    pub fn sub(&mut self, value: u32) -> u32 {
        let counter = {
            // Access mutable state
            let mut data_mut = self.data.get_mut();
            data_mut.counter -= value;
            data_mut.counter
        };
        // Emit event right before the method returns via
        // the generated `emit_event` method.
        self.emit_event(CounterEvents::Subtracted(value)).unwrap();
        counter
    }

    /// Get the current value
    #[export]
    pub fn value(&self) -> u32 {
        self.data.get().counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sails_rs::gstd::services::Service;

    #[test]
    fn value_returns_initial_state() {
        let data = RefCell::new(CounterData::new(7));
        let service = CounterService::new(&data).expose(0);
        assert_eq!(service.value(), 7);
    }

    #[test]
    fn add_increments_and_emits_event() {
        Syscall::with_message_id(MessageId::from(1));

        let data = RefCell::new(CounterData::new(0));
        let mut service = CounterService::new(&data).expose(1);
        let mut emitter = service.emitter();

        assert_eq!(service.add(5), 5);
        assert_eq!(service.add(3), 8);
        assert_eq!(data.borrow().counter, 8);

        let events = emitter.take_events();
        assert_eq!(events, [CounterEvents::Added(5), CounterEvents::Added(3)]);
    }

    #[test]
    fn sub_decrements_and_emits_event() {
        Syscall::with_message_id(MessageId::from(2));

        let data = RefCell::new(CounterData::new(10));
        let mut service = CounterService::new(&data).expose(1);
        let mut emitter = service.emitter();

        assert_eq!(service.sub(4), 6);
        assert_eq!(data.borrow().counter, 6);

        let events = emitter.take_events();
        assert_eq!(events, [CounterEvents::Subtracted(4)]);
    }

    // Demonstrates the StateMut abstraction with an owned `RefCell` (the
    // default `S`) — no borrow plumbing needed.
    #[test]
    fn works_with_owned_refcell() {
        Syscall::with_message_id(MessageId::from(3));

        let service: CounterService = CounterService::new(RefCell::new(CounterData::new(0)));
        let mut exposed = service.expose(1);

        assert_eq!(exposed.add(2), 2);
        assert_eq!(exposed.value(), 2);
    }

    // Demonstrates the `&mut S` blanket impl in `state.rs`: passing
    // `&mut RefCell<T>` satisfies `StateMut` without a new concrete impl.
    #[test]
    fn works_with_mut_ref_to_refcell() {
        Syscall::with_message_id(MessageId::from(4));

        let mut data = RefCell::new(CounterData::new(0));
        let service = CounterService::new(&mut data);
        let mut exposed = service.expose(1);

        assert_eq!(exposed.add(2), 2);
        assert_eq!(exposed.value(), 2);
    }
}
