use sails_rs::{cell::RefCell, prelude::*};

pub struct ValidatorData {
    pub total_errors: u32,
}

impl ValidatorData {
    pub fn new() -> Self {
        Self { total_errors: 0 }
    }
}

pub struct Validator<'a> {
    data: &'a RefCell<ValidatorData>,
}

impl<'a> Validator<'a> {
    pub fn new(data: &'a RefCell<ValidatorData>) -> Self {
        Self { data }
    }
}

#[derive(Debug, Encode, Decode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs::sails_reflect_hash)]
pub enum ValidationError {
    TooSmall,
    TooBig,
}

#[service]
impl<'a> Validator<'a> {
    #[export]
    pub fn validate_range(
        &mut self,
        value: u32,
        min: u32,
        max: u32,
    ) -> Result<u32, ValidationError> {
        if value < min {
            self.data.borrow_mut().total_errors += 1;
            Err(ValidationError::TooSmall)
        } else if value > max {
            self.data.borrow_mut().total_errors += 1;
            Err(ValidationError::TooBig)
        } else {
            Ok(value)
        }
    }

    #[export]
    pub fn validate_nonzero(&mut self, value: u32) -> Result<(), String> {
        if value == 0 {
            self.data.borrow_mut().total_errors += 1;
            Err("Value is zero".to_string())
        } else {
            Ok(())
        }
    }

    #[export]
    pub fn validate_even(&self, value: u32) -> Result<u32, ()> {
        if !value.is_multiple_of(2) {
            Err(())
        } else {
            Ok(value)
        }
    }

    #[export]
    pub fn total_errors(&self) -> u32 {
        self.data.borrow().total_errors
    }
}
