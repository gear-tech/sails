use anyhow::Result;
use cargo_generate::{GenerateArgs, TemplatePath};
use std::{env, path::PathBuf};

const TEMPLATES_BRANCH_ENV_VAR: &str = "SAILS_CLI_TEMPLATES_BRANCH";
const TEMPLATES_REPO: &str = "https://github.com/gear-tech/sails.git";
const PROGRAM_TEMPLATE_PATH: &str = "templates/program";

pub struct ProgramGenerator {
    path: String,
    name: Option<String>,
    with_client: bool,
    with_gtest: bool,
}

impl ProgramGenerator {
    pub fn new(path: String) -> Self {
        Self {
            path,
            name: None,
            with_client: false,
            with_gtest: false,
        }
    }

    pub fn with_name(self, name: Option<String>) -> Self {
        Self { name, ..self }
    }

    pub fn with_client(self, with_client: bool) -> Self {
        Self {
            with_client,
            ..self
        }
    }

    pub fn with_gtest(self, with_gtest: bool) -> Self {
        Self {
            with_gtest,
            with_client: self.with_client | with_gtest,
            ..self
        }
    }

    pub fn generate(self) -> Result<()> {
        let template_path = TemplatePath {
            auto_path: Some(PROGRAM_TEMPLATE_PATH.into()),
            git: Some(TEMPLATES_REPO.into()),
            branch: env::var(TEMPLATES_BRANCH_ENV_VAR).ok(),
            ..TemplatePath::default()
        };

        let (path, name) = if self.name.is_none() {
            let path_buf = PathBuf::from(&self.path);
            let path = path_buf.parent().map(|p| p.to_path_buf());
            let name = path_buf.file_name().map(|n| {
                n.to_str()
                    .expect("unreachable as was built from UTF-8")
                    .to_string()
            });
            (path, name)
        } else {
            (Some(PathBuf::from(&self.path)), self.name)
        };

        let generate_args = GenerateArgs {
            template_path,
            name,
            destination: path,
            silent: true,
            define: vec![
                format!("with-client={}", self.with_client),
                format!("with-gtest={}", self.with_gtest),
            ],
            ..GenerateArgs::default()
        };
        cargo_generate::generate(generate_args)?;
        Ok(())
    }
}
