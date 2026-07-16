fn main() {
    sails::ClientBuilder::<inspector_app::InspectorProgram>::from_env()
        .build_idl()
        .generate()
        .unwrap();
}
