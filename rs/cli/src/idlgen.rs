use anyhow::Context;
use cargo_metadata::{DependencyKind, Package, PackageId, TargetKind, camino::*};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

type DocCacheKey = (Utf8PathBuf, String);

pub struct CrateIdlGenerator {
    manifest_path: Utf8PathBuf,
    target_dir: Option<Utf8PathBuf>,
    deps_level: usize,
}

impl CrateIdlGenerator {
    pub fn new(
        manifest_path: Option<PathBuf>,
        target_dir: Option<PathBuf>,
        deps_level: Option<usize>,
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
            deps_level: deps_level.unwrap_or(1),
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        generate_program_artifact(
            &self.manifest_path,
            self.target_dir.as_ref().map(|p| p.as_path()),
            self.deps_level,
            ProgramArtifactKind::Idl,
        )
        .map(|file| {
            println!("Generated IDL: {file}");
        })
    }
}

pub enum ProgramArtifactKind {
    Idl,
    Canonical,
}

pub fn generate_program_artifact(
    manifest_path: &Utf8Path,
    target_dir: Option<&Utf8Path>,
    deps_level: usize,
    kind: ProgramArtifactKind,
) -> anyhow::Result<Utf8PathBuf> {
    println!("...reading metadata: {manifest_path}");
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()?;
    let metadata_fingerprint = metadata_fingerprint(&metadata)?;

    let target_dir = target_dir.unwrap_or(&metadata.target_directory);
    let doc_cache_dir = target_dir.join("sails").join("doc-cache");

    let sails_rs_packages = metadata
        .packages
        .iter()
        .filter(|&p| p.name == "sails-rs")
        .collect::<Vec<_>>();
    let sails_interface_id_packages = metadata
        .packages
        .iter()
        .filter(|&p| p.name == "sails-interface-id")
        .collect::<Vec<_>>();

    let package_list = get_package_list(&metadata, deps_level)?;
    let mut doc_cache: HashSet<DocCacheKey> = HashSet::new();

    let mut candidate_packages = Vec::new();
    for package in package_list {
        if let Some(reason) = skip_reason_for_package(package) {
            log::debug!("skipping package `{}`: {}", package.name, reason);
            continue;
        }
        candidate_packages.push(package);
    }

    println!(
        "...looking for Program implementation in {} package(s)",
        candidate_packages.len()
    );
    for program_package in candidate_packages {
        let generator = PackageGenerator::new(
            program_package,
            &sails_rs_packages,
            &sails_interface_id_packages,
            target_dir,
            &metadata.workspace_root,
        );
        match get_program_struct_path_from_doc(
            program_package,
            target_dir,
            &metadata_fingerprint,
            &mut doc_cache,
            &doc_cache_dir,
        ) {
            Ok((program_struct_path, meta_path_version)) => {
                println!("...found Program implementation: {program_struct_path}");
                match generator.try_generate_for_package(
                    &program_struct_path,
                    meta_path_version,
                    &kind,
                ) {
                    Ok(file_path) => return Ok(file_path),
                    Err(err) => println!("...failed to generate artifact: {err}"),
                }
            }
            Err(err) => {
                log::debug!(
                    "...no Program implementation found in `{}`: {err}",
                    program_package.name
                );
            }
        }
    }
    Err(anyhow::anyhow!("no Program implementation found"))
}

struct PackageGenerator<'a> {
    program_package: &'a Package,
    sails_rs_packages: &'a Vec<&'a Package>,
    sails_interface_id_packages: &'a Vec<&'a Package>,
    target_dir: &'a Utf8Path,
    workspace_root: &'a Utf8Path,
}

impl<'a> PackageGenerator<'a> {
    fn new(
        program_package: &'a Package,
        sails_rs_packages: &'a Vec<&'a Package>,
        sails_interface_id_packages: &'a Vec<&'a Package>,
        target_dir: &'a Utf8Path,
        workspace_root: &'a Utf8Path,
    ) -> Self {
        Self {
            program_package,
            sails_rs_packages,
            sails_interface_id_packages,
            target_dir,
            workspace_root,
        }
    }

