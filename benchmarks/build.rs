use std::{env, path::PathBuf};
use alloc_stress::AllocStressProgram;
use compute_stress::ComputeStressProgram;
use counter_bench::CounterBenchProgram;
use convert_case::{Case, Casing};

macro_rules! build_client {
    ($program:ty, $clients_path:expr, $idls_path:expr) => {
        let program_name = stringify!($program)
            .to_case(Case::Snake);
        let client_path = $clients_path
            .clone()
            .join(&program_name)
            .with_extension("rs");
        let idl_path = $idls_path
            .clone()
            .join(&program_name)
            .with_extension("idl");
        sails_rs::ClientBuilder::<$program>::empty(program_name)
            .with_idl_path(idl_path)
            .with_client_path(client_path)
            .build_idl()
            .generate()
            .unwrap();
    };
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap();
    let clients_path = PathBuf::from(&manifest_dir).join("src");
    let idls_path = PathBuf::from(&manifest_dir).join("idls");

    // Generate IDL file for the `AllocStress` app and client code from IDL file
    build_client!(AllocStressProgram, clients_path, idls_path);

    // Generate IDL file for the `ComputeStress` app and client code from IDL file
    build_client!(ComputeStressProgram, clients_path, idls_path);

    // Generate IDL file for the `CounterBench` app and client code from IDL file
    build_client!(CounterBenchProgram, clients_path, idls_path);
}