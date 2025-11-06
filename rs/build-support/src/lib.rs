use anyhow::{Context, Result};
use cargo_metadata::{
    DependencyKind, Metadata, Package, PackageId, TargetKind, camino::Utf8PathBuf,
};
use sails_interface_id::canonical::CanonicalDocument;
use std::sync::OnceLock;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{Arc, Mutex},
};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table};

pub use metadata::metadata_fingerprint;

type MetadataCache = HashMap<(Utf8PathBuf, Option<String>), (Arc<Metadata>, String)>;

static METADATA_CACHE: OnceLock<Mutex<MetadataCache>> = OnceLock::new();

fn load_metadata(
    manifest_path: &Utf8PathBuf,
    out_dir: Option<&str>,
) -> Result<(Arc<Metadata>, String)> {
    let cache = METADATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_key = (manifest_path.clone(), out_dir.map(|s| s.to_string()));
    {
        let cache_guard = cache.lock().expect("metadata cache mutex poisoned");
        if let Some((metadata, fingerprint)) = cache_guard.get(&cache_key) {
            return Ok((Arc::clone(metadata), fingerprint.clone()));
        }
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .context("failed to read cargo metadata")?;
    let fingerprint = metadata::metadata_fingerprint(&metadata)?;
    let metadata_arc = Arc::new(metadata);

    let mut cache_guard = cache.lock().expect("metadata cache mutex poisoned");
    cache_guard.insert(cache_key, (Arc::clone(&metadata_arc), fingerprint.clone()));

    Ok((metadata_arc, fingerprint))
}

mod metadata {
    use super::*;

    pub fn metadata_fingerprint(metadata: &Metadata) -> Result<String> {
        let bytes =
            serde_json::to_vec(metadata).context("failed to serialize cargo metadata to JSON")?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgramArtifactKind {
    Idl,
    Canonical,
}

pub fn generate_program_artifact(
    manifest_path: &Path,
    target_dir: Option<&Path>,
    kind: ProgramArtifactKind,
) -> Result<Utf8PathBuf> {
    let manifest_abs = manifest_path.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize manifest path {}",
            manifest_path.display()
        )
    })?;
    let manifest_utf8 = Utf8PathBuf::from_path_buf(manifest_abs)
        .map_err(|_| anyhow::anyhow!("manifest path is not valid UTF-8"))?;

    eprintln!("build-support: reading metadata: {manifest_utf8}");
    let (metadata, _fingerprint) = load_metadata(&manifest_utf8, None)?;

    generate_program_artifact_with_metadata(&manifest_utf8, metadata, target_dir, kind)
}

