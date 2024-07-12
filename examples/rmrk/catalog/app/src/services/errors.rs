use sails::{Encode, TypeInfo};

#[derive(Encode, TypeInfo)]
pub enum Error {
    PartIdCantBeZero,
    BadConfig,
    PartAlreadyExists,
    ZeroLengthPassed,
    PartDoesNotExist,
    WrongPartFormat,
    NotAllowedToCall,
}