    fn try_generate_for_package(
        &self,
        program_struct_path: &str,
        meta_path_version: MetaPathVersion,
        kind: &ProgramArtifactKind,
    ) -> anyhow::Result<Utf8PathBuf> {
        if matches!(kind, ProgramArtifactKind::Canonical)
            && meta_path_version == MetaPathVersion::V1
        {
            anyhow::bail!("canonical artifact generation requires Sails >= 0.9.0");
        }

        // find `sails-rs` dependency
        let sails_dep = self
            .program_package
            .dependencies
            .iter()
            .find(|p| p.name == "sails-rs")
            .context("failed to find `sails-rs` dependency")?;
        // find `sails-rs` package matches dep version
        let sails_package = self
            .sails_rs_packages
            .iter()
            .find(|p| sails_dep.req.matches(&p.version))
            .context(format!(
                "failed to find `sails-rs` package with matching version {}",
                &sails_dep.req
            ))?;
        let sails_interface_package = self
            .sails_interface_id_packages
            .iter()
            .find(|p| sails_dep.req.matches(&p.version))
            .copied();

        let crate_name = get_generator_crate_name(kind, self.program_package);
        let crate_dir = &self.target_dir.join(&crate_name);
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let gen_manifest_path = crate_dir.join("Cargo.toml");
        write_file(
            &gen_manifest_path,
            gen_cargo_toml(
                self.program_package,
                sails_package,
                sails_interface_package,
                meta_path_version,
                kind,
            ),
        )?;

        let out_file = get_artifact_output_path(kind, self.target_dir, self.program_package);
        let main_rs_path = src_dir.join("main.rs");
        write_file(
            main_rs_path,
            gen_main_rs(kind, program_struct_path, &out_file),
        )?;

        let from_lock = &self.workspace_root.join("Cargo.lock");
        let to_lock = &crate_dir.join("Cargo.lock");
        drop(fs::copy(from_lock, to_lock));

        let res = cargo_run_bin(&gen_manifest_path, &crate_name, self.target_dir);

        fs::remove_dir_all(crate_dir)?;

        match res {
            Ok(exit_status) if exit_status.success() => Ok(out_file),
            Ok(exit_status) => Err(anyhow::anyhow!("Exit status: {}", exit_status)),
            Err(err) => Err(err),
        }
    }
}

/// Get list of packages from the root package and its dependencies
fn get_package_list(
    metadata: &cargo_metadata::Metadata,
    deps_level: usize,
) -> Result<Vec<&Package>, anyhow::Error> {
    let resolve = metadata
        .resolve
        .as_ref()
        .context("failed to get resolve from metadata")?;
    let root_package_id = resolve
        .root
        .as_ref()
        .context("failed to find root package")?;
    let node_map = resolve
        .nodes
        .iter()
        .map(|n| (&n.id, n))
        .collect::<HashMap<_, _>>();
    let package_map = metadata
        .packages
        .iter()
        .map(|p| (&p.id, p))
        .collect::<HashMap<_, _>>();

    let mut deps_set: HashSet<&PackageId> = HashSet::new();
    deps_set.insert(root_package_id);

    let mut deps = vec![root_package_id];
    for _ in 0..deps_level {
        deps = deps
            .iter()
            .filter_map(|id| node_map.get(id))
            .flat_map(|&n| &n.dependencies)
            .filter(|&id| metadata.workspace_members.contains(id))
            .collect();
        if deps.is_empty() {
            break;
        }
        deps_set.extend(deps.iter());
    }
    let package_list: Vec<&Package> = deps_set
        .iter()
        .filter_map(|id| package_map.get(id))
        .copied()
        .collect();
    Ok(package_list)
}

