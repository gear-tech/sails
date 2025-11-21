use sails_rs::{Encode, ReflectHash, TypeInfo};

#[derive(Encode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs)]
pub enum Error {
    PartIdCantBeZero,
    BadConfig,
    PartAlreadyExists,
    ZeroLengthPassed,
    PartDoesNotExist,
    WrongPartFormat,
    NotAllowedToCall,
}
