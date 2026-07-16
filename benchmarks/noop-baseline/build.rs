fn main() {
    // Build WASM
    if let Some((_, wasm_path)) = sails::build_wasm() {
        // Generate IDL and embed it into WASM
        sails::ClientBuilder::<::noop_baseline_app::Program>::from_wasm_path(wasm_path).build_idl();
    }
}