fn generate_program_artifact_with_metadata(
    manifest_path: &Utf8PathBuf,
    metadata: Arc<Metadata>,
    target_dir: Option<&Path>,
    kind: ProgramArtifactKind,
) -> Result<Utf8PathBuf> {
    let metadata_ref = metadata.as_ref();
    let target_dir = target_dir
        .map(|path| {
            Utf8PathBuf::from_path_buf(path.to_path_buf())
                .unwrap_or_else(|_| metadata_ref.target_directory.clone())
        })
        .unwrap_or_else(|| metadata_ref.target_directory.clone());
    let sails_rs_packages = metadata_ref
        .packages
        .iter()
        .filter(|p| p.name == "sails-rs")
        .collect::<Vec<_>>();
    let sails_interface_id_packages = metadata_ref
        .packages
        .iter()
        .filter(|p| p.name == "sails-interface-id")
        .collect::<Vec<_>>();

    let manifest_package = metadata_ref
        .packages
        .iter()
        .find(|package| &package.manifest_path == manifest_path)
        .with_context(|| {
            format!(
                "failed to locate package for manifest path {}",
                manifest_path
            )
        })?;

    const DEP_SEARCH_DEPTH: usize = 2;
    let mut candidate_packages =
        collect_candidate_packages(metadata_ref, manifest_package, DEP_SEARCH_DEPTH)?;

    // Ensure the manifest package is the first candidate.
    if let Some(pos) = candidate_packages
        .iter()
        .position(|pkg| pkg.id == manifest_package.id)
    {
        candidate_packages.swap(0, pos);
    } else {
        candidate_packages.insert(0, manifest_package);
    }

    let mut errors: Vec<anyhow::Error> = Vec::new();

    for program_package in candidate_packages {
        if let Some(reason) = skip_reason_for_package(program_package) {
            errors.push(anyhow::anyhow!(
                "package `{}` is not a valid Sails program: {reason}",
                program_package.name
            ));
            continue;
        }

        if kind == ProgramArtifactKind::Canonical {
            let canonical_path = get_artifact_output_path(kind, &target_dir, program_package);
            if canonical_path.as_std_path().exists() {
                match fs::read_to_string(canonical_path.as_std_path())
                    .with_context(|| format!("failed to read canonical at {canonical_path}"))
                    .and_then(|content| {
                        CanonicalDocument::from_json_str(&content)
                            .context("existing canonical document is invalid")
                    }) {
                    Ok(_) => {
                        eprintln!(
                            "build-support: using existing canonical document: {canonical_path}"
                        );
                        return Ok(canonical_path);
                    }
                    Err(err) => {
                        errors.push(err);
                        continue;
                    }
                }
            }
        }

        let generator = PackageGenerator::new(
            program_package,
            &sails_rs_packages,
            &sails_interface_id_packages,
            &target_dir,
            &metadata_ref.workspace_root,
        );

        match generator.try_generate_for_package(kind) {
            Ok(path) => return Ok(path),
            Err(err) => errors.push(err),
        }
    }

    if let Some(err) = errors.pop() {
        Err(err.context(format!(
            "no Sails program implementation found starting from manifest `{manifest_path}`"
        )))
    } else {
        Err(anyhow::anyhow!(
            "no Sails program implementation found starting from manifest `{manifest_path}`"
        ))
    }
}

pub fn ensure_canonical_artifact(
    manifest_path: &Path,
    target_dir: Option<&Path>,
    output_path: &Path,
) -> Result<bool> {
    let manifest_abs = manifest_path.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize manifest path {}",
            manifest_path.display()
        )
    })?;
    let manifest_utf8 = Utf8PathBuf::from_path_buf(manifest_abs)
        .map_err(|_| anyhow::anyhow!("manifest path is not valid UTF-8"))?;
    let (metadata, fingerprint) = load_metadata(&manifest_utf8, None)?;

    let fingerprint_path = output_path.with_extension("fingerprint");
    if output_path.exists()
        && fingerprint_path.exists()
        && fs::read_to_string(&fingerprint_path).ok().as_deref() == Some(&fingerprint)
    {
        return Ok(false);
    }

    let canonical_source = generate_program_artifact_with_metadata(
        &manifest_utf8,
        metadata,
        target_dir,
        ProgramArtifactKind::Canonical,
    )?;

    let canonical_bytes = fs::read(&canonical_source)
        .with_context(|| format!("failed to read {canonical_source}"))?;
    CanonicalDocument::from_json_str(std::str::from_utf8(&canonical_bytes).unwrap())
        .context("generated canonical document is invalid")?;
    if let Some(dir) = output_path.parent() {
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "failed to create directory {} for canonical output",
                dir.display()
            )
        })?;
    }
    fs::write(output_path, &canonical_bytes).with_context(|| {
        format!(
            "failed to write canonical document to {}",
            output_path.display()
        )
    })?;
    fs::write(&fingerprint_path, fingerprint)
        .with_context(|| format!("failed to write fingerprint {}", fingerprint_path.display()))?;
    Ok(true)
}

pub fn ensure_canonical_artifact_from_str(json_str: &str, output_path: &Path) -> Result<()> {
    CanonicalDocument::from_json_str(json_str).context("provided canonical document is invalid")?;
    if let Some(dir) = output_path.parent() {
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "failed to create directory {} for canonical output",
                dir.display()
            )
        })?;
    }
    fs::write(output_path, json_str).with_context(|| {
        format!(
            "failed to write canonical document to {}",
            output_path.display()
        )
    })?;
    Ok(())
}

