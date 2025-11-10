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

    impl From<Extended> for base::Base {
        fn from(value: Extended) -> Self {
            value.base
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

    impl From<ExtendedPure> for base::Base {
        fn from(value: ExtendedPure) -> Self {
            value.base
        }
    }
}

pub(super) mod extended_renamed {
    use super::{
        *,
        base::Base as RenamedBase
    };

    pub struct ExtendedRenamed {
        base: (RenamedBase, other_base::Base),
    }

    impl ExtendedRenamed {
        pub fn new(base: (RenamedBase, other_base::Base)) -> Self {
            Self { base }
        }
    }

    #[service(extends = [RenamedBase, other_base::Base])]
    impl ExtendedRenamed {}

    impl From<ExtendedRenamed> for (RenamedBase, other_base::Base) {
        fn from(value: ExtendedRenamed) -> Self {
            (value.base.0, value.base.1)
        }
    }
}

pub(super) mod other_base {
    use super::*;

    pub struct Base;

    #[service]
    impl Base {
        #[export]
        pub fn other_base_name(&self) -> String {
            "other-base-name".to_string()
        }
    }
}
