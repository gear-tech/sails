fn main() {
    sails_rs::ClientBuilder::<inspector_app::InspectorProgram>::from_env()
        .build_idl()
        .generate()
        .unwrap();
}