fn skip_reason_for_package(package: &Package) -> Option<String> {
    if let Some(sails) = package.metadata.get("sails")
        && let Some(program_flag) = sails.get("program").and_then(Value::as_bool)
    {
        if !program_flag {
            return Some("package.metadata.sails.program = false".to_owned());
        }
        return None;
    }

    // Ensure the crate exposes some form of library target before probing it.
    let has_library_target = package.targets.iter().any(|target| {
        target.kind.iter().any(|kind| {
            matches!(
                kind,
                TargetKind::Lib
                    | TargetKind::CDyLib
                    | TargetKind::DyLib
                    | TargetKind::RLib
                    | TargetKind::StaticLib
            )
        })
    });
    if !has_library_target {
        return Some("no library target".to_owned());
    }

    // Ignore crates that only depend on sails-rs for dev/test consumers.
    let has_sails_dependency = package
        .dependencies
        .iter()
        .any(|dep| dep.name == "sails-rs" && dep.kind != DependencyKind::Development);

    if has_sails_dependency {
        None
    } else {
        Some("missing non-dev dependency on sails-rs".to_owned())
    }
}

fn doc_file_name(package: &Package) -> String {
    package.name.to_lowercase().replace('-', "_")
}

pub fn get_program_struct_path_from_doc(
    program_package: &Package,
    target_dir: &Utf8Path,
    metadata_fingerprint: &str,
    doc_cache: &mut HashSet<DocCacheKey>,
    persistent_cache_dir: &Utf8Path,
) -> anyhow::Result<(String, MetaPathVersion)> {
    let manifest_path = program_package.manifest_path.clone();
    let docs_path = target_dir
        .join("doc")
        .join(format!("{}.json", doc_file_name(program_package)));

    ensure_program_doc(
        &manifest_path,
        target_dir,
        &docs_path,
        metadata_fingerprint,
        doc_cache,
        persistent_cache_dir,
    )?;

    println!("...reading doc: {docs_path}");
    let json_string = std::fs::read_to_string(docs_path.as_std_path())?;
    let doc_crate: rustdoc_types::Crate = serde_json::from_str(&json_string)?;

    // find `sails_rs::meta::ProgramMeta` path id
    let (program_meta_id, meta_path_version) = doc_crate
        .paths
        .iter()
        .find_map(|(id, summary)| MetaPathVersion::matches(&summary.path).map(|v| (id, v)))
        .context("failed to find `sails_rs::meta::ProgramMeta` definition in dependencies")?;
    // find struct implementing `sails_rs::meta::ProgramMeta`
    let program_struct_path = doc_crate
        .index
        .values()
        .find_map(|idx| try_get_trait_implementation_path(idx, program_meta_id))
        .context("failed to find `sails_rs::meta::ProgramMeta` implemetation")?;
    let program_struct = doc_crate
        .paths
        .get(&program_struct_path.id)
        .context("failed to get Program struct by id")?;
    let program_struct_path = program_struct.path.join("::");
    Ok((program_struct_path, meta_path_version))
}

fn try_get_trait_implementation_path(
    idx: &rustdoc_types::Item,
    program_meta_id: &rustdoc_types::Id,
) -> Option<rustdoc_types::Path> {
    if let rustdoc_types::ItemEnum::Impl(item) = &idx.inner
        && let Some(tp) = &item.trait_
        && &tp.id == program_meta_id
        && let rustdoc_types::Type::ResolvedPath(path) = &item.for_
    {
        return Some(path.clone());
    }
    None
}

