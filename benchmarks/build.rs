use convert_case::{Case, Casing};
use sails_client_gen_v2::ClientGenerator;
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

fn generate_idl_and_client<P: sails_rs::meta::ProgramMeta>(
    program_name_str: &str,
    clients_path: &std::path::Path,
    idls_path: &std::path::Path,
) {
    let program_name = program_name_str.to_case(Case::Snake);
    let client_path = clients_path.join(&program_name).with_extension("rs");
    let idl_path = idls_path.join(&program_name).with_extension("idl");
    let program_name_pascal = program_name_str
        .strip_suffix("_program")
        .unwrap_or(program_name_str)
        .to_case(Case::Pascal);

    sails_idl_gen::generate_idl_to_file::<P>(Some(&program_name_pascal), &idl_path).unwrap();
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
    generate_idl_and_client::<storage_stress::StorageStressProgram>(
        "storage_stress_program",
        &clients_path,
        &idls_path,
    );
    generate_idl_and_client::<vft_stress::VftStressProgram>(
        "vft_stress_program",
        &clients_path,
        &idls_path,
    );
    generate_idl_and_client::<storage_million::StorageMillionProgram>(
        "storage_million_program",
        &clients_path,
        &idls_path,
    );
}
