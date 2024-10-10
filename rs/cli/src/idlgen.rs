use std::{
    fs,
    path::{Path, PathBuf},
    process::{exit, Command, ExitStatus},
};

use anyhow::Context;
use cargo_metadata::Package;

pub struct CrateIdlGenerator {
    manifest_path: PathBuf,
    target_dir: Option<PathBuf>,
}

impl CrateIdlGenerator {
    pub fn new(manifest_path: PathBuf, target_dir: Option<PathBuf>) -> Self {
        Self {
            manifest_path,
            target_dir,
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            //.no_deps()
            .exec()?;

        let sails_package = metadata
            .packages
            .iter()
            .find(|p| p.name == "sails-rs")
            .unwrap();

        //self.cargo_doc()
        // print!("{:?}", sails_package);

        let program_package = metadata
            .packages
            .iter()
            .find(|p| &p.id == metadata.workspace_default_members.first().unwrap())
            .unwrap();
        // print!("{:?}", program_package);
        let crate_name = get_crate_name(program_package);
        let workspace_root = metadata.workspace_root;

        let crate_path = workspace_root.join(&crate_name);
        let src_path = crate_path.join("src");
        fs::create_dir_all(&src_path)?;

        let manifest_path = crate_path.join("Cargo.toml");
        write_file(&manifest_path, gen_toml(program_package, sails_package))?;
        let main_rs_path = src_path.join("main.rs");

        let target_dir = self
            .target_dir
            .unwrap_or_else(|| metadata.target_directory.clone().into_std_path_buf());
        let out_file = target_dir.join(format!("{}.idl", program_package.name));
        write_file(
            main_rs_path,
            gen_main_rs("proxy::ProxyProgram", out_file.as_path()),
        )?;

        workspace_members_add(&workspace_root.as_std_path(), &crate_name)?;

        _ = cargo_run_bin(&manifest_path, &crate_name)?;

        workspace_members_remove(&workspace_root.as_std_path(), &crate_name)?;

        fs::remove_dir_all(crate_path)?;

        Ok(())
    }

    fn cargo_doc(&self) -> anyhow::Result<()> {
        let cargo_path = std::env::var("CARGO").unwrap_or("cargo".into());

        let mut cmd = Command::new(cargo_path);
        cmd.env("RUSTC_BOOTSTRAP", "1")
            .env(
                "RUSTDOCFLAGS",
                "-Z unstable-options --document-private-items --document-hidden-items --output-format=json --cap-lints=allow",
            )
            .stdout(std::process::Stdio::null()) // Don't pollute output
            .arg("doc")
            .arg("--manifest-path")
            .arg(self.manifest_path.to_str().unwrap())
            .arg("--no-deps");

        let status = match cmd
            .status()
            .context("Failed to execute `cargo doc` command")
        {
            Ok(status) => status,
            Err(e) => {
                // let _ = display_error(&e, &mut shell);
                exit(1);
            }
        };

        exit(status.code().unwrap_or(1));
    }
}

fn get_crate_name(program_package: &Package) -> String {
    format!("{}-idl-gen", program_package.name)
}

fn gen_toml(program_package: &Package, sails_package: &Package) -> String {
    let mut manifest = toml_edit::DocumentMut::new();
    manifest["package"] = toml_edit::Item::Table(toml_edit::Table::new());
    manifest["package"]["name"] = toml_edit::value(get_crate_name(program_package));
    manifest["package"]["version"] = toml_edit::value("0.1.0");
    manifest["package"]["edition"] = toml_edit::value(program_package.edition.as_str());

    let mut dep_table = toml_edit::Table::default();
    let mut package_table = toml_edit::InlineTable::new();
    let manifets_dir = program_package.manifest_path.parent().unwrap();
    package_table.insert("path", manifets_dir.as_str().into());
    dep_table[&program_package.name] = toml_edit::value(package_table);

    let mut features = toml_edit::Array::default();
    features.push("idl-gen");
    let mut sails_table = toml_edit::InlineTable::new();
    let manifets_dir = sails_package.manifest_path.parent().unwrap();
    sails_table.insert("path", manifets_dir.as_str().into());
    sails_table.insert("features", features.into());
    dep_table[&sails_package.name] = toml_edit::value(sails_table);

    manifest["dependencies"] = toml_edit::Item::Table(dep_table);

    let mut bin = toml_edit::Table::new();
    bin["name"] = toml_edit::value(get_crate_name(program_package));
    bin["path"] = toml_edit::value("src/main.rs");
    manifest["bin"]
        .or_insert(toml_edit::Item::ArrayOfTables(
            toml_edit::ArrayOfTables::new(),
        ))
        .as_array_of_tables_mut()
        .expect("bin is an array of tables")
        .push(bin);

    manifest.to_string()
}

fn gen_main_rs(program_struct_path: &str, out_file: &Path) -> String {
    format!(
        "
fn main() {{
    sails_rs::generate_idl_to_file::<{}>(
        std::path::PathBuf::from(r\"{}\")
    )
    .unwrap();
}}",
        program_struct_path,
        out_file.to_str().unwrap(),
    )
}

fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> anyhow::Result<()> {
    let path = path.as_ref();
    fs::write(path, contents.as_ref())
        .with_context(|| format!("failed to write `{}`", path.display()))
}

fn workspace_members_add(path: &Path, name: &str) -> anyhow::Result<()> {
    println!("adding member to workspace: {:?}", name);

    let workspace_cargo_toml = path.join("Cargo.toml");
    let toml = fs::read_to_string(&workspace_cargo_toml).context("failed to read Cargo.toml")?;
    let mut doc = toml
        .parse::<toml_edit::DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    let members =
        doc["workspace"]["members"].or_insert(toml_edit::value(toml_edit::Array::default()));
    members.as_array_mut().unwrap().push(name);
    write_file(&workspace_cargo_toml, doc.to_string())
}

fn workspace_members_remove(path: &Path, name: &str) -> anyhow::Result<()> {
    println!("removing member from workspace: {:?}", name);

    let workspace_cargo_toml = path.join("Cargo.toml");
    let toml = fs::read_to_string(&workspace_cargo_toml).context("failed to read Cargo.toml")?;
    let mut doc = toml
        .parse::<toml_edit::DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    let members = doc["workspace"]["members"].as_array_mut();

    if let Some(members) = members {
        let position = members.iter().position(|m| m.as_str() == Some(name));
        if let Some(position) = position {
            members.remove(position);
            write_file(&workspace_cargo_toml, doc.to_string())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn cargo_run_bin(
    manifest_path: &cargo_metadata::camino::Utf8Path,
    bin_name: &str,
) -> anyhow::Result<ExitStatus> {
    let cargo_path = std::env::var("CARGO").unwrap_or("cargo".into());

    let args = vec![
        "run",
        "--manifest-path",
        manifest_path.as_str(),
        "--bin",
        bin_name,
    ];
    let mut cmd = Command::new(cargo_path);
    cmd.args(args);
    cmd.status().context("Failed to execute `cargo` command")
}
