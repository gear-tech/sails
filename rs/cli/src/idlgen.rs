use anyhow::Context;
use cargo_metadata::{camino::*, Package};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

pub struct CrateIdlGenerator {
    manifest_path: Utf8PathBuf,
    target_dir: Option<Utf8PathBuf>,
    workspace: bool,
}

impl CrateIdlGenerator {
    pub fn new(
        manifest_path: Option<PathBuf>,
        target_dir: Option<PathBuf>,
        workspace: bool,
    ) -> Self {
        Self {
            manifest_path: Utf8PathBuf::from_path_buf(
                manifest_path.unwrap_or_else(|| env::current_dir().unwrap().join("Cargo.toml")),
            )
            .unwrap(),
            target_dir: target_dir
                .and_then(|p| p.canonicalize().ok())
                .map(Utf8PathBuf::from_path_buf)
                .and_then(|t| t.ok()),
            workspace,
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        println!("...reading metadata: {}", &self.manifest_path);
        // get metadata with deps
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            .exec()?;

        // find `sails-rs` package
        let sails_package = metadata
            .packages
            .iter()
            .find(|p| p.name == "sails-rs")
            .context("failed to find `sails-rs` package in dependencies")?;

        let target_dir = self
            .target_dir
            .as_ref()
            .unwrap_or(&metadata.target_directory);

        // get all workspace packages or only default
        let package_list = if self.workspace {
            metadata.workspace_packages()
        } else {
            metadata.workspace_default_packages()
        };

        for program_package in package_list {
            let idl_gen = PackageIdlGenerator::new(
                program_package,
                sails_package,
                target_dir,
                &metadata.workspace_root,
            );
            match idl_gen.read_program_path_from_doc() {
                Ok((program_struct_path, meta_path_version)) => {
                    println!("...found Program implemetation: {}", program_struct_path);
                    let file_path = idl_gen
                        .try_generate_for_package(&program_struct_path, meta_path_version)?;
                    println!("Generated IDL: {}", file_path);
                }
                Err(err) => {
                    println!("...no Program implementation found: {}", err);
                    if !self.workspace {
                        println!("Try run `cargo sails idl-gen --workspace` to generate IDL for all packages in the workspace");
                    }
                }
            }
        }

        Ok(())
    }
}

struct PackageIdlGenerator<'a> {
    program_package: &'a Package,
    sails_package: &'a Package,
    target_dir: &'a Utf8Path,
    workspace_root: &'a Utf8Path,
}

impl<'a> PackageIdlGenerator<'a> {
    fn new(
        program_package: &'a Package,
        sails_package: &'a Package,
        target_dir: &'a Utf8Path,
        workspace_root: &'a Utf8Path,
    ) -> Self {
        Self {
            program_package,
            sails_package,
            target_dir,
            workspace_root,
        }
    }

    fn read_program_path_from_doc(&self) -> anyhow::Result<(String, MetaPathVersion)> {
        let program_package_file_name = &self.program_package.name.to_lowercase().replace('-', "_");
        println!(
            "...running `cargo doc --manifest-path {}`",
            &self.program_package.manifest_path
        );
        // run `cargo doc`
        _ = cargo_doc(&self.program_package.manifest_path, self.target_dir)?;
        // read doc
        let docs_path = &self
            .target_dir
            .join("doc")
            .join(format!("{}.json", &program_package_file_name));
        println!("...reading docs: {}", docs_path);
        let json_string = std::fs::read_to_string(docs_path)?;
        let doc_crate: rustdoc_types::Crate = serde_json::from_str(&json_string)?;

        // find `sails_rs::meta::ProgramMeta` path id
        let (program_meta_id, meta_path_version) = doc_crate
            .paths
            .iter()
            .find_map(|p| {
                if p.1.path == META_PATH_V1 {
                    Some((p.0, MetaPathVersion::V1))
                } else if p.1.path == META_PATH_V2 {
                    Some((p.0, MetaPathVersion::V2))
                } else {
                    None
                }
            })
            .context("failed to find `sails_rs::meta::ProgramMeta` definition in dependencies")?;
        // find struct implementing `sails_rs::meta::ProgramMeta`
        let program_struct_path = doc_crate
            .index
            .values()
            .find_map(|idx| is_implement_trait_path(idx, program_meta_id))
            .context("failed to find `sails_rs::meta::ProgramMeta` implemetation")?;
        let program_struct = doc_crate
            .paths
            .get(&program_struct_path.id)
            .context("failed to get Program struct by id")?;
        let program_struct_path = program_struct.path.join("::");
        Ok((program_struct_path, meta_path_version))
    }

