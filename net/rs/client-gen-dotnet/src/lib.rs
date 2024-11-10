use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use root_generator::RootGenerator;
use sails_idl_parser::ast::visitor;
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
    sails_path: Option<&'a str>,
    mocks_feature_name: Option<&'a str>,
    external_types: HashMap<&'a str, &'a str>,
    no_derive_traits: bool,
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

    /// Add an map from IDL type to crate path
    ///
    /// Instead of generating type in client code, use type path from external crate
    ///
    /// # Example
    ///
    /// Following code generates `use my_crate::MyParam as MyFuncParam;`
    /// ```
    /// let code = sails_client_gen_dotnet::ClientGenerator::from_idl("<idl>")
    ///     .with_external_type("MyFuncParam", "my_crate::MyParam");
    /// ```
    pub fn with_external_type(self, name: &'a str, path: &'a str) -> Self {
        let mut external_types = self.external_types;
        external_types.insert(name, path);
        Self {
            external_types,
            ..self
        }
    }
}

impl<'a> ClientGenerator<'a, IdlPath<'a>> {
    pub fn from_idl_path(idl_path: &'a Path) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            external_types: HashMap::new(),
            no_derive_traits: false,
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
            external_types: self.external_types,
            no_derive_traits: self.no_derive_traits,
            idl: IdlString(idl),
        }
    }
}

impl<'a> ClientGenerator<'a, IdlString<'a>> {
    pub fn from_idl(idl: &'a str) -> Self {
        Self {
            sails_path: None,
            mocks_feature_name: None,
            external_types: HashMap::new(),
            no_derive_traits: false,
            idl: IdlString(idl),
        }
    }

    pub fn generate(self, anonymous_service_name: &str) -> Result<String> {
        let idl = self.idl.0;
        let program = sails_idl_parser::ast::parse_idl(idl).context("Failed to parse IDL")?;

        let mut generator = RootGenerator::new(anonymous_service_name, self.external_types);
        visitor::accept_program(&program, &mut generator);
        let tokens = generator.finalize();

        let fmt = genco::fmt::Config::from_lang::<genco::lang::Csharp>()
            .with_indentation(genco::fmt::Indentation::Space(4));
        let config = genco::lang::csharp::Config::default();
        let mut w = genco::fmt::FmtWriter::new(String::new());

        tokens.format_file(&mut w.as_formatter(&fmt), &config)?;

        Ok(w.into_inner())
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

/// # Safety
///
/// Function [`free_c_string`] should be called after this function
#[no_mangle]
pub unsafe extern "C" fn generate_dotnet_client(
    program_utf8: *const u8,
    program_len: i32,
    service_name_utf8: *const u8,
    service_name_len: i32,
) -> *const std::ffi::c_char {
    let slice = unsafe { std::slice::from_raw_parts(program_utf8, program_len as usize) };
    let program = unsafe { String::from_utf8_unchecked(slice.to_vec()) };
    let slice = unsafe { std::slice::from_raw_parts(service_name_utf8, service_name_len as usize) };
    let service_name = unsafe { String::from_utf8_unchecked(slice.to_vec()) };

    let res = ClientGenerator::from_idl(program.as_str())
        .generate(service_name.as_str())
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
