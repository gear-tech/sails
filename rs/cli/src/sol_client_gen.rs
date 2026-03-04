use anyhow::Result;
use sails_sol_client_gen::ClientGenerator;
use std::path::PathBuf;

pub struct SolClientGenerator {
    idl_path: PathBuf,
    out_path: Option<PathBuf>,
    sails_crate: Option<String>,
    external_types: Vec<(String, String)>,
    no_derive_traits: bool,
}

impl SolClientGenerator {
    pub fn new(
        idl_path: PathBuf,
        out_path: Option<PathBuf>,
        sails_crate: Option<String>,
        external_types: Vec<(String, String)>,
        no_derive_traits: bool,
    ) -> Self {
        Self {
            idl_path,
            out_path,
            sails_crate,
            external_types,
            no_derive_traits,
        }
    }

    pub fn generate(self) -> Result<()> {
        let mut client_gen = ClientGenerator::from_idl_path(self.idl_path.as_ref());
        if let Some(sails_crate) = self.sails_crate.as_ref() {
            client_gen = client_gen.with_sails_crate(sails_crate);
        }
        for (name, path) in self.external_types.iter() {
            client_gen = client_gen.with_external_type(name, path);
        }
        if self.no_derive_traits {
            client_gen = client_gen.with_no_derive_traits();
        }

        let out_path = self
            .out_path
            .unwrap_or_else(|| self.idl_path.with_extension("sol.rs"));
        client_gen.generate_to(out_path)
    }
}

