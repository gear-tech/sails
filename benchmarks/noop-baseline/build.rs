fn main() {
    // Build WASM
    if let Some((_, wasm_path)) = sails_rs::build_wasm() {
        // Generate IDL and embed it into WASM
        sails_rs::ClientBuilder::<::noop_baseline_app::Program>::from_wasm_path(wasm_path)
            .build_idl();
    }
}
