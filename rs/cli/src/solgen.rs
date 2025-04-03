use anyhow::Result;
use sails_sol_gen::generate_solidity_contract;
use std::path::PathBuf;

pub struct SolidityGenerator {
    idl_path: String,
    target_dir: PathBuf,
    contract_name: String,
}

impl SolidityGenerator {
    pub fn new(idl_path: String, target_dir: Option<PathBuf>, contract_name: String) -> Self {
        Self {
            idl_path,
            target_dir: target_dir.unwrap_or_default(),
            contract_name,
        }
    }

    pub fn generate(&self) -> Result<()> {
        let idl_content = std::fs::read_to_string(&self.idl_path)?;

        let contract = generate_solidity_contract(&idl_content, &self.contract_name)?;

        let target_file = self.target_dir.join(format!("{}.sol", self.contract_name));

        std::fs::write(&target_file, contract)?;

        println!("Generated contract: {:?}", target_file);

        Ok(())
    }
}