pub fn ensure_canonical_env() -> Result<Option<PathBuf>> {
    if env::var_os("SAILS_CANONICAL_BUILD").is_some()
        || env::var_os("__GEAR_WASM_BUILDER_NO_BUILD").is_some()
    {
        return Ok(None);
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR environment variable missing")?;
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable missing")?;
    let pkg_name =
        env::var("CARGO_PKG_NAME").context("CARGO_PKG_NAME environment variable missing")?;

    let manifest_path = PathBuf::from(manifest_dir).join("Cargo.toml");
    let out_root = PathBuf::from(out_dir.clone());
    let canonical_path = out_root
        .join("canonical")
        .join(format!("{pkg_name}.canonical.json"));
    let canonical_target_dir = out_root.join("canonical-target");

    // Check if canonical already exists and is valid
    if canonical_path.exists() {
        if let Ok(content) = fs::read_to_string(&canonical_path) {
            if CanonicalDocument::from_json_str(&content).is_ok() {
                println!(
                    "...using existing canonical document: {}",
                    canonical_path.display()
                );
                println!(
                    "cargo:rustc-env=SAILS_INTERFACE_CANONICAL={}",
                    canonical_path.display()
                );
                return Ok(Some(canonical_path));
            }
        }
    }

    let manifest_utf8 = Utf8PathBuf::from_path_buf(manifest_path.to_path_buf())
        .map_err(|_| anyhow::anyhow!("manifest path is not valid UTF-8"))?;
    let (metadata, fingerprint) = load_metadata(&manifest_utf8, Some(&out_dir))?;

    let canonical_source = generate_program_artifact_with_metadata(
        &manifest_utf8,
        metadata,
        Some(&canonical_target_dir),
        ProgramArtifactKind::Canonical,
    )?;

    let canonical_bytes = fs::read(&canonical_source)
        .with_context(|| format!("failed to read {canonical_source}"))?;
    CanonicalDocument::from_json_str(std::str::from_utf8(&canonical_bytes).unwrap())
        .context("generated canonical document is invalid")?;
    if let Some(dir) = canonical_path.parent() {
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "failed to create directory {} for canonical output",
                dir.display()
            )
        })?;
    }
    fs::write(&canonical_path, &canonical_bytes).with_context(|| {
        format!(
            "failed to write canonical document to {}",
            canonical_path.display()
        )
    })?;
    fs::write(canonical_path.with_extension("fingerprint"), fingerprint).with_context(|| {
        format!(
            "failed to write fingerprint {}",
            canonical_path.with_extension("fingerprint").display()
        )
    })?;

    println!(
        "cargo:rustc-env=SAILS_INTERFACE_CANONICAL={}",
        canonical_path.display()
    );

    Ok(Some(canonical_path))
}

fn skip_reason_for_package(package: &Package) -> Option<String> {
    if let Some(sails) = package.metadata.get("sails")
        && let Some(program_flag) = sails.get("program").and_then(|v| v.as_bool())
    {
        if !program_flag {
            return Some("package.metadata.sails.program = false".to_owned());
        }
        return None;
    }

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

// rustdoc-based discovery removed

fn collect_candidate_packages<'a>(
    metadata: &'a Metadata,
    manifest_package: &'a Package,
    max_depth: usize,
) -> Result<Vec<&'a Package>> {
    if max_depth == 0 {
        return Ok(vec![manifest_package]);
    }

    let resolve = metadata
        .resolve
        .as_ref()
        .context("failed to get dependency graph from cargo metadata")?;
    let node_map: HashMap<&PackageId, _> =
        resolve.nodes.iter().map(|node| (&node.id, node)).collect();
    let package_map: HashMap<&PackageId, &Package> = metadata
        .packages
        .iter()
        .map(|package| (&package.id, package))
        .collect();
    let workspace_members: HashSet<&PackageId> = metadata.workspace_members.iter().collect();

    let mut visited: HashSet<PackageId> = HashSet::new();
    let mut queue: VecDeque<(PackageId, usize)> =
        VecDeque::from([(manifest_package.id.clone(), 0)]);
    let mut candidates = Vec::new();

    while let Some((package_id, depth)) = queue.pop_front() {
        if !visited.insert(package_id.clone()) {
            continue;
        }

        if let Some(package) = package_map.get(&package_id) {
            candidates.push(*package);
        }

        if depth >= max_depth {
            continue;
        }

        if let Some(node) = node_map.get(&package_id) {
            for dependency_id in &node.dependencies {
                if workspace_members.contains(dependency_id) {
                    queue.push_back((dependency_id.clone(), depth + 1));
                }
            }
        }
    }

    Ok(candidates)
}

