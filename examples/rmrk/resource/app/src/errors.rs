use gstd::prelude::*;

pub type Result<T> = gstd::Result<T, Error>;

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
