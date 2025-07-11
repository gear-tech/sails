use sails_rs::prelude::*;

pub const BASE_NAME_RESULT: &str = "base-name";
pub const HIDDEN_NAME_RESULT: &str = "base";

#[derive(Clone)]
pub struct BaseWithLifetime<'a> {
    _int: &'a u64,
}

impl<'a> BaseWithLifetime<'a> {
    pub fn new(int: &'a u64) -> Self {
        BaseWithLifetime { _int: int }
    }
}

#[service]
impl BaseWithLifetime<'_> {
    #[export]
    pub fn base_name(&self) -> String {
        BASE_NAME_RESULT.to_string()
    }

    #[export]
    pub fn name(&self) -> String {
        HIDDEN_NAME_RESULT.to_string()
    }
}

pub const EXTENDED_NAME_RESULT: &str = "extended-name";
pub const NAME_RESULT: &str = "extended";

pub struct ExtendedWithLifetime<'a> {
    base: BaseWithLifetime<'a>,
}

impl<'a> ExtendedWithLifetime<'a> {
    pub fn new(base: BaseWithLifetime<'a>) -> Self {
        Self { base }
    }
}

#[allow(clippy::needless_lifetimes)]
#[service(extends = BaseWithLifetime<'a>)]
impl<'a> ExtendedWithLifetime<'a> {
    #[export]
    pub fn extended_name(&self) -> String {
        EXTENDED_NAME_RESULT.to_string()
    }

    #[export]
    pub fn name(&self) -> String {
        NAME_RESULT.to_string()
    }
}

impl<'a> From<ExtendedWithLifetime<'a>> for BaseWithLifetime<'a> {
    fn from(value: ExtendedWithLifetime<'a>) -> Self {
        value.base
    }
}