struct PackageGenerator<'a> {
    program_package: &'a Package,
    sails_rs_packages: &'a Vec<&'a Package>,
    sails_interface_id_packages: &'a Vec<&'a Package>,
    target_dir: &'a Utf8PathBuf,
    workspace_root: &'a Utf8PathBuf,
}

impl<'a> PackageGenerator<'a> {
    fn new(
        program_package: &'a Package,
        sails_rs_packages: &'a Vec<&'a Package>,
        sails_interface_id_packages: &'a Vec<&'a Package>,
        target_dir: &'a Utf8PathBuf,
        workspace_root: &'a Utf8PathBuf,
    ) -> Self {
        Self {
            program_package,
            sails_rs_packages,
            sails_interface_id_packages,
            target_dir,
            workspace_root,
        }
    }

    fn try_generate_for_package(&self, kind: ProgramArtifactKind) -> Result<Utf8PathBuf> {
        let sails_dep = self
            .program_package
            .dependencies
            .iter()
            .find(|p| p.name == "sails-rs")
            .context("failed to find `sails-rs` dependency")?;
        let sails_package = self
            .sails_rs_packages
            .iter()
            .find(|p| sails_dep.req.matches(&p.version))
            .context("failed to find matching `sails-rs` package version")?;
        let sails_interface_package = self
            .sails_interface_id_packages
            .iter()
            .find(|p| sails_dep.req.matches(&p.version))
            .copied();

        let crate_name = get_generator_crate_name(kind, self.program_package);
        let crate_dir = self.target_dir.join(&crate_name);
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let host_features = host_build_features(self.program_package);

        let gen_manifest_path = crate_dir.join("Cargo.toml");
        write_file(&gen_manifest_path, {
            let contents = gen_cargo_toml(
                self.program_package,
                sails_package,
                sails_interface_package,
                kind,
                self.workspace_root,
                &host_features,
            );
            eprintln!("build-support: generator Cargo.toml:\n{contents}");
            contents
        })?;

        let out_file = get_artifact_output_path(kind, self.target_dir, self.program_package);
        let main_rs_path = src_dir.join("main.rs");
        write_file(
            &main_rs_path,
            gen_main_rs(kind, self.program_package, &out_file),
        )?;

        let from_lock = self.workspace_root.join("Cargo.lock");
        let to_lock = crate_dir.join("Cargo.lock");
        let _ = fs::copy(from_lock.as_std_path(), to_lock.as_std_path());

        eprintln!("build-support: running generator crate at {}", crate_dir);
        let res = cargo_run_bin(&gen_manifest_path, &crate_name, self.target_dir);
        if res.as_ref().is_ok_and(|status| status.success()) {
            let _ = fs::remove_dir_all(crate_dir.as_std_path());
        }

        match res {
            Ok(exit_status) if exit_status.success() => Ok(out_file),
            Ok(exit_status) => Err(anyhow::anyhow!("Exit status: {}", exit_status)),
            Err(err) => Err(err),
        }
    }
}

fn get_generator_crate_name(kind: ProgramArtifactKind, program_package: &Package) -> String {
    let suffix = match kind {
        ProgramArtifactKind::Idl => "idl",
        ProgramArtifactKind::Canonical => "canonical",
    };
    format!(
        "sails-gen-{}-{}",
        program_package.name.replace('-', "_"),
        suffix
    )
}

