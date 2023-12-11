fn main() {
    gwasm_builder::build_with_metadata::<this_that_app::ProgramMetadata>();
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let wasm_target_dir = wasm_target_dir(&out_dir);
    // TODO: This path could be generated based on the path to the generated wasm file, but it
    //       needs to be returned from the `gwasm_builder::build...` methods.
    let idl_path = wasm_target_dir.join("this_that.sails.idl");
    let idl_file = std::fs::File::create(idl_path).expect("failed to create IDL file");
    sails_idlgen::generate_serivce_idl::<
        this_that_app::CommandProcessorMeta,
        this_that_app::QueryProcessorMeta,
    >(None, idl_file)
    .expect("failed to write IDL file");
}

// TODO: This code is copy-pasted from the wasm-build. It would be nice if `wasm_builder::build...`
//       methods returned a path to generated wasm file so it could be used for generating a path
//       to the IDL file for the cases when IDL generator is used as a part of build script.
fn wasm_target_dir(out_dir: &std::path::Path) -> std::path::PathBuf {
    let profile: String = out_dir
        .components()
        .rev()
        .take_while(|c| c.as_os_str() != "target")
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take_while(|c| c.as_os_str() != "build")
        .last()
        .expect("Path should have subdirs in the `target` dir")
        .as_os_str()
        .to_string_lossy()
        .into();

    let mut target_dir = out_dir.to_path_buf();
    while !target_dir.ends_with("target") && target_dir.pop() {}

    target_dir.join("wasm32-unknown-unknown").join(profile)
}
