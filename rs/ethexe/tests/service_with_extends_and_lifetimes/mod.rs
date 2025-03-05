use sails_rs::service;

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
    pub fn base_name(&self) -> String {
        BASE_NAME_RESULT.to_string()
    }

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
    pub fn extended_name(&self) -> String {
        EXTENDED_NAME_RESULT.to_string()
    }

    pub fn name(&self) -> String {
        NAME_RESULT.to_string()
    }
}

impl<'a> AsRef<BaseWithLifetime<'a>> for ExtendedWithLifetime<'a> {
    fn as_ref(&self) -> &BaseWithLifetime<'a> {
        &self.base
    }
}