fn host_build_features(package: &Package) -> Vec<String> {
    let mut features = package
        .metadata
        .get("sails")
        .and_then(|value| {
            value
                .get("host_features")
                .or_else(|| value.get("host-features"))
        })
        .and_then(|value| value.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|value| value.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if features.is_empty() && package.features.contains_key("std") {
        features.push("std".to_string());
    }

    if features.is_empty() && package.features.contains_key("mockall") {
        features.push("mockall".to_string());
    }

    features
}

fn gen_cargo_toml(
    program_package: &Package,
    sails_package: &Package,
    sails_interface_package: Option<&Package>,
    kind: ProgramArtifactKind,
    workspace_root: &Utf8PathBuf,
    host_features: &[String],
) -> String {
    let mut manifest = DocumentMut::new();
    manifest["package"] = Item::Table(Table::new());
    manifest["package"]["name"] = toml_edit::value(get_generator_crate_name(kind, program_package));
    manifest["package"]["version"] = toml_edit::value("0.1.0");
    manifest["package"]["edition"] = toml_edit::value(program_package.edition.as_str());

    let mut dep_table = Table::default();
    let mut package_table = InlineTable::new();
    let manifest_dir = program_package.manifest_path.parent().unwrap();
    package_table.insert("path", manifest_dir.as_str().into());
    if !host_features.is_empty() {
        let mut feats = Array::default();
        for feature in host_features {
            feats.push(feature.as_str());
        }
        package_table.insert("features", feats.into());
    }
    dep_table[&program_package.name] = toml_edit::value(package_table);

    let sails_dep = sails_dep_v2(sails_package, kind);
    dep_table[&sails_package.name] = toml_edit::value(sails_dep);

    // TODO(sails-release): drop the crates.io patch once the updated crates are published.
    let patch_table = manifest
        .entry("patch")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .expect("patch is a table")
        .entry("crates-io")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .expect("patch.crates-io is a table");

    let mut idl_meta_patch = InlineTable::new();
    idl_meta_patch.insert(
        "path",
        workspace_root.join("rs").join("idl-meta").as_str().into(),
    );
    patch_table.insert("sails-idl-meta", toml_edit::value(idl_meta_patch));

    let mut idl_gen_patch = InlineTable::new();
    idl_gen_patch.insert(
        "path",
        workspace_root.join("rs").join("idl-gen").as_str().into(),
    );
    patch_table.insert("sails-idl-gen", toml_edit::value(idl_gen_patch));

    let mut sails_rs_patch = InlineTable::new();
    sails_rs_patch.insert("path", workspace_root.join("rs").as_str().into());
    patch_table.insert("sails-rs", toml_edit::value(sails_rs_patch));

    let mut registry_patch = InlineTable::new();
    registry_patch.insert(
        "path",
        workspace_root
            .join("rs")
            .join("program-registry")
            .as_str()
            .into(),
    );
    patch_table.insert("sails-program-registry", toml_edit::value(registry_patch));

    if matches!(kind, ProgramArtifactKind::Canonical) {
        let mut idl_meta_dep = InlineTable::new();
        idl_meta_dep.insert(
            "path",
            workspace_root.join("rs").join("idl-meta").as_str().into(),
        );
        dep_table["sails-idl-meta"] = toml_edit::value(idl_meta_dep);

        let iface_dep = if let Some(package) = sails_interface_package {
            let mut iface_dep = InlineTable::new();
            let manifest_dir = package.manifest_path.parent().unwrap();
            iface_dep.insert("package", package.name.as_str().into());
            iface_dep.insert("path", manifest_dir.as_str().into());
            iface_dep
        } else {
            let mut table = InlineTable::new();
            let iface_path = workspace_root.join("rs").join("interface-id");
            table.insert("path", iface_path.as_str().into());
            table
        };
        dep_table["sails-interface-id"] = toml_edit::value(iface_dep);

        let mut iface_patch = InlineTable::new();
        iface_patch.insert(
            "path",
            workspace_root
                .join("rs")
                .join("interface-id")
                .as_str()
                .into(),
        );
        patch_table.insert("sails-interface-id", toml_edit::value(iface_patch));
    }

    manifest["dependencies"] = Item::Table(dep_table);

    let mut bin = Table::new();
    bin["name"] = toml_edit::value(get_generator_crate_name(kind, program_package));
    bin["path"] = toml_edit::value("src/main.rs");
    manifest["bin"]
        .or_insert(Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .expect("bin is an array of tables")
        .push(bin);

    manifest["workspace"] = Item::Table(Table::new());

    manifest.to_string()
}

fn sails_dep_v2(sails_package: &Package, kind: ProgramArtifactKind) -> InlineTable {
    let mut features = Array::default();
    match kind {
        ProgramArtifactKind::Idl => {
            features.push("idl-gen");
            features.push("std");
        }
        ProgramArtifactKind::Canonical => {
            features.push("std");
        }
    }
    let mut sails_table = InlineTable::new();
    let manifest_dir = sails_package.manifest_path.parent().unwrap();
    sails_table.insert("package", sails_package.name.as_str().into());
    sails_table.insert("path", manifest_dir.as_str().into());
    if !features.is_empty() {
        sails_table.insert("features", features.into());
    }
    sails_table
}

fn gen_main_rs(
    kind: ProgramArtifactKind,
    program_package: &Package,
    out_path: &Utf8PathBuf,
) -> String {
    let crate_ident = program_package.name.replace('-', "_");
    let package_name = &program_package.name;
    let artifact_call = match kind {
        ProgramArtifactKind::Idl => r#"            entry
                .write_idl(&out_path)
                .expect("failed to generate IDL");"#
            .to_string(),
        ProgramArtifactKind::Canonical => r#"            entry
                .write_canonical(&out_path)
                .expect("failed to generate canonical document");"#
            .to_string(),
    };

    format!(
        r#"
#![allow(unexpected_cfgs)]

#[allow(unused_extern_crates)]
extern crate {crate_ident} as _program_crate;

fn main() {{
    let out_path = std::path::PathBuf::from(r"{out_path}");
    let entry = match sails_rs::program_registry::lookup_by_package("{package_name}") {{
        Ok(entry) => entry,
        Err(err) => {{
            eprintln!(
                "sails-build-support: package `{package_name}` is not registered: {{err}}"
            );
            std::process::exit(1);
        }}
    }};
    match entry.meta_path_version {{
        sails_rs::program_registry::MetaPathVersion::V2 => {{
{artifact_call}
        }}
        other => panic!(
            "unsupported meta path version {{:?}} for `{package_name}`; expected sails-rs::meta::ProgramMeta",
            other
        ),
    }}
}}
"#,
        crate_ident = crate_ident,
        out_path = out_path.as_str(),
        package_name = package_name,
        artifact_call = artifact_call
    )
}

fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, contents.as_ref())
        .with_context(|| format!("failed to write `{}`", path.display()))
}

