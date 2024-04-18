use sails_idl_meta::{ProgramMeta, ServiceMeta};
use std::{env, fs::File, path::PathBuf};

pub struct Builder {
    idl_file_path: PathBuf,
}

impl Builder {
    pub fn new() -> Self {
        let pkg_name = env::var("CARGO_PKG_NAME").unwrap();
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let idl_file_path = PathBuf::from(manifest_dir).join(format!("{}.idl", pkg_name));

        Self { idl_file_path }
    }

    pub fn build(self) -> Self {
        gwasm_builder::build();

        self
    }

    pub fn generate_service_idl<S: ServiceMeta>(self) -> Self {
        let idl_file = File::create(&self.idl_file_path).unwrap();
        sails_idlgen::service::generate_idl::<S>(idl_file).unwrap();

        self
    }

    pub fn generate_program_idl<P: ProgramMeta>(self) -> Self {
        let idl_file = File::create(&self.idl_file_path).unwrap();
        sails_idlgen::program::generate_idl::<P>(idl_file).unwrap();

        self
    }
}