fn ensure_program_doc(
    manifest_path: &Utf8Path,
    target_dir: &Utf8Path,
    docs_path: &Utf8Path,
    metadata_fingerprint: &str,
    doc_cache: &mut HashSet<DocCacheKey>,
    persistent_cache_dir: &Utf8Path,
) -> anyhow::Result<()> {
    let mem_key = (manifest_path.to_owned(), metadata_fingerprint.to_owned());
    if doc_cache.contains(&mem_key) && docs_path.as_std_path().exists() {
        return Ok(());
    }

    if docs_path.as_std_path().exists() {
        doc_cache.insert(mem_key);
        return Ok(());
    }

    let cache_key = doc_cache_key(manifest_path, metadata_fingerprint);
    let cache_path = persistent_cache_dir.join(format!("{cache_key}.json"));
    if cache_path.as_std_path().exists() {
        if let Some(parent) = docs_path.parent() {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create `{}`", parent))?;
        }
        // Fast-path: pull the cached rustdoc JSON back into the expected location.
        fs::copy(cache_path.as_std_path(), docs_path.as_std_path())
            .with_context(|| format!("failed to restore cached docs for `{}`", manifest_path))?;
        println!("...reusing cached doc for `{manifest_path}`");
        doc_cache.insert(mem_key);
        return Ok(());
    }

    println!("...running doc generation for `{manifest_path}`");
    cargo_doc(manifest_path, target_dir)?;
    doc_cache.insert(mem_key);

    fs::create_dir_all(persistent_cache_dir.as_std_path())
        .with_context(|| format!("failed to create `{}`", persistent_cache_dir))?;
    if docs_path.as_std_path().exists() {
        // Cache the freshly generated JSON so repeat runs can skip cargo doc.
        fs::copy(docs_path.as_std_path(), cache_path.as_std_path())
            .with_context(|| format!("failed to write doc cache for `{}`", manifest_path))?;
    } else {
        anyhow::bail!(
            "cargo doc for `{}` did not produce `{}`",
            manifest_path,
            docs_path
        );
    }

    Ok(())
}

