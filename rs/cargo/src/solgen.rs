use crate::utils::{self, OutputExt};
use anyhow::{Context, Result, bail};
use askama::Template;
use chrono::{Datelike, Utc};
use convert_case::{Case, Casing};
use sails_sol_gen::{LICENSE_IDENTIFIER, SOLIDITY_VERSION, SolidityFile};
use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const FORGE_STD_VERSION: &str = "1.16.2";
const EVM_VERSION: &str = "cancun";
const ICON_CONFIG: &str = "📋";
const ICON_WORKSPACE: &str = "📜";
const ICON_DEPENDENCIES: &str = "📦";
const ICON_FORMAT: &str = "✨";
const ICON_DONE: &str = "✅";

#[derive(Template)]
#[template(path = "forge/.github/workflows/test.askama")]
struct TestWorkflow {
    git_branch_name: String,
}

#[derive(Template)]
#[template(path = "forge/vscode/settings.askama")]
struct VsCodeSettings {
    solidity_version: String,
}

// TODO: script, src, test

#[derive(Template)]
#[template(path = "forge/script/AbiScript.askama")]
struct AbiScript {
    license_identifier: String,
    solidity_version: String,
    contract_name: String,
    contract_name_camel_case: String,
}

#[derive(Template)]
#[template(path = "forge/script/CallerScript.askama")]
struct CallerScript {
    license_identifier: String,
    solidity_version: String,
    contract_name: String,
    contract_name_camel_case: String,
}

#[derive(Template)]
#[template(path = "forge/test/AbiTest.askama")]
struct AbiTest {
    license_identifier: String,
    solidity_version: String,
    contract_name: String,
    contract_name_camel_case: String,
}

#[derive(Template)]
#[template(path = "forge/test/CallerTest.askama")]
struct CallerTest {
    license_identifier: String,
    solidity_version: String,
    contract_name: String,
    contract_name_camel_case: String,
}

#[derive(Template)]
#[template(path = "forge/env_example.askama")]
struct RootEnvExample {
    contract_name: String,
}

#[derive(Template)]
#[template(path = "forge/foundry.askama")]
struct RootFoundry {
    forge_std_version: String,
    evm_version: String,
    solidity_version: String,
}

#[derive(Template)]
#[template(path = "forge/gitignore.askama")]
struct RootGitignore;

#[derive(Template)]
#[template(path = "forge/license.askama")]
struct RootLicense {
    copyright_year: String,
    package_author: String,
}

#[derive(Template)]
#[template(path = "forge/readme.askama")]
struct RootReadme {
    contract_name: String,
    contract_name_kebab_case: String,
    github_username: String,
}

#[derive(Template)]
#[template(path = "forge/remappings.askama")]
struct RootRemappings {
    forge_std_version: String,
}

pub enum SolidityGeneratorOutputType {
    SingleSolidity,
    ForgeProject,
}

pub struct SolidityGenerator {
    idl_path: PathBuf,
    target_dir: PathBuf,
    contract_name: Option<String>,
    package_author: String,
    github_username: String,
    offline: bool,
    output_type: SolidityGeneratorOutputType,
}

impl SolidityGenerator {
    const DEFAULT_AUTHOR: &str = "Gear Technologies";
    const DEFAULT_GITHUB_USERNAME: &str = "gear-tech";

    pub fn new(
        idl_path: PathBuf,
        target_dir: Option<PathBuf>,
        contract_name: Option<String>,
        author: Option<String>,
        username: Option<String>,
        offline: bool,
        output_type: SolidityGeneratorOutputType,
    ) -> Self {
        let package_author = author.unwrap_or_else(|| Self::DEFAULT_AUTHOR.to_string());
        let github_username = username.unwrap_or_else(|| Self::DEFAULT_GITHUB_USERNAME.to_string());
        Self {
            idl_path,
            target_dir: target_dir.unwrap_or_default(),
            contract_name,
            package_author,
            github_username,
            offline,
            output_type,
        }
    }

    fn test_workflow(&self, git_branch_name: &str) -> TestWorkflow {
        TestWorkflow {
            git_branch_name: git_branch_name.into(),
        }
    }

    fn vscode_settings(&self) -> VsCodeSettings {
        VsCodeSettings {
            solidity_version: SOLIDITY_VERSION.into(),
        }
    }

    // TODO: script, src, test

