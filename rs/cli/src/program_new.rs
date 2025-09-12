use anyhow::Context;
use askama::Template;
use cargo_metadata::DependencyKind::{Build, Development, Normal};
use convert_case::{Case, Casing};
use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

const SAILS_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Template)]
#[template(path = "build.askama")]
struct RootBuild {
    app_crate_name: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "src/lib.askama")]
struct RootLib {
    app_crate_name: String,
}

#[derive(Template)]
#[template(path = "readme.askama")]
struct Readme {
    program_crate_name: String,
    app_crate_name: String,
    client_crate_name: String,
    service_name: String,
    app: bool,
}

#[derive(Template)]
#[template(path = "app/src/lib.askama")]
struct AppLib {
    service_name: String,
    service_name_snake: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "client/build.askama")]
struct ClientBuild {
    app_crate_name: String,
    program_struct_name: String,
}

#[derive(Template)]
#[template(path = "client/src/lib.askama")]
struct ClientLib {
    client_file_name: String,
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

pub struct ProgramGenerator {
    path: PathBuf,
    package_name: String,
    sails_path: Option<PathBuf>,
    app: bool,
    offline: bool,
    service_name: String,
    program_struct_name: String,
}

impl ProgramGenerator {
    pub fn new(
        path: PathBuf,
        name: Option<String>,
        sails_path: Option<PathBuf>,
        app: bool,
        offline: bool,
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
        Self {
            path,
            package_name,
            sails_path,
            app,
            offline,
            service_name,
            program_struct_name: "Program".to_string(),
        }
    }

    fn root_build(&self) -> RootBuild {
        RootBuild {
            app_crate_name: self.app_name().to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn root_lib(&self) -> RootLib {
        RootLib {
            app_crate_name: self.app_name().to_case(Case::Snake),
        }
    }

    fn root_readme(&self) -> Readme {
        Readme {
            program_crate_name: self.package_name.to_owned(),
            app_crate_name: self.app_name(),
            client_crate_name: self.client_name(),
            service_name: self.service_name.clone(),
            app: self.app,
        }
    }

    fn app_lib(&self) -> AppLib {
        AppLib {
            service_name: self.service_name.clone(),
            service_name_snake: self.service_name.to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn client_build(&self) -> ClientBuild {
        ClientBuild {
            app_crate_name: self.app_name().to_case(Case::Snake),
            program_struct_name: self.program_struct_name.clone(),
        }
    }

    fn client_lib(&self) -> ClientLib {
        ClientLib {
            client_file_name: format!("{}_client", self.package_name.to_case(Case::Snake)),
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

    fn app_path(&self) -> PathBuf {
        if self.app {
            self.path.clone()
        } else {
            self.path.join("app")
        }
    }

    fn app_name(&self) -> String {
        if self.app {
            self.package_name.clone()
        } else {
            format!("{}-app", self.package_name)
        }
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
    ) -> anyhow::Result<ExitStatus> {
        if let Some(sails_path) = self.sails_path.as_ref() {
            cargo_add_by_path(
                manifest_path,
                sails_path,
                dependency,
                features,
                self.offline,
            )
        } else {
            let sails_package = &[format!("sails-rs@{SAILS_VERSION}")];
            cargo_add(
                manifest_path,
                sails_package,
                dependency,
                features,
                self.offline,
            )
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        if self.app {
            self.generate_app()?;
        } else {
            self.generate_root()?;
            self.generate_app()?;
            self.generate_client()?;
            self.generate_build()?;
            self.generate_tests()?;
        }
        self.fmt()?;
        Ok(())
    }

    fn generate_app(&self) -> anyhow::Result<()> {
        let path = &self.app_path();
        cargo_new(path, &self.app_name(), self.offline)?;
        let manifest_path = &manifest_path(path);

        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, None)?;

        let mut lib_rs = File::create(lib_rs_path(path))?;
        self.app_lib().write_into(&mut lib_rs)?;

        Ok(())
    }

    fn generate_root(&self) -> anyhow::Result<()> {
        let path = &self.path;
        cargo_new(path, &self.package_name, self.offline)?;

        let manifest_path = &manifest_path(path);
        cargo_toml_create_workspace(manifest_path)?;

        let mut readme_md = File::create(readme_path(path))?;
        self.root_readme().write_into(&mut readme_md)?;

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
        cargo_add_by_path(manifest_path, self.app_path(), Normal, None, self.offline)?;
        cargo_add_by_path(manifest_path, self.app_path(), Build, None, self.offline)?;
        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(manifest_path, Build, Some("build"))?;

        Ok(())
    }

    fn generate_client(&self) -> anyhow::Result<()> {
        let path = &self.client_path();
        cargo_new(path, &self.client_name(), self.offline)?;

        let manifest_path = &manifest_path(path);
        // add sails-rs refs
        self.cargo_add_sails_rs(manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(manifest_path, Build, Some("build"))?;

        // add app ref
        cargo_add_by_path(manifest_path, self.app_path(), Build, None, self.offline)?;

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
        self.cargo_add_sails_rs(manifest_path, Development, Some("gtest,gclient"))?;

        // add tokio
        cargo_add(
            manifest_path,
            ["tokio"],
            Development,
            Some("rt,macros"),
            self.offline,
        )?;

        // add app ref
        cargo_add_by_path(
            manifest_path,
            self.app_path(),
            Development,
            None,
            self.offline,
        )?;
        // add client ref
        cargo_add_by_path(
            manifest_path,
            self.client_path(),
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

    fn fmt(&self) -> anyhow::Result<ExitStatus> {
        let manifest_path = &manifest_path(&self.path);
        cargo_fmt(manifest_path)
    }
}

fn cargo_new<P: AsRef<Path>>(
    target_dir: P,
    name: &str,
    offline: bool,
) -> anyhow::Result<ExitStatus> {
    let cargo_command = cargo_command();
    let target_dir = target_dir.as_ref();
    let cargo_new_or_init = if target_dir.exists() { "init" } else { "new" };
    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
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
        .context("failed to execute `cargo new` command")
}

fn cargo_add<P, I, S>(
    manifest_path: P,
    packages: I,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
    offline: bool,
) -> anyhow::Result<ExitStatus>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cargo_command = cargo_command();

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("add")
        .args(packages)
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
        .context("failed to execute `cargo add` command")
}

fn cargo_fmt<P: AsRef<Path>>(manifest_path: P) -> anyhow::Result<ExitStatus> {
    let cargo_command = cargo_command();

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo fmt` command")
}

fn cargo_add_by_path<P1: AsRef<Path>, P2: AsRef<Path>>(
    manifest_path: P1,
    crate_path: P2,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
    offline: bool,
) -> anyhow::Result<ExitStatus> {
    let crate_path = crate_path.as_ref().to_str().context("Invalid UTF-8 Path")?;
    let package = &["--path", crate_path];
    cargo_add(manifest_path, package, dependency, features, offline)
}

fn cargo_toml_create_workspace<P: AsRef<Path>>(manifest_path: P) -> anyhow::Result<()> {
    let manifest_path = manifest_path.as_ref();
    let cargo_toml = fs::read_to_string(manifest_path)?;
    let mut document: toml_edit::DocumentMut = cargo_toml.parse()?;

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
    _ = workspace_package
        .entry("edition")
        .or_insert_with(|| toml_edit::value("2024"));

    fs::write(manifest_path, document.to_string())?;

    Ok(())
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

fn readme_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("README.md")
}

fn cargo_command() -> String {
    std::env::var("CARGO").unwrap_or("cargo".into())
}
