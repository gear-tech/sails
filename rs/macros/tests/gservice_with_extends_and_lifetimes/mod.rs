use sails_rs::gstd::gservice;

pub(super) mod base {
    use super::*;

    pub const BASE_NAME_RESULT: &str = "base-name";
    #[allow(dead_code)]
    pub const NAME_RESULT: &str = "base";

    #[derive(Clone)]
    pub struct BaseWithLifetime<'a> {
        _int: &'a u64,
    }

    impl<'a> BaseWithLifetime<'a> {
        pub fn new(int: &'a u64) -> Self {
            BaseWithLifetime { _int: int }
        }
    }

    #[gservice]
    impl<'a> BaseWithLifetime<'a> {
        pub fn base_name(&self) -> String {
            "base-name".to_string()
        }

        pub fn name(&self) -> String {
            "base".to_string()
        }
    }
}

pub(super) mod extended {
    use super::*;

    pub const EXTENDED_NAME_RESULT: &str = "extended-name";
    pub const NAME_RESULT: &str = "extended";

    pub struct ExtendedWithLifetime<'a> {
        base: base::BaseWithLifetime<'a>,
    }

    impl<'a> ExtendedWithLifetime<'a> {
        pub fn new(base: base::BaseWithLifetime<'a>) -> Self {
            Self { base }
        }
    }

    #[gservice(extends = base::BaseWithLifetime<'a>)]
    impl<'a> ExtendedWithLifetime<'a> {
        pub fn extended_name(&self) -> String {
            "extended-name".to_string()
        }

        pub fn name(&self) -> String {
            "extended".to_string()
        }
    }

    impl<'a> AsRef<base::BaseWithLifetime<'a>> for ExtendedWithLifetime<'a> {
        fn as_ref(&self) -> &base::BaseWithLifetime<'a> {
            &self.base
        }
    }
}
