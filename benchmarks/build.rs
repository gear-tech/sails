use convert_case::{Case, Casing};
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn generate_client(
    program_name_str: &str,
    clients_path: &std::path::Path,
    idls_path: &std::path::Path,
) {
    let program_name = program_name_str.to_case(Case::Snake);
    let client_path = clients_path.join(&program_name).with_extension("rs");
    let idl_path = idls_path.join(&program_name).with_extension("idl");
    ClientGenerator::from_idl_path(&idl_path)
        .generate_to(client_path)
        .unwrap();
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let clients_path = PathBuf::from(&manifest_dir).join("src");
    let idls_path = PathBuf::from(&manifest_dir).join("idls");

    generate_client("alloc_stress_program", &clients_path, &idls_path);
    generate_client("compute_stress_program", &clients_path, &idls_path);
    generate_client("counter_bench_program", &clients_path, &idls_path);
}
