mod generator;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full() {
        const IDL: &str = r#"
        type ThisThatSvcAppTupleStruct = struct {
            bool,
        };

        type ThisThatSvcAppDoThatParam = struct {
            p1: u32,
            p2: str,
            p3: ThisThatSvcAppManyVariants,
        };

        type ThisThatSvcAppManyVariants = enum {
            One,
            Two: u32,
            Three: opt u32,
            Four: struct { a: u32, b: opt u16 },
            Five: struct { str, u32 },
            Six: struct { u32 },
        };

        type T = enum { One };

        constructor {
            New : (a: u32);
        };

        service {
            DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 };
            DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str });
            query This : (v1: vec u16) -> u32;
            query That : (v1: null) -> result (str, str);
        };
        "#;
        let program = sails_idl_parser::ast::parse_idl(IDL).expect("parse IDL");

        let generated = generator::generate(program, "Service").unwrap();

        dbg!(&generated);

        insta::assert_snapshot!(generated);
    }
}
