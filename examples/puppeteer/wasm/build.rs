use std::path::Path;

fn main() {
    gwasm_builder::build();

    let idl_path = Path::new("../../this-that-svc/wasm/this-that-svc.idl");
    let out_path = Path::new("../app/src/puppet.rs");
    sails_clientgen::generate_client_from_idl(idl_path, out_path).unwrap();
}