fn doc_cache_key(manifest_path: &Utf8Path, metadata_fingerprint: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(manifest_path.as_str().as_bytes());
    hasher.update(metadata_fingerprint.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn metadata_fingerprint(metadata: &cargo_metadata::Metadata) -> anyhow::Result<String> {
    let bytes = serde_json::to_vec(metadata)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetaPathVersion {
    V1,
    V2,
}

impl MetaPathVersion {
    const META_PATH_V1: &[&str] = &["sails_rs", "meta", "ProgramMeta"];
    const META_PATH_V2: &[&str] = &["sails_idl_meta", "ProgramMeta"];

    fn matches(path: &Vec<String>) -> Option<Self> {
        if path == Self::META_PATH_V1 {
            Some(MetaPathVersion::V1)
        } else if path == Self::META_PATH_V2 {
            Some(MetaPathVersion::V2)
        } else {
            None
        }
    }
}

fn gen_cargo_toml(
    program_package: &Package,
    sails_package: &Package,
    sails_interface_package: Option<&Package>,
    meta_path_version: MetaPathVersion,
    kind: &ProgramArtifactKind,
) -> String {
    let mut manifest = toml_edit::DocumentMut::new();
    manifest["package"] = toml_edit::Item::Table(toml_edit::Table::new());
    manifest["package"]["name"] = toml_edit::value(get_generator_crate_name(kind, program_package));
    manifest["package"]["version"] = toml_edit::value("0.1.0");
    manifest["package"]["edition"] = toml_edit::value(program_package.edition.as_str());

    let mut dep_table = toml_edit::Table::default();
    let mut package_table = toml_edit::InlineTable::new();
    let manifets_dir = program_package.manifest_path.parent().unwrap();
    package_table.insert("path", manifets_dir.as_str().into());
    dep_table[&program_package.name] = toml_edit::value(package_table);

    let sails_dep = match meta_path_version {
        MetaPathVersion::V1 => sails_dep_v1(sails_package),
        MetaPathVersion::V2 => sails_dep_v2(sails_package, kind),
    };
    dep_table[&sails_package.name] = toml_edit::value(sails_dep);

    if matches!(kind, ProgramArtifactKind::Canonical) {
        if let Some(package) = sails_interface_package {
            let mut iface_dep = toml_edit::InlineTable::new();
            let manifets_dir = package.manifest_path.parent().unwrap();
            iface_dep.insert("package", package.name.as_str().into());
            iface_dep.insert("path", manifets_dir.as_str().into());
            dep_table["sails-interface-id"] = toml_edit::value(iface_dep);
        } else {
            dep_table["sails-interface-id"] = toml_edit::value(sails_interface_dep_from_crates(
                sails_package.version.to_string(),
            ));
        }
    }

    manifest["dependencies"] = toml_edit::Item::Table(dep_table);

    let mut bin = toml_edit::Table::new();
    bin["name"] = toml_edit::value(get_generator_crate_name(kind, program_package));
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

fn sails_dep_v2(sails_package: &Package, _kind: &ProgramArtifactKind) -> toml_edit::InlineTable {
    let mut features = toml_edit::Array::default();
    features.push("idl-gen");
    features.push("std");
    let mut sails_table = toml_edit::InlineTable::new();
    let manifets_dir = sails_package.manifest_path.parent().unwrap();
    sails_table.insert("package", sails_package.name.as_str().into());
    sails_table.insert("path", manifets_dir.as_str().into());
    sails_table.insert("features", features.into());
    sails_table
}

fn sails_interface_dep_from_crates(version: String) -> toml_edit::InlineTable {
    let mut table = toml_edit::InlineTable::new();
    table.insert("version", version.into());
    table
}

fn gen_main_rs(
    kind: &ProgramArtifactKind,
    program_struct_path: &str,
    out_file: &cargo_metadata::camino::Utf8Path,
) -> String {
    match kind {
        ProgramArtifactKind::Idl => format!(
            "
#![allow(unexpected_cfgs)]

fn main() {{
    sails_rs::generate_idl_to_file::<{}>(
        std::path::PathBuf::from(r\"{}\")
    )
    .unwrap();
}}",
            program_struct_path,
            out_file.as_str(),
        ),
        ProgramArtifactKind::Canonical => format!(
            "
#![allow(unexpected_cfgs)]

fn main() {{
    use sails_rs::meta::ProgramMeta;
    use std::collections::BTreeMap;
    use sails_interface_id::canonical::{{CanonicalDocument, CanonicalHashMeta, CANONICAL_SCHEMA, CANONICAL_VERSION, CANONICAL_HASH_ALGO}};
    use sails_interface_id::INTERFACE_HASH_DOMAIN_STR;
    use sails_interface_id::runtime::build_canonical_document_from_meta;

    let mut services = BTreeMap::new();
    let mut types = BTreeMap::new();
    for (_, service_fn) in <{} as ProgramMeta>::SERVICES {{
        let meta = service_fn();
        let doc = build_canonical_document_from_meta(&meta)
            .expect(\"failed to build canonical document\");
        services.extend(doc.services);
        types.extend(doc.types);
    }}

    let document = CanonicalDocument {{
        canon_schema: CANONICAL_SCHEMA.to_owned(),
        canon_version: CANONICAL_VERSION.to_owned(),
        hash: CanonicalHashMeta {{
            algo: CANONICAL_HASH_ALGO.to_owned(),
            domain: INTERFACE_HASH_DOMAIN_STR.to_owned(),
        }},
        services,
        types,
    }};

    let bytes = document
        .to_bytes()
        .expect(\"failed to serialize canonical document\");
    std::fs::write(std::path::PathBuf::from(r\"{}\"), bytes)
        .expect(\"failed to write canonical document\");
}}",
            program_struct_path,
            out_file.as_str(),
        ),
    }
}

fn get_generator_crate_name(kind: &ProgramArtifactKind, program_package: &Package) -> String {
    match kind {
        ProgramArtifactKind::Idl => format!("{}-idl-gen", program_package.name),
        ProgramArtifactKind::Canonical => format!("{}-canonical-gen", program_package.name),
    }
}

fn get_artifact_output_path(
    kind: &ProgramArtifactKind,
    target_dir: &Utf8Path,
    program_package: &Package,
) -> Utf8PathBuf {
    match kind {
        ProgramArtifactKind::Idl => target_dir.join(format!("{}.idl", &program_package.name)),
        ProgramArtifactKind::Canonical => {
            target_dir.join(format!("{}.canonical.json", &program_package.name))
        }
    }
}
