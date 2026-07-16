use sails::prelude::*;

pub type Result<T, E = Error> = sails::Result<T, E>;

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