    fn abi_script(&self, contract_name: &str, contract_name_camel_case: &str) -> AbiScript {
        AbiScript {
            license_identifier: LICENSE_IDENTIFIER.into(),
            solidity_version: SOLIDITY_VERSION.into(),
            contract_name: contract_name.into(),
            contract_name_camel_case: contract_name_camel_case.into(),
        }
    }

    fn caller_script(&self, contract_name: &str, contract_name_camel_case: &str) -> CallerScript {
        CallerScript {
            license_identifier: LICENSE_IDENTIFIER.into(),
            solidity_version: SOLIDITY_VERSION.into(),
            contract_name: contract_name.into(),
            contract_name_camel_case: contract_name_camel_case.into(),
        }
    }

    fn abi_test(&self, contract_name: &str, contract_name_camel_case: &str) -> AbiTest {
        AbiTest {
            license_identifier: LICENSE_IDENTIFIER.into(),
            solidity_version: SOLIDITY_VERSION.into(),
            contract_name: contract_name.into(),
            contract_name_camel_case: contract_name_camel_case.into(),
        }
    }

    fn caller_test(&self, contract_name: &str, contract_name_camel_case: &str) -> CallerTest {
        CallerTest {
            license_identifier: LICENSE_IDENTIFIER.into(),
            solidity_version: SOLIDITY_VERSION.into(),
            contract_name: contract_name.into(),
            contract_name_camel_case: contract_name_camel_case.into(),
        }
    }

    fn root_env_example(&self, contract_name: &str) -> RootEnvExample {
        RootEnvExample {
            contract_name: contract_name.into(),
        }
    }

    fn root_foundry(&self) -> RootFoundry {
        RootFoundry {
            forge_std_version: FORGE_STD_VERSION.into(),
            evm_version: EVM_VERSION.into(),
            solidity_version: SOLIDITY_VERSION.into(),
        }
    }

    fn root_gitignore(&self) -> RootGitignore {
        RootGitignore
    }

    fn root_license(&self) -> RootLicense {
        RootLicense {
            copyright_year: Utc::now().year().to_string(),
            package_author: self.package_author.clone(),
        }
    }

    fn root_readme(&self, contract_name: &str, contract_name_kebab_case: &str) -> RootReadme {
        RootReadme {
            contract_name: contract_name.into(),
            contract_name_kebab_case: contract_name_kebab_case.into(),
            github_username: self.github_username.clone(),
        }
    }

    fn root_remappings(&self) -> RootRemappings {
        RootRemappings {
            forge_std_version: FORGE_STD_VERSION.into(),
        }
    }

    pub fn generate(self) -> Result<()> {
        let idl_content = fs::read_to_string(&self.idl_path)?;
        let filename: String = if let Some(stem) = self.idl_path.file_stem() {
            stem.to_string_lossy().into()
        } else {
            bail!("No filename found");
        };
        let contract_name = self.contract_name.clone().unwrap_or(filename.to_string());

        println!("⛵ Creating bindings for Solidity...");
        self.print_config(&contract_name);

        match self.output_type {
            SolidityGeneratorOutputType::SingleSolidity => {
                self.generate_single_solidity(&idl_content, &contract_name.to_case(Case::Pascal))?
            }
            SolidityGeneratorOutputType::ForgeProject => {
                self.generate_forge_project(&idl_content, &contract_name.to_case(Case::Kebab))?
            }
        }

        Ok(())
    }

    fn print_config(&self, contract_name: &str) {
        let print_field = |label: &str, value: &dyn Display| {
            println!("   {label:<10} {value}");
        };

        println!("{ICON_CONFIG} Solidity config:");
        print_field("idl:", &self.idl_path.display());
        if self.target_dir.as_os_str().is_empty() {
            print_field("path:", &"none");
        } else {
            print_field("path:", &self.target_dir.display());
        }
        print_field("contract:", &contract_name);
        print_field("author:", &self.package_author);
        print_field("username:", &self.github_username);
        print_field("offline:", &self.offline);
        match self.output_type {
            SolidityGeneratorOutputType::SingleSolidity => {
                print_field("output:", &"single Solidity file");
            }
            SolidityGeneratorOutputType::ForgeProject => {
                print_field("output:", &"Forge project");
            }
        }
    }

    fn generate_single_solidity(&self, idl_content: &str, contract_name: &str) -> Result<()> {
        let data = sails_sol_gen::generate_solidity_contract(
            contract_name,
            idl_content,
            SolidityFile::SingleFile,
        )?;
        let target_file = self.target_dir.join(format!("{contract_name}.sol"));

        fs::write(&target_file, data)?;

        println!(
            "Generated single Solidity file: {target_file}",
            target_file = target_file.display()
        );

        Ok(())
    }

