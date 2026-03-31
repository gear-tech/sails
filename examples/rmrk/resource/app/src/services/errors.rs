use sails_rs::prelude::*;

pub type Result<T, E = Error> = sails_rs::Result<T, E>;

#[derive(Encode, Decode, TypeInfo, Debug, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[reflect_hash(crate = sails_rs)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
