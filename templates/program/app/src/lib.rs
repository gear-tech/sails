#![no_std]

use sails_rs::prelude::*;

// Program's state
static mut STATE: Option<String> = None;

// Service's events
#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum {{ service-events-name }} {
    SaidHello { name: String },
    Forgot,
}

struct {{ service-struct-name }}(());

#[sails_rs::service(events = {{ service-events-name }})]
impl {{ service-struct-name }} {
    pub fn new() -> Self {
        Self(())
    }

    // Service's method (command)
    pub fn say_hello(&mut self, name: String) -> String {
        unsafe {
            STATE = Some(name.clone());
        }
        self.notify_on({{ service-events-name }}::SaidHello { name: name.clone() })
            .unwrap();
        format!("Hello {} from {{ service-name }}!", name)
    }

    // Service's method (command)
    pub fn forget(&mut self) {
        unsafe {
            STATE = None;
        }
        self.notify_on({{ service-events-name }}::Forgot).unwrap();
    }

    // Service's query with lifetime
    pub fn last_name<'a>(&self) -> Option<&'a str> {
        unsafe { STATE.as_deref() }
    }
}

pub struct {{ program-struct-name }}(());

#[sails_rs::program]
impl {{ program-struct-name }} {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    // Exposed service
    pub fn {{ service-name-snake}}(&self) -> {{ service-struct-name }} {
        {{ service-struct-name }}::new()
    }
}
