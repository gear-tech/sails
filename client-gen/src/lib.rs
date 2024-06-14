mod ctor_generators;
mod helpers;
mod io_generators;
mod root_generator;
mod service_generators;
mod type_generators;

use root_generator::RootGenerator;

use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use sails_idl_parser::ast::{visitor, Program};
use std::{ffi::OsStr, fs, io::Write, path::Path};

pub fn generate_client_from_idl(
    idl_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
) -> Result<()> {
    let idl_path = idl_path.as_ref();
    let out_path = out_path.as_ref();

    let idl = fs::read_to_string(idl_path)
        .with_context(|| format!("Failed to open {} for reading", idl_path.display()))?;

    let program = match sails_idl_parser::ast::parse_idl(&idl) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Failed to parse IDL: {}", e);
            std::process::exit(1);
        }
    };

    let file_name = idl_path.file_stem().unwrap_or(OsStr::new("service"));
    let service_name = file_name.to_string_lossy().to_case(Case::Pascal);

    let buf = generate(program, &service_name).context("failed to generate client")?;

    fs::write(out_path, buf)
        .with_context(|| format!("Failed to write generated client to {}", out_path.display()))?;

    Ok(())
}

pub fn generate(program: Program, default_service_name: &str) -> Result<String> {
    let mut generator = RootGenerator::new(default_service_name);
    visitor::accept_program(&program, &mut generator);

    let code = generator.finalize();

    // Check for parsing errors
    let code = pretty_with_rustfmt(&code);

    Ok(code)
}

// not using prettyplease since it's bad at reporting syntax errors and also removes comments
// TODO(holykol): Fallback if rustfmt is not in PATH would be nice
fn pretty_with_rustfmt(code: &str) -> String {
    use std::process::Command;
    let mut child = Command::new("rustfmt")
        .arg("--config")
        .arg("format_strings=false")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt");

    let child_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    child_stdin
        .write_all(code.as_bytes())
        .expect("Failed to write to rustfmt");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for rustfmt");

    if !output.status.success() {
        panic!(
            "rustfmt failed with status: {}\n{}",
            output.status,
            String::from_utf8(output.stderr).expect("Failed to read rustfmt stderr")
        );
    }

    String::from_utf8(output.stdout).expect("Failed to read rustfmt output")
}
