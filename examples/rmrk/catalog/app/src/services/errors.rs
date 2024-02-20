use sails_rtl_gstd::{Encode, TypeInfo};

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
