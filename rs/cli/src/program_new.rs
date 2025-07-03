use anyhow::Context;
use askama::Template;
use cargo_metadata::DependencyKind::{Build, Development, Normal};
use convert_case::{Case, Casing};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

const SAILS_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct ProgramGenerator {
    path: PathBuf,
    package_name: String,
    sails_path: Option<PathBuf>,
}

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

impl ProgramGenerator {
    pub fn new(path: PathBuf, name: Option<String>, sails_path: Option<PathBuf>) -> Self {
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
        let mut manifest_path = self.path.clone();
        manifest_path.push("app");
        manifest_path
    }

    fn app_name(&self) -> String {
        format!("{}-app", self.package_name)
    }

    fn client_path(&self) -> PathBuf {
        let mut manifest_path = self.path.clone();
        manifest_path.push("client");
        manifest_path
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
            let path = path.to_str().expect("Invalid UTF-8 Path");
            let sails_package = &["--path", path];
            cargo_add(manifest_path.as_ref(), sails_package, dependency, features)
        } else {
            let sails_package = &[format!("sails-rs@{SAILS_VERSION}")];
            cargo_add(manifest_path.as_ref(), sails_package, dependency, features)
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        self.generate_app()?;
        self.generate_client()?;
        self.generate_tests()?;
        self.fmt()?;
        Ok(())
    }

    fn generate_app(&self) -> anyhow::Result<()> {
        let path = self.app_path();
        cargo_new(&path, self.app_name().as_str())?;
        let manifest_path = manifest_path(&path);

        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(&manifest_path, Build, Some("wasm-builder"))?;

        let build_rs = self.app_build().render()?;
        fs::write(build_rs_path(&path), build_rs)?;

        let lib_rs = self.app_lib().render()?;
        fs::write(lib_rs_path(&path), lib_rs)?;

        Ok(())
    }

    fn generate_client(&self) -> anyhow::Result<()> {
        let path = self.client_path();
        cargo_new(&path, self.client_name().as_str())?;

        let manifest_path = manifest_path(&path);
        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Normal, None)?;
        self.cargo_add_sails_rs(&manifest_path, Build, Some("build"))?;

        // add app ref
        cargo_add_by_path(&manifest_path, &self.app_path(), Build, None)?;

        let build_rs = self.client_build().render()?;
        fs::write(build_rs_path(&path), build_rs)?;

        let lib_rs = self.client_lib().render()?;
        fs::write(lib_rs_path(&path), lib_rs)?;

        Ok(())
    }

    fn generate_tests(&self) -> anyhow::Result<()> {
        let path = &self.path;
        cargo_new(path, &self.package_name)?;

        let manifest_path = manifest_path(path);
        // add sails-rs refs
        self.cargo_add_sails_rs(&manifest_path, Development, Some("gtest,gclient"))?;

        // add tokio
        cargo_add(&manifest_path, ["tokio"], Development, Some("rt,macros"))?;

        // add app ref
        cargo_add_by_path(&manifest_path, &self.app_path(), Development, None)?;
        // add client ref
        cargo_add_by_path(&manifest_path, &self.client_path(), Development, None)?;

        let mut src_path = path.clone();
        src_path.push("src");
        fs::remove_dir_all(src_path)?;

        let test_path = tests_path(path);
        fs::create_dir_all(test_path.parent().expect("Parent exists"))?;
        let tests = self.tests_gtest().render()?;
        fs::write(&test_path, tests)?;

        Ok(())
    }

    fn fmt(&self) -> anyhow::Result<ExitStatus> {
        let manifest_path = manifest_path(&self.path);
        cargo_fmt(&manifest_path)
    }
}

fn cargo_new<P: AsRef<Path>>(target_dir: P, name: &str) -> anyhow::Result<ExitStatus> {
    let cargo_command = cargo_command();
    let target_dir = target_dir.as_ref();
    let cargo_new_or_init = if target_dir.exists() { "init" } else { "new" };
    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
        .arg(cargo_new_or_init)
        .arg(target_dir.to_str().expect("Invalid UTF-8 Path"))
        .arg("--name")
        .arg(name)
        .arg("--lib")
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo new` command")
}

fn cargo_add<I, S>(
    manifest_path: &Path,
    packages: I,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
) -> anyhow::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cargo_command = cargo_command();

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("add")
        .args(packages)
        .arg("--manifest-path")
        .arg(manifest_path.to_str().expect("Invalid UTF-8 Path"))
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

    cmd.status()
        .context("failed to execute `cargo new` command")
}

fn cargo_fmt<P: AsRef<Path>>(manifest_path: P) -> anyhow::Result<ExitStatus> {
    let cargo_command = cargo_command();

    let mut cmd = Command::new(cargo_command);
    cmd.stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref().to_str().expect("Invalid UTF-8 Path"))
        .arg("--quiet");

    cmd.status()
        .context("failed to execute `cargo new` command")
}

fn cargo_add_by_path<P: AsRef<Path>>(
    manifest_path: P,
    crate_path: P,
    dependency: cargo_metadata::DependencyKind,
    features: Option<&str>,
) -> anyhow::Result<ExitStatus> {
    let crate_path = crate_path.as_ref().to_str().expect("Invalid UTF-8 Path");
    let package = &["--path", crate_path];
    cargo_add(manifest_path.as_ref(), package, dependency, features)
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
