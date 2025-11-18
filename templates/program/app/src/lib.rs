#![no_std]

extern crate alloc;

use sails_rs::prelude::*;

struct {{ service-struct-name }}(());

impl {{ service-struct-name }} {
    pub fn new() -> Self {
        Self(())
    }
}

#[sails_rs::service]
impl {{ service-struct-name }} {
    // Service's method (command)
    #[export]
    pub fn do_something(&mut self) -> String {
        "Hello from {{ service-name }}!".to_string()
    }

    // Service's query
    #[export]
    pub fn get_something(&self) -> String {
        "Hello from {{ service-name }}!".to_string()
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
