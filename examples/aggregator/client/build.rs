fn main() {
    sails_rs::ClientBuilder::<aggregator_app::AggregatorProgram>::from_env()
        .build_idl()
        .generate()
        .unwrap();
}