fn cargo_run_bin(
    manifest_path: &Utf8PathBuf,
    bin_name: &str,
    target_dir: &Utf8PathBuf,
) -> Result<ExitStatus> {
    let cargo_path = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let runner_target_dir = target_dir
        .join("sails")
        .join("generator-run")
        .join(bin_name);

    let mut cmd = Command::new(cargo_path);
    cmd.env("CARGO_TARGET_DIR", runner_target_dir.as_str())
        .env("__GEAR_WASM_BUILDER_NO_BUILD", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .arg("run")
        .arg("--target-dir")
        .arg(runner_target_dir.as_str())
        .arg("--manifest-path")
        .arg(manifest_path.as_str())
        .arg("--bin")
        .arg(bin_name);

    if let Ok(host_target) = std::env::var("HOST") {
        cmd.env("CARGO_BUILD_TARGET", &host_target);
        cmd.arg("--target").arg(host_target);
    }

    let mut child = cmd
        .spawn()
        .context("failed to spawn `cargo run` command for generator crate")?;

    let mut stderr = String::new();
    if let Some(ref mut stream) = child.stderr {
        stream
            .read_to_string(&mut stderr)
            .context("failed to read generator stderr")?;
    }

    let status = child.wait().context("failed to wait for generator")?;
    if !stderr.trim().is_empty() {
        if !stderr.trim().is_empty() {
            eprintln!("build-support: generator stderr:\n{stderr}");
        }
    }
    Ok(status)
}

fn get_artifact_output_path(
    kind: ProgramArtifactKind,
    target_dir: &Utf8PathBuf,
    program_package: &Package,
) -> Utf8PathBuf {
    let suffix = match kind {
        ProgramArtifactKind::Idl => ".idl",
        ProgramArtifactKind::Canonical => ".canonical.json",
    };
    target_dir
        .join("sails")
        .join(&program_package.name)
        .with_extension(suffix.trim_start_matches('.'))
}
