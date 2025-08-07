use std::env;

fn main() {
    if let Ok(target_arch) = env::var("CARGO_CFG_TARGET_ARCH")
        && target_arch == "wasm32"
    {
        println!("cargo:rustc-link-arg=--import-memory");
    }

    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .emit_rerun_directives(true)
        .process_file("src/grammar.lalrpop")
        .unwrap();
}
