fn main() {
    sails::ClientBuilder::<aggregator_app::AggregatorProgram>::from_env()
        .build_idl()
        .generate()
        .unwrap();
}
