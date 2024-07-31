use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use root_generator::RootGenerator;
use sails_idl_parser::ast::visitor;
use std::{ffi::OsStr, fs, io::Write, path::Path};

mod ctor_generators;
mod events_generator;
mod helpers;
mod io_generators;
mod mock_generator;
mod root_generator;
mod service_generators;
mod type_generators;

const SAILS: &str = "sails_rs";

pub struct IdlPath<'a>(&'a Path);
pub struct IdlString<'a>(&'a str);
pub struct ClientGenerator<'a, S> {
    sails_path: Option<&'a str>,
    mocks_feature_name: Option<&'a str>,
    idl: S,
}

impl<'a, S> ClientGenerator<'a, S> {
    pub fn with_mocks(self, mocks_feature_name: &'a str) -> Self {
        Self {
            mocks_feature_name: Some(mocks_feature_name),
            ..self
        }
    }

    pub fn with_sails_crate(self, sails_path: &'a str) -> Self {
        Self {
            sails_path: Some(sails_path),
            ..self
        }
    }
}

impl<'a> ClientGenerator<'a, IdlPath<'a>> {
    pub fn from_idl_path(idl_path: &'a Path) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            idl: IdlPath(idl_path),
        }
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let idl_path = self.idl.0;

        let idl = fs::read_to_string(idl_path)
            .with_context(|| format!("Failed to open {} for reading", idl_path.display()))?;

        let file_name = idl_path.file_stem().unwrap_or(OsStr::new("service"));
        let service_name = file_name.to_string_lossy().to_case(Case::Pascal);

        self.with_idl(&idl)
            .generate_to(&service_name, out_path)
            .context("failed to generate client")?;
        Ok(())
    }

    fn with_idl(self, idl: &'a str) -> ClientGenerator<'a, IdlString<'a>> {
        ClientGenerator {
            sails_path: self.sails_path,
            mocks_feature_name: self.mocks_feature_name,
            idl: IdlString(idl),
        }
    }
}

impl<'a> ClientGenerator<'a, IdlString<'a>> {
    pub fn from_idl(idl: &'a str) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            idl: IdlString(idl),
        }
    }

    pub fn generate(&self, anonymous_service_name: &str) -> Result<String> {
        let idl = self.idl.0;
        let sails_path = self.sails_path.unwrap_or(SAILS);
        let mut generator =
            RootGenerator::new(anonymous_service_name, self.mocks_feature_name, sails_path);
        let program = sails_idl_parser::ast::parse_idl(idl).context("Failed to parse IDL")?;
        visitor::accept_program(&program, &mut generator);

        let code = generator.finalize();

        // Check for parsing errors
        let code = pretty_with_rustfmt(&code);

        Ok(code)
    }

    pub fn generate_to(
        self,
        anonymous_service_name: &str,
        out_path: impl AsRef<Path>,
    ) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self
            .generate(anonymous_service_name)
            .context("failed to generate client")?;

        fs::write(out_path, code).with_context(|| {
            format!("Failed to write generated client to {}", out_path.display())
        })?;

        Ok(())
    }
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
