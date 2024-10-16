use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use anyhow::Context;
use cargo_metadata::{camino::*, Package};

pub struct CrateIdlGenerator {
    manifest_path: Utf8PathBuf,
    target_dir: Option<PathBuf>,
}

impl CrateIdlGenerator {
    pub fn new(manifest_path: PathBuf, target_dir: Option<PathBuf>) -> Self {
        Self {
            manifest_path: Utf8PathBuf::from_path_buf(manifest_path).unwrap(),
            target_dir,
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        println!("...reading metadata: {}", &self.manifest_path);
        // Get metadata with deps
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            .exec()?;

        let sails_package = metadata
            .packages
            .iter()
            .find(|p| p.name == "sails-rs")
            .unwrap();

        //self.cargo_doc()
        // print!("{:?}", sails_package);

        let program_package = metadata.root_package().unwrap();
        let program_package_file_name = program_package.name.to_lowercase().replace('-', "_");
        // print!("{:?}", program_package);

        let target_dir = self
            .target_dir
            .map(|p| p.canonicalize().ok())
            .flatten()
            .map(Utf8PathBuf::from_path_buf)
            .map(|t| t.ok())
            .flatten()
            .unwrap_or_else(|| metadata.target_directory.clone());

        _ = cargo_doc(&self.manifest_path, &target_dir)?;
        let docs_path = target_dir
            .join("doc")
            .join(format!("{}.json", &program_package_file_name));

        println!("...reading docs: {:?}", docs_path);
        let json_string = std::fs::read_to_string(docs_path)?;
        let doc_crate: rustdoc_types::Crate = serde_json::from_str(&json_string)?;
        let program_meta = doc_crate
            .paths
            .iter()
            .find(|p| p.1.path == META_PATH_V2)
            .context("failed to find `sails_rs::meta::ProgramMeta` definition in dependencies")?;
        // println!("{:?}", program_meta);

        let program_struct_path = find_program_path(&doc_crate, program_meta)
            .context("failed to find `sails_rs::meta::ProgramMeta` implemetation")?;
        let program_struct = doc_crate
            .paths
            .get(&program_struct_path.id)
            .context("failed to find Program path")?;
        let program_struct_path = program_struct.path.join("::");
        println!("...found Program implemetation: {:?}", program_struct_path);

        let crate_name = get_crate_name(program_package);

        let crate_dir = target_dir.join(&crate_name);
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let gen_manifest_path = crate_dir.join("Cargo.toml");
        write_file(&gen_manifest_path, gen_toml(program_package, sails_package))?;
        let main_rs_path = src_dir.join("main.rs");

        let out_file = target_dir.join(format!("{}.idl", program_package.name));
        write_file(
            main_rs_path,
            gen_main_rs(&program_struct_path, out_file.as_path()),
        )?;

        // Copy original `Cargo.lock` if any
        let from_lock = &metadata.workspace_root.join("Cargo.lock");
        let to_lock = &crate_dir.join("Cargo.lock");
        drop(fs::copy(from_lock, to_lock));

        // execute cargo run on generated manifest
        _ = cargo_run_bin(&gen_manifest_path, &crate_name, &target_dir)?;

        // remove generated files
        fs::remove_dir_all(crate_dir)?;

        Ok(())
    }
}

const META_PATH_V1: &[&str] = &["sails_rs", "meta", "ProgramMeta"];
const META_PATH_V2: &[&str] = &["sails_idl_meta", "ProgramMeta"];

fn find_program_path(
    doc_crate: &rustdoc_types::Crate,
    program_meta: (&rustdoc_types::Id, &rustdoc_types::ItemSummary),
) -> Option<rustdoc_types::Path> {
    let program_struct_path = doc_crate.index.values().find_map(|idx| {
        if let rustdoc_types::ItemEnum::Impl(item) = &idx.inner {
            if let Some(tp) = &item.trait_ {
                if &tp.id == program_meta.0 {
                    if let rustdoc_types::Type::ResolvedPath(path) = &item.for_ {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    });
    program_struct_path.cloned()
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

    manifest["workspace"] = toml_edit::Item::Table(toml_edit::Table::new());

    manifest.to_string()
}

fn gen_main_rs(program_struct_path: &str, out_file: &cargo_metadata::camino::Utf8Path) -> String {
    format!(
        "
fn main() {{
    sails_rs::generate_idl_to_file::<{}>(
        std::path::PathBuf::from(r\"{}\")
    )
    .unwrap();
}}",
        program_struct_path,
        out_file.as_str(),
    )
}

fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> anyhow::Result<()> {
    let path = path.as_ref();
    fs::write(path, contents.as_ref())
        .with_context(|| format!("failed to write `{}`", path.display()))
}

fn cargo_doc(
    manifest_path: &cargo_metadata::camino::Utf8Path,
    target_dir: &cargo_metadata::camino::Utf8Path,
) -> anyhow::Result<ExitStatus> {
    let cargo_path = std::env::var("CARGO").unwrap_or("cargo".into());

    let mut cmd = Command::new(cargo_path);
    cmd.env("RUSTC_BOOTSTRAP", "1")
        .env(
            "RUSTDOCFLAGS",
            "-Z unstable-options --output-format=json --cap-lints=allow",
        )
        .env("__GEAR_WASM_BUILDER_NO_BUILD", "1")
        .stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("doc")
        .arg("--manifest-path")
        .arg(manifest_path.as_str())
        .arg("--target-dir")
        .arg(target_dir.as_str())
        .arg("--no-deps");

    cmd.status()
        .context("failed to execute `cargo doc` command")
}

fn cargo_run_bin(
    manifest_path: &cargo_metadata::camino::Utf8Path,
    bin_name: &str,
    target_dir: &cargo_metadata::camino::Utf8Path,
) -> anyhow::Result<ExitStatus> {
    let cargo_path = std::env::var("CARGO").unwrap_or("cargo".into());

    let mut cmd = Command::new(cargo_path);
    cmd.env("CARGO_TARGET_DIR", &target_dir)
        .env("__GEAR_WASM_BUILDER_NO_BUILD", "1")
        .stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("run")
        .arg("--manifest-path")
        .arg(manifest_path.as_str())
        .arg("--bin")
        .arg(bin_name);
    cmd.status().context("failed to execute `cargo` command")
}
