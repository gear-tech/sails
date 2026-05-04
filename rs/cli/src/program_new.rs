use anyhow::Context;
use askama::Template;
use cargo_metadata::DependencyKind::{Build, Development, Normal};
use convert_case::{Case, Casing};
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

const SAILS_VERSION: &str = env!("CARGO_PKG_VERSION");
const TOKIO_VERSION: &str = "1.50.0";
const ICON_CONFIG: &str = "📋";
const ICON_WORKSPACE: &str = "⚓";
const ICON_APP: &str = "📦";
const ICON_CLIENT: &str = "📡";
const ICON_BUILD: &str = "🔨";
const ICON_TESTS: &str = "🔬";
const ICON_FORMAT: &str = "✨";
const ICON_DONE: &str = "✅";
const CRATES_IO: &str = "crates-io";

trait ExitStatusExt: Sized {
    fn exit_result(self) -> io::Result<()>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_result(self) -> io::Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(io::Error::from(io::ErrorKind::Other))
        }
    }
}

trait OutputExt: Sized {
    fn exit_result(self) -> io::Result<Self>;
}

impl OutputExt for Output {
    fn exit_result(self) -> io::Result<Self> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(io::Error::from(io::ErrorKind::Other))
        }
    }
}

#[derive(Template)]
#[template(path = ".github/workflows/ci.askama")]
struct CIWorkflow {
    git_branch_name: String,
    client_file_name: String,
}

#[derive(Template)]
#[template(path = "app/src/lib.askama")]
struct AppLib {
    service_name: String,
    service_name_snake: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "client/src/lib.askama")]
struct ClientLib {
    client_file_name: String,
}

#[derive(Template)]
#[template(path = "client/build.askama")]
struct ClientBuild {
    app_crate_name: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "src/lib.askama")]
struct RootLib {
    app_crate_name: String,
}

#[derive(Template)]
#[template(path = "tests/gtest.askama")]
struct TestsGtest {
    program_crate_name: String,
    client_crate_name: String,
    client_program_name: String,
    service_name: String,
    service_name_snake: String,
}

#[derive(Template)]
#[template(path = "build.askama")]
struct RootBuild {
    app_crate_name: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "license.askama")]
struct RootLicense {
    package_author: String,
}

#[derive(Template)]
#[template(path = "readme.askama")]
struct RootReadme {
    program_crate_name: String,
    github_username: String,
    app_crate_name: String,
    client_crate_name: String,
    service_name: String,
}

#[derive(Template)]
#[template(path = "rust-toolchain.askama")]
struct RootRustToolchain;

pub struct ProgramGenerator {
    path: PathBuf,
    package_name: String,
    package_author: String,
    github_username: String,
    client_file_name: String,
    sails_path: Option<PathBuf>,
    offline: bool,
    ethereum: bool,
    service_name: String,
    program_struct_name: String,
}

impl ProgramGenerator {
    const DEFAULT_AUTHOR: &str = "Gear Technologies";
    const DEFAULT_GITHUB_USERNAME: &str = "gear-tech";

    const GITIGNORE_ENTRIES: &[&str] =
        &[".binpath", ".DS_Store", ".vscode", ".idea", "/target", ""];

    pub fn new(
        path: PathBuf,
        name: Option<String>,
        author: Option<String>,
        username: Option<String>,
        sails_path: Option<PathBuf>,
        offline: bool,
        ethereum: bool,
    ) -> Self {
        let package_name = name.map_or_else(
            || {
                path.file_name()
                    .expect("Invalid Path")
                    .to_str()
                    .expect("Invalid UTF-8 Path")
                    .to_case(Case::Kebab)
            },
            |name| name.to_case(Case::Kebab),
        );
        let service_name = package_name.to_case(Case::Pascal);
        let package_author = author.unwrap_or_else(|| Self::DEFAULT_AUTHOR.to_string());
        let github_username = username.unwrap_or_else(|| Self::DEFAULT_GITHUB_USERNAME.to_string());
        let client_file_name = format!("{}_client", package_name.to_case(Case::Snake));
        Self {
            path,
            package_name,
            package_author,
            github_username,
            client_file_name,
            sails_path,
            offline,
            ethereum,
            service_name,
            program_struct_name: "Program".to_string(),
        }
    }

