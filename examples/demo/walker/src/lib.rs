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

#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum WalkerEvents {
    Walked { from: (i32, i32), to: (i32, i32) },
}

// Service extension requires the service to implement `Clone`
#[derive(Clone)]
pub struct WalkerService {
    // Yet another example of using `RefCell` for service
    // state. This time it demonstrates use of static lifetime
    // even though it is not the best option
    data: &'static RefCell<WalkerData>,
}

impl WalkerService {
    pub fn new(data: &'static RefCell<WalkerData>) -> Self {
        Self { data }
    }
}

#[service(events = WalkerEvents)]
impl WalkerService {
    pub fn walk(&mut self, dx: i32, dy: i32) {
        let from = self.position();
        {
            let mut data = self.data.borrow_mut();
            data.x += dx;
            data.y += dy;
        }
        let to = self.position();
        self.notify_on(WalkerEvents::Walked { from, to }).unwrap();
    }

    pub fn position(&self) -> (i32, i32) {
        let data = self.data.borrow();
        (data.x, data.y)
    }
}