    fn generate_forge_project(
        &self,
        idl_content: &str,
        contract_name_kebab_case: &str,
    ) -> Result<()> {
        println!("{ICON_WORKSPACE} [1/3] Initializing workspace...");
        let project_dir = self.generate_root(idl_content, contract_name_kebab_case)?;

        println!("{ICON_DEPENDENCIES} [2/3] Installing dependencies...");
        self.install(&project_dir)?;

        println!("{ICON_FORMAT} [3/3] Formatting workspace...");
        self.fmt(&project_dir)?;

        println!("{ICON_DONE} Done.");

        Ok(())
    }

    fn generate_root(&self, idl_content: &str, contract_name_kebab_case: &str) -> Result<PathBuf> {
        let contract_name_pascal_case = contract_name_kebab_case.to_case(Case::Pascal);
        let contract_name_camel_case = contract_name_kebab_case.to_case(Case::Camel);

        let project_dir = self.target_dir.join(contract_name_kebab_case);
        if project_dir.exists() {
            bail!("Target directory already exists: {}", project_dir.display());
        }
        fs::create_dir_all(&project_dir)?;

        let project_dir = project_dir
            .canonicalize()
            .context("Failed to canonicalize project directory")?;
        let path = &project_dir;

        git_init(path)?;

        let git_branch_name = utils::git_show_current_branch(path)?;
        println!("   git branch: {git_branch_name}");

        fs::create_dir_all(test_workflow_dir_path(path))?;
        let mut test_workflow_yml = File::create(test_workflow_path(path))?;
        self.test_workflow(&git_branch_name)
            .write_into(&mut test_workflow_yml)?;

        fs::create_dir_all(vscode_dir_path(path))?;
        let mut vscode_settings_json = File::create(vscode_settings_path(path))?;
        self.vscode_settings()
            .write_into(&mut vscode_settings_json)?;

        // TODO: script, src, test

        fs::create_dir_all(script_dir_path(path))?;

        let mut abi_script_sol =
            File::create(abi_script_sol_path(path, &contract_name_pascal_case))?;
        self.abi_script(&contract_name_pascal_case, &contract_name_camel_case)
            .write_into(&mut abi_script_sol)?;

        let mut caller_script_sol =
            File::create(caller_script_sol_path(path, &contract_name_pascal_case))?;
        self.caller_script(&contract_name_pascal_case, &contract_name_camel_case)
            .write_into(&mut caller_script_sol)?;

        fs::create_dir_all(src_dir_path(path))?;

        let mut solidity_interface_file = File::create(solidity_interface_file_path(
            path,
            &contract_name_pascal_case,
        ))?;
        let solidity_interface_file_content = sails_sol_gen::generate_solidity_contract(
            &contract_name_pascal_case,
            idl_content,
            SolidityFile::InterfaceFile,
        )?;
        solidity_interface_file.write_all(&solidity_interface_file_content)?;

        let mut solidity_abi_interface_file = File::create(solidity_abi_interface_file_path(
            path,
            &contract_name_pascal_case,
        ))?;
        let solidity_abi_interface_file_content = sails_sol_gen::generate_solidity_contract(
            &contract_name_pascal_case,
            idl_content,
            SolidityFile::AbiInterfaceFile,
        )?;
        solidity_abi_interface_file.write_all(&solidity_abi_interface_file_content)?;

        let mut solidity_callbacks_interface_file = File::create(
            solidity_callbacks_interface_file_path(path, &contract_name_pascal_case),
        )?;
        let solidity_callbacks_interface_file_content = sails_sol_gen::generate_solidity_contract(
            &contract_name_pascal_case,
            idl_content,
            SolidityFile::CallbacksInterfaceFile,
        )?;
        solidity_callbacks_interface_file.write_all(&solidity_callbacks_interface_file_content)?;

        let mut solidity_caller_file =
            File::create(solidity_caller_file_path(path, &contract_name_pascal_case))?;
        let solidity_caller_file_content = sails_sol_gen::generate_solidity_contract(
            &contract_name_pascal_case,
            idl_content,
            SolidityFile::CallerFile,
        )?;
        solidity_caller_file.write_all(&solidity_caller_file_content)?;

        fs::create_dir_all(test_dir_path(path))?;

        let mut test_abi_test_sol =
            File::create(test_abi_test_sol_path(path, &contract_name_pascal_case))?;
        self.abi_test(&contract_name_pascal_case, &contract_name_camel_case)
            .write_into(&mut test_abi_test_sol)?;

        let mut test_caller_test_sol =
            File::create(test_caller_test_sol_path(path, &contract_name_pascal_case))?;
        self.caller_test(&contract_name_pascal_case, &contract_name_camel_case)
            .write_into(&mut test_caller_test_sol)?;

        let mut root_env_example = File::create(env_example_path(path))?;
        self.root_env_example(&contract_name_pascal_case)
            .write_into(&mut root_env_example)?;

        let mut foundry_toml = File::create(foundry_path(path))?;
        self.root_foundry().write_into(&mut foundry_toml)?;

        let mut gitignore = File::create(gitignore_path(path))?;
        self.root_gitignore().write_into(&mut gitignore)?;

        let mut license = File::create(license_path(path))?;
        self.root_license().write_into(&mut license)?;

        let mut readme = File::create(readme_path(path))?;
        self.root_readme(&contract_name_pascal_case, contract_name_kebab_case)
            .write_into(&mut readme)?;

        let mut remappings = File::create(remappings_path(path))?;
        self.root_remappings().write_into(&mut remappings)?;

        Ok(project_dir)
    }

