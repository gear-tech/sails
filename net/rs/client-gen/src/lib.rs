use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use root_generator::RootGenerator;
use sails_idl_parser::ast::visitor;
use serde::Deserialize;
use std::{collections::HashMap, ffi::OsStr, fs, path::Path};

mod ctor_generators;
mod events_generator;
mod helpers;
mod root_generator;
mod service_generators;
mod type_generators;

pub struct IdlPath<'a>(&'a Path);

pub struct IdlString<'a>(&'a str);

pub struct ClientGenerator<'a, S> {
    external_types: HashMap<&'a str, &'a str>,
    idl: S,
}

impl<'a> ClientGenerator<'a, IdlPath<'a>> {
    pub fn from_idl_path(idl_path: &'a Path) -> Self {
        Self {
            external_types: HashMap::new(),
            idl: IdlPath(idl_path),
        }
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let idl_path = self.idl.0;

        let idl = fs::read_to_string(idl_path)
            .with_context(|| format!("Failed to open {} for reading", idl_path.display()))?;

        let file_name = idl_path.file_stem().unwrap_or(OsStr::new("service"));
        let service_name = file_name.to_string_lossy().to_case(Case::Pascal);
        let namepace = format!("{}.Client", service_name);

        self.with_idl(&idl)
            .generate_to(&service_name, &namepace, out_path)
            .context("failed to generate client")?;
        Ok(())
    }

    fn with_idl(self, idl: &'a str) -> ClientGenerator<'a, IdlString<'a>> {
        ClientGenerator {
            external_types: self.external_types,
            idl: IdlString(idl),
        }
    }
}

impl<'a> ClientGenerator<'a, IdlString<'a>> {
    pub fn from_idl(idl: &'a str) -> Self {
        Self {
            external_types: HashMap::new(),
            idl: IdlString(idl),
        }
    }

    pub fn generate(self, anonymous_service_name: &str, namespace: &str) -> Result<String> {
        let idl = self.idl.0;
        let program = sails_idl_parser::ast::parse_idl(idl).context("Failed to parse IDL")?;

        let mut generator =
            RootGenerator::new(anonymous_service_name, namespace, self.external_types);
        visitor::accept_program(&program, &mut generator);
        let tokens = generator.finalize();
        let code = tokens.to_file_string()?;
        Ok(code)
    }

    pub fn generate_to(
        self,
        anonymous_service_name: &str,
        namespace: &str,
        out_path: impl AsRef<Path>,
    ) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self
            .generate(anonymous_service_name, namespace)
            .context("failed to generate client")?;

        fs::write(out_path, code).with_context(|| {
            format!("Failed to write generated client to {}", out_path.display())
        })?;

        Ok(())
    }
}

/// # Safety
///
/// Function [`free_c_string`] should be called after this function
#[no_mangle]
pub unsafe extern "C" fn generate_dotnet_client(
    program_utf8: *const u8,
    program_len: i32,
    config_utf8: *const u8,
    config_len: i32,
) -> *const std::ffi::c_char {
    let slice = unsafe { std::slice::from_raw_parts(program_utf8, program_len as usize) };
    let program = unsafe { String::from_utf8_unchecked(slice.to_vec()) };
    let slice = unsafe { std::slice::from_raw_parts(config_utf8, config_len as usize) };
    let config: GeneratorConfig = serde_json::from_slice(slice).expect("failed to parse config");

    let res = ClientGenerator::from_idl(program.as_str())
        .generate(&config.service_name, &config.namespace)
        .expect("failed to generate client");
    std::ffi::CString::new(res)
        .expect("failed to create cstring")
        .into_raw()
}

/// # Safety
///
/// This function should not be called before the [`generate_dotnet_client`]
#[no_mangle]
pub unsafe extern "C" fn free_c_string(str: *mut std::ffi::c_char) {
    // drop
    _ = unsafe { std::ffi::CString::from_raw(str) };
}

#[derive(Deserialize, Debug)]
struct GeneratorConfig {
    service_name: String,
    namespace: String,
}
