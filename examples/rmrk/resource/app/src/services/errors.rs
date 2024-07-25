use sails_rs::prelude::*;

pub type Result<T, E = Error> = sails_rs::Result<T, E>;

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
