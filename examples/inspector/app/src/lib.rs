#![no_std]

use demo_client::{
    DemoClient, DemoClientCtors, DemoClientProgram,
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

    /// Try to deploy demo with error - should return Err
    #[export]
    pub async fn test_failing_demo_ctor(&self, demo_code_id: CodeId) -> Result<ActorId, String> {
        use sails_rs::client::Program;
        let demo_factory = DemoClientProgram::deploy(demo_code_id, vec![1]);
        let res = demo_factory.new_with_error(0).await;
        match res {
            Ok(Ok(client)) => Ok(client.id()),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.to_string()),
        }
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

    pub fn new_with_result(target: ActorId) -> Result<Self, String> {
        if target.is_zero() {
            return Err("Target program cannot be zero".to_string());
        }
        Ok(Self { target })
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
