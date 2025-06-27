use sails_rs::prelude::*;

pub(super) mod base {
    use super::*;

    pub const BASE_NAME_RESULT: &str = "base-name";
    #[allow(dead_code)]
    pub const NAME_RESULT: &str = "base";

    #[derive(Clone)]
    pub struct Base;

    #[service]
    impl Base {
        #[export]
        pub fn base_name(&self) -> String {
            "base-name".to_string()
        }

        #[export]
        pub fn name(&self) -> String {
            "base".to_string()
        }
    }
}

pub(super) mod extended {
    use super::*;

    pub const EXTENDED_NAME_RESULT: &str = "extended-name";
    pub const NAME_RESULT: &str = "extended";

    pub struct Extended {
        base: base::Base,
    }

    impl Extended {
        pub fn new(base: base::Base) -> Self {
            Self { base }
        }
    }

    #[service(extends = base::Base)]
    impl Extended {
        #[export]
        pub fn extended_name(&self) -> String {
            "extended-name".to_string()
        }

        #[export]
        pub fn name(&self) -> String {
            "extended".to_string()
        }
    }

    #[allow(clippy::from_over_into)]
    impl Into<base::Base> for Extended {
        fn into(self) -> base::Base {
            self.base
        }
    }
}

pub(super) mod extended_pure {
    use super::*;

    pub struct ExtendedPure {
        base: base::Base,
    }

    impl ExtendedPure {
        pub fn new(base: base::Base) -> Self {
            Self { base }
        }
    }

    #[service(extends = base::Base)]
    impl ExtendedPure {}

    #[allow(clippy::from_over_into)]
    impl Into<base::Base> for ExtendedPure {
        fn into(self) -> base::Base {
            self.base
        }
    }
}
