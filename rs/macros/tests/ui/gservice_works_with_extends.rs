use sails_rs::{
    Decode, Encode,
    gstd::{service, services::Service},
};

mod base {
    use super::*;

    pub const BASE_NAME_RESULT: &str = "base-name";
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

mod extended {
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

    impl AsRef<base::Base> for Extended {
        fn as_ref(&self) -> &base::Base {
            &self.base
        }
    }
}

#[tokio::main]
async fn main() {
    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";

    let mut extended_svc = extended::Extended::new(base::Base).expose(123.into(), &[1, 2, 3]);

    let output = extended_svc.handle(&EXTENDED_NAME_METHOD.encode()).await;

    assert_eq!(
        output,
        [
            EXTENDED_NAME_METHOD.encode(),
            extended::EXTENDED_NAME_RESULT.encode()
        ]
        .concat()
    );

    let _base: &<base::Base as Service>::Exposure = extended_svc.as_ref();

    let output = extended_svc.handle(&BASE_NAME_METHOD.encode()).await;
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, BASE_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, base::BASE_NAME_RESULT);

    let output = extended_svc.handle(&EXTENDED_NAME_METHOD.encode()).await;
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, EXTENDED_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, extended::EXTENDED_NAME_RESULT);

    let output = extended_svc.handle(&NAME_METHOD.encode()).await;
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, extended::NAME_RESULT);
}
