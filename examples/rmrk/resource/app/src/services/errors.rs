use sails::prelude::*;

pub type Result<T, E = Error> = sails::Result<T, E>;

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = sails::scale_codec)]
#[scale_info(crate = sails::scale_info)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
