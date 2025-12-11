fn main() {
    sails_rs::build_wasm();

    sails_rs::build_client::<no_svcs_prog_app::Program>();
}
