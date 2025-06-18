fn main() {
    sails_rs::build_wasm();

    sails_rs::build_client::<rmrk_resource_app::Program>();
}
