use gstd::prelude::*;

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
