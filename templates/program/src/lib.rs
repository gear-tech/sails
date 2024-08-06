#![no_std]

use sails_rs::prelude::*;

struct {{ program-name }}Service(());

#[sails_rs::service]
impl {{ program-name }}Service {
    pub fn new() -> Self {
        Self(())
    }

    pub fn do_something(&mut self) -> String {
        "Hello from {{ program-name }}!".to_string()
    }
}

pub struct {{ program-name }}Program(());

#[sails_rs::program]
impl {{ program-name }}Program {
    pub fn new() -> Self {
        Self(())
    }

    pub fn {{ program-name-snake}}(&self) -> {{ program-name }}Service {
        {{ program-name }}Service::new()
    }
}
