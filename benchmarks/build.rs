use convert_case::{Case, Casing};
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let clients_path = PathBuf::from(&manifest_dir).join("src");
    let idls_path = PathBuf::from(&manifest_dir).join("idls");

    // Generate client code for the `AllocStress` app from IDL file
    let program_name = "alloc_stress_program".to_case(Case::Snake);
    let client_path = clients_path
        .clone()
        .join(&program_name)
        .with_extension("rs");
    let idl_path = idls_path.clone().join(&program_name).with_extension("idl");
    ClientGenerator::from_idl_path(&idl_path)
        .generate_to(client_path)
        .unwrap();

    // Generate client code for the `ComputeStress` app from IDL file
    let program_name = "compute_stress_program".to_case(Case::Snake);
    let client_path = clients_path
        .clone()
        .join(&program_name)
        .with_extension("rs");
    let idl_path = idls_path.clone().join(&program_name).with_extension("idl");
    ClientGenerator::from_idl_path(&idl_path)
        .generate_to(client_path)
        .unwrap();

    // Generate client code for the `CounterBench` app from IDL file
    let program_name = "counter_bench_program".to_case(Case::Snake);
    let client_path = clients_path
        .clone()
        .join(&program_name)
        .with_extension("rs");
    let idl_path = idls_path.clone().join(&program_name).with_extension("idl");
    ClientGenerator::from_idl_path(&idl_path)
        .generate_to(client_path)
        .unwrap();
}
