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
#[template(path = "app/build.askama")]
struct AppBuild;

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
    program_crate_name: String,
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
}

pub struct ProgramGenerator {
    path: PathBuf,
    package_name: String,
    sails_path: Option<PathBuf>,
    app: bool,
    offline: bool,
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
        Self {
            path,
            package_name,
            sails_path,
            app,
            offline,
        }
    }

    fn app_build(&self) -> AppBuild {
        AppBuild
    }

    fn app_lib(&self) -> AppLib {
        AppLib {
            service_name: "Service".to_string(),
            service_name_snake: "service".to_string(),
            program_struct_name: "Program".to_string(),
        }
    }

    fn client_build(&self) -> ClientBuild {
        ClientBuild {
            program_crate_name: self.app_name().to_case(Case::Snake),
            program_struct_name: "Program".to_string(),
        }
    }

    fn client_lib(&self) -> ClientLib {
        ClientLib {
            client_file_name: format!("{}_client", self.package_name.to_case(Case::Snake)),
        }
    }

    fn tests_gtest(&self) -> TestsGtest {
        TestsGtest {
            program_crate_name: self.app_name().to_case(Case::Snake),
            client_crate_name: self.client_name().to_case(Case::Snake),
            client_program_name: self.client_name().to_case(Case::Pascal),
            service_name: "Service".to_string(),
        }
    }

    fn app_path(&self) -> PathBuf {
        if self.app {
            self.path.clone()
        } else {
            let mut path = self.path.clone();
            path.push("app");
            path
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
        let mut path = self.path.clone();
        path.push("client");
        path
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
        if let Some(path) = self.sails_path.as_ref() {
            let path = path.to_str().context("Invalid UTF-8 Path")?;
            let sails_package = &["--path", path];
            cargo_add(
                manifest_path.as_ref(),
                sails_package,
                dependency,
                features,
                self.offline,
            )
        } else {
            let sails_package = &[format!("sails-rs@{SAILS_VERSION}")];
            cargo_add(
                manifest_path.as_ref(),
                sails_package,
                dependency,
                features,
                self.offline,
            )
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        self.generate_app()?;
        if !self.app {
            self.generate_client()?;
            self.generate_tests()?;
        }
        self.fmt()?;
        Ok(())
    }

    fn generate_app(&self) -> anyhow::Result<()> {
        let path = self.app_path();
        cargo_new(&path, self.app_name().as_str(), self.offline)?;
        let manifest_path = manifest_path(&path);

        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(&manifest_path, Build, Some("wasm-builder"))?;

        let mut build_rs = File::create(build_rs_path(&path))?;
        self.app_build().write_into(&mut build_rs)?;

        let mut lib_rs = File::create(lib_rs_path(&path))?;
        self.app_lib().write_into(&mut lib_rs)?;

        Ok(())
    }

    fn generate_client(&self) -> anyhow::Result<()> {
        let path = self.client_path();
        cargo_new(&path, self.client_name().as_str(), self.offline)?;

        let manifest_path = manifest_path(&path);
        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(&manifest_path, Build, Some("build"))?;

        // add app ref
        cargo_add_by_path(&manifest_path, &self.app_path(), Build, None, self.offline)?;

        let mut build_rs = File::create(build_rs_path(&path))?;
        self.client_build().write_into(&mut build_rs)?;

        let mut lib_rs = File::create(lib_rs_path(&path))?;
        self.client_lib().write_into(&mut lib_rs)?;

        Ok(())
    }

    fn generate_tests(&self) -> anyhow::Result<()> {
        let path = &self.path;
        cargo_new(path, &self.package_name, self.offline)?;

        let manifest_path = manifest_path(path);
        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Development, Some("gtest,gclient"))?;

        // add tokio
        cargo_add(
            &manifest_path,
            ["tokio"],
            Development,
            Some("rt,macros"),
            self.offline,
        )?;

        // add app ref
        cargo_add_by_path(
            &manifest_path,
            &self.app_path(),
            Development,
            None,
            self.offline,
        )?;
        // add client ref
        cargo_add_by_path(
            &manifest_path,
            &self.client_path(),
            Development,
            None,
            self.offline,
        )?;

        // delete ./src folder
        let mut src_path = path.clone();
        src_path.push("src");
        fs::remove_dir_all(src_path)?;

        // add tests
        let test_path = tests_path(path);
        fs::create_dir_all(test_path.parent().context("Parent should exists")?)?;
        let mut gtest_rs = File::create(&test_path)?;
        self.tests_gtest().write_into(&mut gtest_rs)?;

        Ok(())
    }

    fn fmt(&self) -> anyhow::Result<ExitStatus> {
        let manifest_path = manifest_path(&self.path);
        cargo_fmt(&manifest_path)
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
        .context("failed to execute `cargo new` command")
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
        .context("failed to execute `cargo new` command")
}

fn cargo_add_by_path<P: AsRef<Path>>(
    manifest_path: P,
    crate_path: P,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
    offline: bool,
) -> anyhow::Result<ExitStatus> {
    let crate_path = crate_path.as_ref().to_str().context("Invalid UTF-8 Path")?;
    let package = &["--path", crate_path];
    cargo_add(manifest_path, package, dependency, features, offline)
}

fn manifest_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_path_buf();
    path.push("Cargo.toml");
    path
}

fn build_rs_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_path_buf();
    path.push("build.rs");
    path
}

fn lib_rs_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_path_buf();
    path.push("src/lib.rs");
    path
}

fn tests_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_path_buf();
    path.push("tests/gtest.rs");
    path
}

fn cargo_command() -> String {
    std::env::var("CARGO").unwrap_or("cargo".into())
}
