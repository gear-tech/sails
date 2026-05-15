fn main() {
    // Generate IDL file for the `Demo` app and client code from IDL file
    sails_rs::ClientBuilder::<demo::DemoProgram>::from_env()
        .build_idl()
        .with_mocks("with_mocks")
        .with_external_type("NonZeroU8", "core::num::NonZeroU8")
        .with_external_type("NonZeroU32", "core::num::NonZeroU32")
        .generate()
        .unwrap();
}
