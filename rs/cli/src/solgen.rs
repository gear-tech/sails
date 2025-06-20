use anyhow::{Result, bail};
use sails_sol_gen::generate_solidity_contract;
use std::path::PathBuf;

pub struct SolidityGenerator {
    idl_path: PathBuf,
    target_dir: PathBuf,
    contract_name: Option<String>,
}

impl SolidityGenerator {
    pub fn new(
        idl_path: PathBuf,
        target_dir: Option<PathBuf>,
        contract_name: Option<String>,
    ) -> Self {
        Self {
            idl_path,
            target_dir: target_dir.unwrap_or_default(),
            contract_name,
        }
    }

    pub fn generate(&self) -> Result<()> {
        let idl_content = std::fs::read_to_string(&self.idl_path)?;

        let filename = if let Some(stem) = self.idl_path.file_stem() {
            stem.to_string_lossy()
        } else {
            bail!("No filename found");
        };

        let unformatted_contract_name = self.contract_name.clone().unwrap_or(filename.to_string());

        let contract = generate_solidity_contract(&idl_content, &unformatted_contract_name)?;

        let target_file = self.target_dir.join(format!("{}.sol", &contract.name));

        std::fs::write(&target_file, contract.data)?;

        println!("Generated contract: {:?}", target_file);

        Ok(())
    }
}
