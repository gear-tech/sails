#![no_std]

use demo_client::{
    DemoClient, DemoClientProgram,
    validator::{ValidationError, Validator},
};
use sails_rs::{client::*, prelude::*};

pub struct InspectorService {
    validator: Service<demo_client::validator::ValidatorImpl>,
}

impl InspectorService {
    pub fn new(target: ActorId) -> Self {
        let demo = DemoClientProgram::client(target);
        Self {
            validator: demo.validator(),
        }
    }
}

#[sails_rs::service]
impl InspectorService {
    /// Proxy call to validator.validate_range(20, 5, 15)
    #[export]
    pub async fn test_range_panic(&mut self) -> Result<u32, ValidationError> {
        self.validator.validate_range(20, 5, 15).await.unwrap()
    }

    /// Proxy call to validator.validate_nonzero(0)
    #[export]
    pub async fn test_nonzero_panic(&mut self) -> Result<(), String> {
        self.validator.validate_nonzero(0).await.unwrap()
    }

    /// Proxy call to validator.validate_even(7)
    #[export]
    pub async fn test_even_panic(&self) -> Result<u32, ()> {
        self.validator.validate_even(7).await.unwrap()
    }

    /// Proxy call to validator.total_errors()
    #[export]
    pub async fn test_total_errors(&self) -> u32 {
        self.validator.total_errors().await.unwrap()
    }
}

pub struct InspectorProgram {
    target: ActorId,
}

#[sails_rs::program]
impl InspectorProgram {
    pub fn new(target: ActorId) -> Self {
        Self { target }
    }

    pub fn inspector(&self) -> InspectorService {
        InspectorService::new(self.target)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
