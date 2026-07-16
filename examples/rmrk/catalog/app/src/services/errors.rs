use sails::prelude::*;

#[sails_type]
pub enum Error {
    PartIdCantBeZero,
    BadConfig,
    PartAlreadyExists,
    ZeroLengthPassed,
    PartDoesNotExist,
    WrongPartFormat,
    NotAllowedToCall,
}
