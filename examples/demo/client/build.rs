fn main() {
    // Generate IDL file for the `Demo` app and client code from IDL file
    sails_rs::Builder::<demo::DemoProgram>::from_env()
        .build_idl()
        .with_mocks("with_mocks")
        .generate()
        .unwrap();
}
