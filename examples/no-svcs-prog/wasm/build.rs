fn main() {
    sails::build_wasm();

    sails::build_client::<no_svcs_prog_app::Program>();
}
