use sails_rs::prelude::*;

pub type Result<T, E = Error> = sails_rs::Result<T, E>;

#[sails_type]
#[derive(Debug)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
