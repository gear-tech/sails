use alloc_stress_app::AllocStressProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("alloc_stress.idl");

    // Generate IDL file for the `AllocStress` app
    sails_idl_gen::generate_idl_to_file::<AllocStressProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/alloc_stress_client.rs"),
        )
        .unwrap();
}
