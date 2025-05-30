use convert_case::{Case, Casing};
use core::marker::PhantomData;
use sails_client_gen::ClientGenerator;
use sails_idl_meta::ProgramMeta;
use std::{
    env,
    path::{Path, PathBuf},
    string::{String, ToString},
    vec::Vec,
};

/// Shorthand function to be used in `build.rs`.
///
/// See [Builder::build()].
///
/// Code
/// ```rust
/// use std::{env, path::PathBuf};
///
/// let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
/// let package_name = env::var("CARGO_PKG_NAME").unwrap();
/// let program_name = package_name.to_case(Case::Snake);
/// let idl_path = PathBuf::from(&out_dir).join(&program_name).with_extension("idl");
/// let client_path = PathBuf::from(&out_dir).join("src").join(&program_name).with_extension("rs");
///
/// // Generate IDL file for the program
/// sails_rs::generate_idl_to_file::<redirect_app::RedirectProgram>(&idl_path).unwrap();
/// // Generate client code from IDL file
/// ClientGenerator::from_idl_path(&idl_path).generate_to(&client_path).unwrap();
/// ```
pub fn build_client<P: ProgramMeta>() -> (Option<PathBuf>, Option<PathBuf>) {
    Builder::<P>::from_env().build()
}

/// Shorthand function to be used in `build.rs`.
///
/// Generates client to `lib.rs` file, and appends `#![no_std]`
///
/// See [build_client()], [Builder::build()].
pub fn build_client_as_lib<P: ProgramMeta>() -> (Option<PathBuf>, Option<PathBuf>) {
    Builder::<P>::from_env().no_std().build()
}

/// Program IDL and client builder.
///
/// This struct uses `sails-idl-gen` package to generate IDL,
/// and `sails-client-gen` package to generate Rust client code.
#[derive(Debug, Clone)]
pub struct Builder<P> {
    idl_path: Option<PathBuf>,
    client_path: Option<PathBuf>,
    program_name: String,
    no_std: bool,
    _marker: PhantomData<P>,
}

impl<P: ProgramMeta> Default for Builder<P> {
    fn default() -> Self {
        Self::from_env()
    }
}

impl<P: ProgramMeta> Builder<P> {
    pub fn from_env() -> Self {
        let out_dir =
            env::var("CARGO_MANIFEST_DIR").expect("Default builder can only be used in crate");
        let package_name =
            env::var("CARGO_PKG_NAME").expect("Default builder can only be used in crate");
        let program_name = package_name.to_case(Case::Snake);

        Self {
            idl_path: Some(
                PathBuf::from(&out_dir)
                    .join(&program_name)
                    .with_extension("idl"),
            ),
            client_path: Some(
                PathBuf::from(&out_dir)
                    .join("src")
                    .join(&program_name)
                    .with_extension("rs"),
            ),
            program_name,
            no_std: false,
            _marker: Default::default(),
        }
    }

    pub fn empty(program_name: String) -> Self {
        Self {
            idl_path: None,
            client_path: None,
            program_name,
            no_std: false,
            _marker: Default::default(),
        }
    }

    pub fn with_idl_path<T: AsRef<Path>>(self, path: T) -> Self {
        Self {
            idl_path: Some(PathBuf::from(path.as_ref())),
            ..self
        }
    }

    pub fn with_client_path<T: AsRef<Path>>(self, path: T) -> Self {
        Self {
            client_path: Some(PathBuf::from(path.as_ref())),
            ..self
        }
    }

    pub fn with_program_name(self, program_name: &str) -> Self {
        Self {
            program_name: program_name.to_string(),
            ..self
        }
    }

    /// Generates client to `lib.rs` file, and appends `#![no_std]`
    pub fn no_std(self) -> Self {
        let Self { client_path, .. } = self;
        Self {
            client_path: client_path.map(|p| p.with_file_name("lib.rs")),
            no_std: true,
            ..self
        }
    }

    /// Build the program IDL and generate client code.
    ///
    /// Returns `(Option<PathBuf>, Option<PathBuf>)` where
    /// - first `Option<PathBuf>` is path to the IDL file if generated.
    /// - second `Option<PathBuf>` is path to the client file if generated.
    pub fn build(self) -> (Option<PathBuf>, Option<PathBuf>) {
        if let Some(idl_path) = self.idl_path.as_ref() {
            // Generate IDL file for the program
            sails_idl_gen::generate_idl_to_file::<P>(idl_path.as_path())
                .expect("Error generating IDL from program");

            if let Some(client_path) = self.client_path.as_ref() {
                // Generate client code from IDL file
                ClientGenerator::from_idl_path(idl_path.as_path())
                    .with_no_std(self.no_std)
                    .generate_to(client_path.as_path())
                    .expect("Error generating client from IDL");
            }
        } else if let Some(client_path) = self.client_path.as_ref() {
            // Generate IDL string for the program
            let mut idl = Vec::new();
            sails_idl_gen::generate_idl::<P>(&mut idl).expect("Error generating IDL from program");
            let idl = String::from_utf8(idl).unwrap();

            // Generate client code from IDL string
            ClientGenerator::from_idl(&idl)
                .with_no_std(self.no_std)
                .generate_to(
                    self.program_name.to_case(Case::Pascal).as_str(),
                    client_path.as_path(),
                )
                .expect("Error generating client from IDL");
        }

        (self.idl_path, self.client_path)
    }
}

#[cfg(test)]
mod tests {
    use gstd::TypeInfo;

    use super::*;

    struct P;

    #[derive(TypeInfo)]
    #[scale_info(crate = crate::scale_info)]
    struct Meta;

    impl ProgramMeta for P {
        type ConstructorsMeta = Meta;
        const SERVICES: &'static [(&'static str, sails_idl_meta::AnyServiceMetaFn)] = &[];
    }

    #[test]
    fn builder_new() {
        let Builder {
            client_path,
            idl_path,
            program_name,
            ..
        } = Builder::<P>::from_env();

        assert_eq!("sails_rs", program_name);
        assert!(client_path.is_some());
        assert!(idl_path.is_some());
        assert!(client_path.unwrap().ends_with("src/sails_rs.rs"));
        assert!(idl_path.unwrap().ends_with("sails_rs.idl"));
    }
}
