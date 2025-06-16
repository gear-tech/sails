use alloc_stress_app::AllocStressProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("alloc_stress.idl");

    // Generate IDL file for the `FiboStress` app
    sails_idl_gen::generate_idl_to_file::<AllocStressProgram>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .with_mocks("with_mocks")
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/alloc_stress_client.rs"),
        )
        .unwrap();
}

/*
{
  "compute": 0,
  "alloc": {
    "0": "1201479547",
    "12": "1207893139",
    "143": "1232282629",
    "986": "1381148295",
    "10945": "3131353243",
    "46367": "9717601871",
    "121392": "24710048191",
    "317810": "62890659634"
  }
}

*/
