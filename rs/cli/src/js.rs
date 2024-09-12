use anyhow::Result;
use std::process::Command;

pub struct JsClientGenerator {
    idl_path: String,
    out_path: String,
    program_name: String,
}

impl JsClientGenerator {
    pub fn new(idl_path: String, out_path: String, program_name: String) -> Self {
        Self {
            idl_path,
            out_path,
            program_name,
        }
    }

    pub fn generate(&self) -> Result<()> {
        check_node()?;
        check_npx()?;

        let out_path = self.out_path.clone();
        let program_name = self.program_name.clone();
        let idl_path = self.idl_path.clone();

        let mut child = Command::new("npx")
            .arg("sails-js-cli@0.1.0")
            .arg("generate")
            .arg(idl_path)
            .arg("-o")
            .arg(out_path)
            .arg("-n")
            .arg(program_name)
            .spawn()
            .expect("Failed to run npx sails-js-cli");

        let status = child.wait().expect("An error occured");

        if !status.success() {
            panic!("Failed to generate JS client");
        }
        Ok(())
    }
}

pub fn check_app_installed(app: &str) -> bool {
    Command::new("which")
        .arg(app)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn check_node() -> Result<()> {
    if !check_app_installed("node") {
        panic!("Node.js is not installed. Please install Node.js to continue.")
    }

    Ok(())
}

pub fn check_npx() -> Result<()> {
    if !check_app_installed("npx") {
        panic!("npx is not installed. Please install npx to continue.")
    }

    Ok(())
}
