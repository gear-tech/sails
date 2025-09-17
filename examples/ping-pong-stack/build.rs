fn main() {
    sails_rs::build_wasm();

    let base_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let idl_path = base_path.join("ping_pong_stack.idl");
    let out_path = base_path.join("src/ping_pong_stack.rs");
    sails_rs::ClientGenerator::from_idl_path(&idl_path)
        .generate_to(out_path)
        .unwrap();
}
