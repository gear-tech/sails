use sails_rs::prelude::*;

#[derive(Default)]
pub struct Validator;

#[derive(Debug, Encode, Decode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs::sails_reflect_hash)]
pub enum ValidationError {
    TooSmall,
    TooBig,
}

#[service]
impl Validator {
    #[export]
    pub fn validate_range(&self, value: u32, min: u32, max: u32) -> Result<u32, ValidationError> {
        if value < min {
            Err(ValidationError::TooSmall)
        } else if value > max {
            Err(ValidationError::TooBig)
        } else {
            Ok(value)
        }
    }
}
