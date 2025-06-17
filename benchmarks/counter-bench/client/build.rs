use counter_bench_app::CounterBenchProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("counter_bench.idl");

    // Generate IDL file for the `CounterBench` app
    sails_idl_gen::generate_idl_to_file::<CounterBenchProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/counter_bench_client.rs"),
        )
        .unwrap();
}
