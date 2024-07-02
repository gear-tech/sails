use sails_rtl::gstd::gservice;

pub(super) mod base {
    use super::*;

    pub const BASE_NAME_RESULT: &str = "base-name";
    #[allow(dead_code)]
    pub const NAME_RESULT: &str = "base";

    #[derive(Clone)]
    pub struct BaseLifetime<'a> {
        _int: &'a u64,
    }

    impl<'a> BaseLifetime<'a> {
        pub fn new(int: &'a u64) -> Self {
            BaseLifetime { _int: int }
        }
    }

    #[gservice]
    impl<'a> BaseLifetime<'a> {
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

    pub struct ExtendedLifetime<'a> {
        base: base::BaseLifetime<'a>,
    }

    impl<'a> ExtendedLifetime<'a> {
        pub fn new(base: base::BaseLifetime<'a>) -> Self {
            Self { base }
        }
    }

    #[gservice(extends = base::BaseLifetime<'a>)]
    impl<'a> ExtendedLifetime<'a> {
        pub fn extended_name(&self) -> String {
            "extended-name".to_string()
        }

        pub fn name(&self) -> String {
            "extended".to_string()
        }
    }

    impl<'a> AsRef<base::BaseLifetime<'a>> for ExtendedLifetime<'a> {
        fn as_ref(&self) -> &base::BaseLifetime<'a> {
            &self.base
        }
    }
}
