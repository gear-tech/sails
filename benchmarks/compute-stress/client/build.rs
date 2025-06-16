use compute_stress_app::ComputeStressProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("compute_stress.idl");

    // Generate IDL file for the `FiboStress` app
    sails_idl_gen::generate_idl_to_file::<ComputeStressProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/compute_stress_client.rs"),
        )
        .unwrap();
}
