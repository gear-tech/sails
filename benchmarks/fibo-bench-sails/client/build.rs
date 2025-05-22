use fibo_bench_sails_app::FiboStressProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("fibo_bench_sails.idl");

    // Generate IDL file for the `FiboStress` app
    sails_idl_gen::generate_idl_to_file::<FiboStressProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/fibo_bench_sails_client.rs"),
        )
        .unwrap();
}
