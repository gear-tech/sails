pub mod generator;

use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use std::{ffi::OsStr, fs, path::Path};

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

    let buf = generator::generate(program, &service_name).context("failed to generate client")?;

    fs::write(out_path, buf)
        .with_context(|| format!("Failed to write generated client to {}", out_path.display()))?;

    Ok(())
}
