#![no_std]

use sails_rs::{cell::RefCell, prelude::*};

#[derive(Clone)]
pub struct WalkerData {
    x: i32,
    y: i32,
}

impl WalkerData {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[event]
#[sails_type]
pub enum WalkerEvents {
    Walked { from: (i32, i32), to: (i32, i32) },
}

// Yet another example of using `StateMut` for service state.
// This time it demonstrates use of static lifetime with `RefCell<T>`
#[derive(Clone)]
pub struct WalkerService<
    S: StateMut<Item = WalkerData, Error = Infallible> = &'static RefCell<WalkerData>,
> {
    data: S,
}

impl<S: StateMut<Item = WalkerData, Error = Infallible>> WalkerService<S> {
    pub fn new(data: S) -> Self {
        Self { data }
    }
}

#[service(events = WalkerEvents)]
impl<S: StateMut<Item = WalkerData, Error = Infallible>> WalkerService<S> {
    #[export]
    pub fn walk(&mut self, dx: i32, dy: i32) {
        let from = self.position();
        {
            let mut data = self.data.get_mut();
            data.x += dx;
            data.y += dy;
        }
        let to = self.position();
        self.emit_event(WalkerEvents::Walked { from, to }).unwrap();
    }

    #[export]
    pub fn position(&self) -> (i32, i32) {
        let data = self.data.get();
        (data.x, data.y)
    }
}