    fn ci_workflow(&self, git_branch_name: &str) -> CIWorkflow {
        CIWorkflow {
            git_branch_name: git_branch_name.into(),
            client_file_name: self.client_file_name.clone(),
        }
    }

    fn app_lib(&self) -> AppLib {
        AppLib {
            service_name: self.service_name.clone(),
            service_name_snake: self.service_name.to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn client_lib(&self) -> ClientLib {
        ClientLib {
            client_file_name: self.client_file_name.clone(),
        }
    }

    fn client_build(&self) -> ClientBuild {
        ClientBuild {
            app_crate_name: self.app_name().to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn root_lib(&self) -> RootLib {
        RootLib {
            app_crate_name: self.app_name().to_case(Case::Snake),
        }
    }

    fn tests_gtest(&self) -> TestsGtest {
        TestsGtest {
            program_crate_name: self.package_name.to_case(Case::Snake),
            client_crate_name: self.client_name().to_case(Case::Snake),
            client_program_name: self.client_name().to_case(Case::Pascal),
            service_name: self.service_name.clone(),
            service_name_snake: self.service_name.to_case(Case::Snake),
        }
    }

    fn root_build(&self) -> RootBuild {
        RootBuild {
            app_crate_name: self.app_name().to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn root_license(&self) -> RootLicense {
        RootLicense {
            package_author: self.package_author.clone(),
        }
    }

    fn root_readme(&self) -> RootReadme {
        RootReadme {
            program_crate_name: self.package_name.clone(),
            github_username: self.github_username.clone(),
            app_crate_name: self.app_name(),
            client_crate_name: self.client_name(),
            service_name: self.service_name.clone(),
        }
    }

    fn root_rust_toolchain(&self) -> RootRustToolchain {
        RootRustToolchain
    }

    fn app_path(&self) -> PathBuf {
        self.path.join("app")
    }

    fn app_name(&self) -> String {
        format!("{}-app", self.package_name)
    }

    fn client_path(&self) -> PathBuf {
        self.path.join("client")
    }

    fn client_name(&self) -> String {
        format!("{}-client", self.package_name)
    }

    fn cargo_add_sails_rs<P: AsRef<Path>>(
        &self,
        manifest_path: P,
        dependency: cargo_metadata::DependencyKind,
        features: Option<&str>,
    ) -> anyhow::Result<()> {
        let sails_package = ["sails-rs"];
        cargo_add(
            manifest_path,
            sails_package,
            dependency,
            features,
            self.offline,
        )
    }

    fn print_config(&self) {
        let sails_source = self
            .sails_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| format!("crates.io:{SAILS_VERSION}"));
        let print_field = |label: &str, value: &dyn std::fmt::Display| {
            println!("   {label:<10} {value}");
        };

        println!("{ICON_CONFIG} Program config:");
        print_field("path:", &self.path.display());
        print_field("package:", &self.package_name);
        print_field("author:", &self.package_author);
        print_field("username:", &self.github_username);
        print_field("sails-rs:", &sails_source);
        print_field("offline:", &self.offline);
        print_field("eth:", &self.ethereum);
    }

    pub fn generate(self) -> anyhow::Result<()> {
        println!("⛵ Creating new Sails program...");
        self.print_config();

        println!("{ICON_WORKSPACE} [1/6] Initializing workspace...");
        self.generate_root()?;
        println!("{ICON_APP} [2/6] Generating app crate...");
        self.generate_app()?;
        println!("{ICON_CLIENT} [3/6] Generating client crate...");
        self.generate_client()?;
        println!("{ICON_BUILD} [4/6] Wiring root crate...");
        self.generate_build()?;
        println!("{ICON_TESTS} [5/6] Generating tests...");
        self.generate_tests()?;
        println!("{ICON_FORMAT} [6/6] Formatting workspace...");
        self.fmt()?;
        println!("{ICON_DONE} Done.");
        Ok(())
    }

    fn generate_app(&self) -> anyhow::Result<()> {
        let path = &self.app_path();
        cargo_new(path, &self.app_name(), self.offline, false)?;
        let manifest_path = &manifest_path(path);

        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, self.ethereum.then_some("ethexe"))?;
        // dev-dep with `std` enables `Syscall::with_*` mocks for inline unit tests.
        self.cargo_add_sails_rs(
            manifest_path,
            Development,
            Some(if self.ethereum { "ethexe,std" } else { "std" }),
        )?;

        let mut lib_rs = File::create(lib_rs_path(path))?;
        self.app_lib().write_into(&mut lib_rs)?;

        Ok(())
    }

    fn generate_root(&self) -> anyhow::Result<()> {
        let path = &self.path;
        cargo_new(path, &self.package_name, self.offline, true)?;

        let git_branch_name = git_show_current_branch(path)?;
        println!("   git branch: {git_branch_name}");

        fs::create_dir_all(ci_workflow_dir_path(path))?;
        let mut ci_workflow_yml = File::create(ci_workflow_path(path))?;
        self.ci_workflow(&git_branch_name)
            .write_into(&mut ci_workflow_yml)?;

        let mut gitignore = File::create(gitignore_path(path))?;
        gitignore.write_all(Self::GITIGNORE_ENTRIES.join("\n").as_ref())?;

        let manifest_path = &manifest_path(path);
        cargo_toml_create_workspace_and_fill_package(
            manifest_path,
            &self.package_name,
            &self.package_author,
            &self.github_username,
            &self.sails_path,
        )?;

        let mut license = File::create(license_path(path))?;
        self.root_license().write_into(&mut license)?;

        let mut readme_md = File::create(readme_path(path))?;
        self.root_readme().write_into(&mut readme_md)?;

        let mut rust_toolchain_toml = File::create(rust_toolchain_path(path))?;
        self.root_rust_toolchain()
            .write_into(&mut rust_toolchain_toml)?;

        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, self.ethereum.then_some("ethexe"))?;

        // update `sails-rs` if not path ref and not offline
        if self.sails_path.is_none() && !self.offline {
            // fix `error: failed to select a version for the requirement``
            cargo_info("sails-idl-embed")?;
            cargo_info("sails-idl-gen")?;
            cargo_info("sails-client-gen-v2")?;
            cargo_info("sails-idl-parser-v2")?;
        }

        self.cargo_add_sails_rs(
            manifest_path,
            Build,
            Some(if self.ethereum {
                "ethexe,build"
            } else {
                "build"
            }),
        )?;

        Ok(())
    }

    fn generate_build(&self) -> anyhow::Result<()> {
        let path = &self.path;
        let manifest_path = &manifest_path(path);

        let mut lib_rs = File::create(lib_rs_path(path))?;
        self.root_lib().write_into(&mut lib_rs)?;

        let mut build_rs = File::create(build_rs_path(path))?;
        self.root_build().write_into(&mut build_rs)?;

        // add app ref
        cargo_add(manifest_path, [self.app_name()], Normal, None, self.offline)?;
        cargo_add(manifest_path, [self.app_name()], Build, None, self.offline)?;

        Ok(())
    }

    fn generate_client(&self) -> anyhow::Result<()> {
        let path = &self.client_path();
        cargo_new(path, &self.client_name(), self.offline, false)?;

        let manifest_path = &manifest_path(path);
        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, self.ethereum.then_some("ethexe"))?;
        self.cargo_add_sails_rs(
            manifest_path,
            Build,
            Some(if self.ethereum {
                "ethexe,build"
            } else {
                "build"
            }),
        )?;

        // add app ref
        cargo_add(manifest_path, [self.app_name()], Build, None, self.offline)?;

        let mut build_rs = File::create(build_rs_path(path))?;
        self.client_build().write_into(&mut build_rs)?;

        let mut lib_rs = File::create(lib_rs_path(path))?;
        self.client_lib().write_into(&mut lib_rs)?;

        Ok(())
    }

    fn generate_tests(&self) -> anyhow::Result<()> {
        let path = &self.path;
        let manifest_path = &manifest_path(path);
        // add sails-rs refs
        self.cargo_add_sails_rs(
            manifest_path,
            Development,
            Some(if self.ethereum {
                "ethexe,gtest,gclient"
            } else {
                "gtest,gclient"
            }),
        )?;

        // add tokio
        cargo_add(
            manifest_path,
            ["tokio"],
            Development,
            Some("rt,macros"),
            self.offline,
        )?;

        // add app ref
        cargo_add(
            manifest_path,
            [self.app_name()],
            Development,
            None,
            self.offline,
        )?;
        // add client ref
        cargo_add(
            manifest_path,
            [self.client_name()],
            Development,
            None,
            self.offline,
        )?;

        // add tests
        let test_path = &tests_path(path);
        fs::create_dir_all(test_path.parent().context("Parent should exists")?)?;
        let mut gtest_rs = File::create(test_path)?;
        self.tests_gtest().write_into(&mut gtest_rs)?;

        Ok(())
    }

    fn fmt(&self) -> anyhow::Result<()> {
        let manifest_path = &manifest_path(&self.path);
        cargo_fmt(manifest_path)
    }
}

fn git_show_current_branch<P: AsRef<Path>>(target_dir: P) -> anyhow::Result<String> {
    let git_command = git_command();
    let mut cmd = Command::new(git_command);
    cmd.stdout(Stdio::piped())
        .arg("-C")
        .arg(target_dir.as_ref())
        .arg("branch")
        .arg("--show-current");

    let output = cmd
        .output()?
        .exit_result()
        .context("failed to get current git branch")?;
    let git_branch_name = String::from_utf8(output.stdout)?;

    Ok(git_branch_name.trim().into())
}

fn cargo_new<P: AsRef<Path>>(
    target_dir: P,
    name: &str,
    offline: bool,
    root: bool,
) -> anyhow::Result<()> {
    let cargo_command = cargo_command();
    let target_dir = target_dir.as_ref();
    let cargo_new_or_init = if target_dir.exists() { "init" } else { "new" };
    println!(
        "   cargo {cargo_new_or_init}: {name} -> {}",
        target_dir.display()
    );
    let mut cmd = Command::new(cargo_command);
    cmd.stdout(Stdio::null()) // Don't pollute output
        .arg(cargo_new_or_init)
        .arg(target_dir)
        .arg("--name")
        .arg(name)
        .arg("--lib")
        .arg("--quiet");

    if offline {
        cmd.arg("--offline");
    }

    cmd.status()
        .context("failed to execute `cargo new` command")?
        .exit_result()
        .context("failed to run `cargo new` command")?;

    if !root {
        let manifest_path = target_dir.join("Cargo.toml");
        let cargo_toml = fs::read_to_string(&manifest_path)?;
        let mut document: toml_edit::DocumentMut = cargo_toml.parse()?;

        let crate_path = name
            .rsplit_once('-')
            .map(|(_, crate_path)| crate_path)
            .unwrap_or(name);
        let description = match crate_path {
            "app" => "Package containing business logic for the program",
            "client" => {
                "Package containing the client for the program allowing to interact with it"
            }
            _ => unreachable!(),
        };

        let package = document
            .entry("package")
            .or_insert_with(toml_edit::table)
            .as_table_mut()
            .context("package was not a table in Cargo.toml")?;

        let mut entries = vec![];

        for key in ["repository", "license", "keywords", "categories"] {
            if let Some(entry) = package.remove_entry(key) {
                entries.push(entry);
            }
        }

        _ = package
            .entry("description")
            .or_insert_with(|| toml_edit::value(description));

        for (key, item) in entries {
            package.insert_formatted(&key, item);
        }

        fs::write(manifest_path, document.to_string())?;

        if let Some(parent_dir) = target_dir.parent() {
            let manifest_path = parent_dir.join("Cargo.toml");
            let cargo_toml = fs::read_to_string(&manifest_path)?;
            let mut document: toml_edit::DocumentMut = cargo_toml.parse()?;

            let workspace = document
                .entry("workspace")
                .or_insert_with(toml_edit::table)
                .as_table_mut()
                .context("workspace was not a table in Cargo.toml")?;

            let dependencies = workspace
                .entry("dependencies")
                .or_insert_with(toml_edit::table)
                .as_table_mut()
                .context("workspace.dependencies was not a table in Cargo.toml")?;

            let mut dependency = toml_edit::InlineTable::new();
            dependency.insert("version", "0.1.0".into());
            dependency.insert("path", crate_path.into());

            dependencies.insert(name, dependency.into());

            fs::write(manifest_path, document.to_string())?;
        }
    }

    Ok(())
}

fn cargo_add<P, I, S>(
    manifest_path: P,
    packages: I,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
    offline: bool,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cargo_command = cargo_command();
    let package_args = packages
        .into_iter()
        .map(|package| package.as_ref().to_os_string())
        .collect::<Vec<OsString>>();
    let package_names = package_args
        .iter()
        .map(|package| package.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(", ");
    let dep_kind = match dependency {
        Development => "dev-dependency",
        Build => "build-dependency",
        Normal => "dependency",
        _ => "dependency",
    };
    let feature_suffix = features
        .map(|features| format!(" [features: {features}]"))
        .unwrap_or_default();
    println!(
        "   cargo add: {package_names} -> {} ({dep_kind}){feature_suffix}",
        manifest_path.as_ref().display()
    );

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(Stdio::null()) // Don't pollute output
        .arg("add")
        .args(&package_args)
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--quiet");

    match dependency {
        Development => {
            cmd.arg("--dev");
        }
        Build => {
            cmd.arg("--build");
        }
        _ => (),
    };

    if let Some(features) = features {
        cmd.arg("--features").arg(features);
    }

    if offline {
        cmd.arg("--offline");
    }

    cmd.status()
        .context("failed to execute `cargo add` command")?
        .exit_result()
        .context("failed to run `cargo add` command")?;

    Ok(())
}

#[allow(unused)]
fn cargo_update<P: AsRef<Path>>(manifest_path: P, package: Option<&str>) -> anyhow::Result<()> {
    let cargo_command = cargo_command();
    if let Some(package) = package {
        println!(
            "   cargo update: {} -> {}",
            package,
            manifest_path.as_ref().display()
        );
    } else {
        println!("   cargo update: {}", manifest_path.as_ref().display());
    }

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(Stdio::null()).arg("update");

    if let Some(package) = package {
        cmd.arg(package);
    }

    cmd.arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo update` command")?
        .exit_result()
        .context("failed to run `cargo update` command")?;

    Ok(())
}

fn cargo_info(package: &str) -> anyhow::Result<()> {
    let cargo_command = cargo_command();
    let package_version = &format!("{package}@{SAILS_VERSION}");
    println!("   cargo info: {package_version}");

    let mut cmd = Command::new(cargo_command);

    cmd.stdout(Stdio::null())
        .arg("info")
        .arg(package_version)
        .arg("--registry")
        .arg(CRATES_IO)
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo info` command")?
        .exit_result()
        .context("failed to run `cargo info` command")?;

    Ok(())
}

fn cargo_fmt<P: AsRef<Path>>(manifest_path: P) -> anyhow::Result<()> {
    let cargo_command = cargo_command();
    println!("   cargo fmt: {}", manifest_path.as_ref().display());

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(Stdio::null()) // Don't pollute output
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo fmt` command")?
        .exit_result()
        .context("failed to run `cargo fmt` command")
}

fn cargo_toml_create_workspace_and_fill_package<P: AsRef<Path>>(
    manifest_path: P,
    name: &str,
    author: &str,
    username: &str,
    sails_path: &Option<PathBuf>,
) -> anyhow::Result<()> {
    let manifest_path = manifest_path.as_ref();
    let cargo_toml = fs::read_to_string(manifest_path)?;
    let mut document: toml_edit::DocumentMut = cargo_toml.parse()?;

    let package = document
        .entry("package")
        .or_insert_with(toml_edit::table)
        .as_table_mut()
        .context("package was not a table in Cargo.toml")?;
    package.remove("edition");
    for key in [
        "version",
        "authors",
        "edition",
        "rust-version",
        "description",
        "repository",
        "license",
        "keywords",
        "categories",
    ] {
        if key == "description" {
            _ = package.entry(key).or_insert_with(|| {
                toml_edit::value(
                    "Package allowing to build WASM binary for the program and IDL file for it",
                )
            });
        } else {
            let item = package.entry(key).or_insert_with(toml_edit::table);
            let mut table = toml_edit::Table::new();
            table.insert("workspace", toml_edit::value(true));
            table.set_dotted(true);
            *item = table.into();
        }
    }

    for key in ["dev-dependencies", "build-dependencies"] {
        _ = document
            .entry(key)
            .or_insert_with(toml_edit::table)
            .as_table_mut()
            .with_context(|| format!("package.{key} was not a table in Cargo.toml"))?;
    }

    let workspace = document
        .entry("workspace")
        .or_insert_with(toml_edit::table)
        .as_table_mut()
        .context("workspace was not a table in Cargo.toml")?;
    _ = workspace
        .entry("resolver")
        .or_insert_with(|| toml_edit::value("3"));
    _ = workspace
        .entry("members")
        .or_insert_with(|| toml_edit::value(toml_edit::Array::new()));

    let workspace_package = workspace
        .entry("package")
        .or_insert_with(toml_edit::table)
        .as_table_mut()
        .context("workspace.package was not a table in Cargo.toml")?;
    _ = workspace_package
        .entry("version")
        .or_insert_with(|| toml_edit::value("0.1.0"));
    let mut authors = toml_edit::Array::new();
    authors.push(author);
    _ = workspace_package
        .entry("authors")
        .or_insert_with(|| toml_edit::value(authors));
    for (key, value) in [
        ("edition", "2024".into()),
        ("rust-version", "1.91".into()),
        (
            "repository",
            format!("https://github.com/{username}/{name}"),
        ),
        ("license", "MIT".into()),
    ] {
        _ = workspace_package
            .entry(key)
            .or_insert_with(|| toml_edit::value(value));
    }
    let keywords =
        toml_edit::Array::from_iter(["gear", "sails", "smart-contracts", "wasm", "no-std"]);
    _ = workspace_package
        .entry("keywords")
        .or_insert_with(|| toml_edit::value(keywords));
    let categories =
        toml_edit::Array::from_iter(["cryptography::cryptocurrencies", "no-std", "wasm"]);
    _ = workspace_package
        .entry("categories")
        .or_insert_with(|| toml_edit::value(categories));

    let dependencies = workspace
        .entry("dependencies")
        .or_insert_with(toml_edit::table)
        .as_table_mut()
        .context("workspace.dependencies was not a table in Cargo.toml")?;

    if let Some(sails_path) = sails_path {
        let mut dependency = toml_edit::InlineTable::new();
        dependency.insert(
            "path",
            sails_path
                .canonicalize()?
                .to_str()
                .context("failed to convert to UTF-8 string")?
                .into(),
        );
        dependencies.insert("sails-rs", dependency.into());
    } else {
        dependencies.insert("sails-rs", SAILS_VERSION.into());
    }

    dependencies.insert("tokio", TOKIO_VERSION.into());

    fs::write(manifest_path, document.to_string())?;

    Ok(())
}

fn ci_workflow_dir_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".github/workflows")
}

fn ci_workflow_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".github/workflows/ci.yml")
}

fn gitignore_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join(".gitignore")
}

fn manifest_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("Cargo.toml")
}

fn build_rs_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("build.rs")
}

fn lib_rs_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("src/lib.rs")
}

fn tests_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("tests/gtest.rs")
}

fn license_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("LICENSE")
}

fn readme_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("README.md")
}

fn rust_toolchain_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("rust-toolchain.toml")
}

fn git_command() -> String {
    env::var("GIT").unwrap_or("git".into())
}

fn cargo_command() -> String {
    env::var("CARGO").unwrap_or("cargo".into())
}