    fn try_generate_for_package(
        &self,
        program_struct_path: &str,
        meta_path_version: MetaPathVersion,
    ) -> Result<Utf8PathBuf, anyhow::Error> {
        let crate_name = get_crate_name(self.program_package);
        let crate_dir = &self.target_dir.join(&crate_name);
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let gen_manifest_path = crate_dir.join("Cargo.toml");
        write_file(
            &gen_manifest_path,
            gen_cargo_toml(self.program_package, self.sails_package, meta_path_version),
        )?;

        let out_file = self
            .target_dir
            .join(format!("{}.idl", &self.program_package.name));
        let main_rs_path = src_dir.join("main.rs");
        write_file(main_rs_path, gen_main_rs(program_struct_path, &out_file))?;

        let from_lock = &self.workspace_root.join("Cargo.lock");
        let to_lock = &crate_dir.join("Cargo.lock");
        drop(fs::copy(from_lock, to_lock));

        _ = cargo_run_bin(&gen_manifest_path, &crate_name, self.target_dir)?;

        fs::remove_dir_all(crate_dir)?;

        Ok(out_file)
    }
}

fn is_implement_trait_path(
    idx: &rustdoc_types::Item,
    program_meta_id: &rustdoc_types::Id,
) -> Option<rustdoc_types::Path> {
    if let rustdoc_types::ItemEnum::Impl(item) = &idx.inner {
        if let Some(tp) = &item.trait_ {
            if &tp.id == program_meta_id {
                if let rustdoc_types::Type::ResolvedPath(path) = &item.for_ {
                    return Some(path.clone());
                }
            }
        }
    }
    None
}

fn get_crate_name(program_package: &Package) -> String {
    format!("{}-idl-gen", program_package.name)
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
        .arg("--no-deps")
        .arg("--quiet");

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
    cmd.env("CARGO_TARGET_DIR", target_dir)
        .env("__GEAR_WASM_BUILDER_NO_BUILD", "1")
        .stdout(std::process::Stdio::null()) // Don't pollute output
        .arg("run")
        .arg("--manifest-path")
        .arg(manifest_path.as_str())
        .arg("--bin")
        .arg(bin_name);
    cmd.status().context("failed to execute `cargo` command")
}

enum MetaPathVersion {
    V1,
    V2,
}

pub const META_PATH_V1: &[&str] = &["sails_rs", "meta", "ProgramMeta"];
pub const META_PATH_V2: &[&str] = &["sails_idl_meta", "ProgramMeta"];

fn gen_cargo_toml(
    program_package: &Package,
    sails_package: &Package,
    meta_path_version: MetaPathVersion,
) -> String {
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

    let sails_dep = match meta_path_version {
        MetaPathVersion::V1 => sails_dep_v1(sails_package),
        MetaPathVersion::V2 => sails_dep_v2(sails_package),
    };
    dep_table[&sails_package.name] = toml_edit::value(sails_dep);

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

fn sails_dep_v1(sails_package: &Package) -> toml_edit::InlineTable {
    let mut sails_table = toml_edit::InlineTable::new();
    sails_table.insert("package", "sails-idl-gen".into());
    sails_table.insert("version", sails_package.version.to_string().into());
    sails_table
}

fn sails_dep_v2(sails_package: &Package) -> toml_edit::InlineTable {
    let mut features = toml_edit::Array::default();
    features.push("idl-gen");
    let mut sails_table = toml_edit::InlineTable::new();
    let manifets_dir = sails_package.manifest_path.parent().unwrap();
    sails_table.insert("package", sails_package.name.as_str().into());
    sails_table.insert("path", manifets_dir.as_str().into());
    sails_table.insert("features", features.into());
    sails_table
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