    fn install<P: AsRef<Path>>(&self, project_dir: P) -> Result<()> {
        let path = &project_dir;

        if !self.offline {
            forge_soldeer_install(path)?;
        }

        Ok(())
    }

    fn fmt<P: AsRef<Path>>(&self, project_dir: P) -> Result<()> {
        let path = &project_dir;
        forge_fmt(path)
    }
}

fn forge_soldeer_install<P: AsRef<Path>>(dir: P) -> Result<()> {
    let forge_command = forge_command();
    let mut cmd = Command::new(forge_command);
    cmd.current_dir(dir)
        .stdout(Stdio::piped())
        .arg("soldeer")
        .arg("install");

    let _ = cmd
        .output()?
        .exit_result()
        .context("failed to install soldeer packages")?;

    Ok(())
}

fn forge_fmt<P: AsRef<Path>>(dir: P) -> Result<()> {
    let forge_command = forge_command();
    let mut cmd = Command::new(forge_command);
    cmd.current_dir(dir).stdout(Stdio::piped()).arg("fmt");

    let _ = cmd
        .output()?
        .exit_result()
        .context("failed to format the project")?;

    Ok(())
}

fn git_init<P: AsRef<Path>>(target_dir: P) -> Result<()> {
    let git_command = utils::git_command();
    let mut cmd = Command::new(git_command);
    cmd.stdout(Stdio::piped())
        .arg("-C")
        .arg(target_dir.as_ref())
        .arg("init");

    let _ = cmd
        .output()?
        .exit_result()
        .context("failed to initialize git repository")?;

    Ok(())
}

fn test_workflow_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".github/workflows")
}

fn test_workflow_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".github/workflows/test.yml")
}

fn vscode_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".vscode")
}

fn vscode_settings_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".vscode/settings.json")
}

// TODO: script, src, test

fn script_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("script")
}

fn abi_script_sol_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("script")
        .join(format!("{contract_name}Abi.s.sol"))
}

fn caller_script_sol_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("script")
        .join(format!("{contract_name}Caller.s.sol"))
}

fn src_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("src")
}

fn solidity_interface_file_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("src")
        .join(format!("I{contract_name}.sol"))
}

fn solidity_abi_interface_file_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("src")
        .join(format!("{contract_name}Abi.sol"))
}

fn solidity_callbacks_interface_file_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("src")
        .join(format!("I{contract_name}Callbacks.sol"))
}

fn solidity_caller_file_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("src")
        .join(format!("{contract_name}Caller.sol"))
}

fn test_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("test")
}

fn test_abi_test_sol_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("test")
        .join(format!("{contract_name}Abi.t.sol"))
}

fn test_caller_test_sol_path<P: AsRef<Path>>(path: P, contract_name: &str) -> PathBuf {
    path.as_ref()
        .join("test")
        .join(format!("{contract_name}Caller.t.sol"))
}

fn env_example_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".env.example")
}

fn foundry_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("foundry.toml")
}

fn gitignore_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".gitignore")
}

fn license_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("LICENSE")
}

fn readme_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("README.md")
}

fn remappings_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("remappings.txt")
}

fn forge_command() -> String {
    env::var("FORGE").unwrap_or("forge".into())
}
