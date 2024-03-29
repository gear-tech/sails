use sails_rtl::{Decode, Encode, Result as RtlResult, TypeInfo};

pub type Result<T, E = Error> = RtlResult<T, E>;

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum Error {
    NotAuthorized,
    ZeroResourceId,
    ResourceAlreadyExists,
    ResourceNotFound,
    WrongResourceType,
    PartNotFound,
}
