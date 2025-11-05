use anyhow::{Context, Result};
use cargo_metadata::{
    DependencyKind, Metadata, Package, PackageId, TargetKind, camino::Utf8PathBuf,
};
use sails_interface_id::canonical::CanonicalDocument;
use std::sync::OnceLock;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{Arc, Mutex},
};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table};

pub use metadata::{DocCache, metadata_fingerprint};

type MetadataCache = HashMap<Utf8PathBuf, (Arc<Metadata>, String)>;

static METADATA_CACHE: OnceLock<Mutex<MetadataCache>> = OnceLock::new();

fn load_metadata(manifest_path: &Utf8PathBuf) -> Result<(Arc<Metadata>, String)> {
    let cache = METADATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    {
        let cache_guard = cache.lock().expect("metadata cache mutex poisoned");
        if let Some((metadata, fingerprint)) = cache_guard.get(manifest_path) {
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
    cache_guard.insert(
        manifest_path.clone(),
        (Arc::clone(&metadata_arc), fingerprint.clone()),
    );

    Ok((metadata_arc, fingerprint))
}

mod metadata {
    use super::*;
    use blake3::Hasher;
    use cargo_metadata::camino::Utf8Path;

    static DOC_CACHE: OnceLock<Mutex<DocCache>> = OnceLock::new();

    pub struct DocCache {
        memory: HashSet<(Utf8PathBuf, String)>,
    }

    impl Default for DocCache {
        fn default() -> Self {
            Self::new()
        }
    }

    impl DocCache {
        pub fn new() -> Self {
            Self {
                memory: HashSet::new(),
            }
        }

        pub fn contains(&self, manifest: &Utf8Path, fingerprint: &str) -> bool {
            self.memory
                .contains(&(manifest.to_owned(), fingerprint.to_owned()))
        }

        pub fn insert(&mut self, manifest: &Utf8Path, fingerprint: &str) {
            self.memory
                .insert((manifest.to_owned(), fingerprint.to_owned()));
        }
    }

    pub fn metadata_fingerprint(metadata: &Metadata) -> Result<String> {
        let bytes =
            serde_json::to_vec(metadata).context("failed to serialize cargo metadata to JSON")?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    pub fn doc_cache_key(manifest_path: &Utf8Path, metadata_fingerprint: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(manifest_path.as_str().as_bytes());
        hasher.update(metadata_fingerprint.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    pub fn doc_cache() -> &'static Mutex<DocCache> {
        DOC_CACHE.get_or_init(|| Mutex::new(DocCache::new()))
    }
}

#[derive(Clone, Copy)]
pub enum ProgramArtifactKind {
    Idl,
    Canonical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaPathVersion {
    V1,
    V2,
}

impl MetaPathVersion {
    fn matches(path: &[String]) -> Option<Self> {
        if path.len() == 2 && path[0] == "sails_idl_meta" && path[1] == "ProgramMeta" {
            Some(Self::V1)
        } else if path.len() == 3
            && path[0] == "sails_rs"
            && path[1] == "meta"
            && path[2] == "ProgramMeta"
        {
            Some(Self::V2)
        } else {
            None
        }
    }
}

pub fn generate_program_artifact(
    manifest_path: &Path,
    target_dir: Option<&Path>,
    deps_level: usize,
    kind: ProgramArtifactKind,
) -> Result<Utf8PathBuf> {
    let manifest_utf8 = Utf8PathBuf::from_path_buf(manifest_path.to_path_buf())
        .map_err(|_| anyhow::anyhow!("manifest path is not valid UTF-8"))?;

    println!("...reading metadata: {manifest_utf8}");
    let (metadata, fingerprint) = load_metadata(&manifest_utf8)?;

    generate_program_artifact_with_metadata(
        &manifest_utf8,
        metadata,
        fingerprint,
        target_dir,
        deps_level,
        kind,
    )
}

fn generate_program_artifact_with_metadata(
    _manifest_path: &Utf8PathBuf,
    metadata: Arc<Metadata>,
    metadata_fingerprint: String,
    target_dir: Option<&Path>,
    deps_level: usize,
    kind: ProgramArtifactKind,
) -> Result<Utf8PathBuf> {
    let metadata_ref = metadata.as_ref();
    let target_dir = target_dir
        .map(|path| {
            Utf8PathBuf::from_path_buf(path.to_path_buf())
                .unwrap_or_else(|_| metadata_ref.target_directory.clone())
        })
        .unwrap_or_else(|| metadata_ref.target_directory.clone());
    let doc_cache_dir = target_dir.join("sails").join("doc-cache");

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

    let package_list = get_package_list(metadata_ref, deps_level)?;
    let doc_cache_mutex = metadata::doc_cache();
    let mut doc_cache = doc_cache_mutex.lock().expect("doc cache mutex poisoned");

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
            &target_dir,
            &metadata_ref.workspace_root,
        );
        match get_program_struct_path_from_doc(
            program_package,
            &target_dir,
            &metadata_fingerprint,
            &mut doc_cache,
            &doc_cache_dir,
        ) {
            Ok((program_struct_path, meta_path_version)) => {
                println!("...found Program implementation: {program_struct_path}");
                match generator.try_generate_for_package(
                    &program_struct_path,
                    meta_path_version,
                    kind,
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

pub fn ensure_canonical_artifact(
    manifest_path: &Path,
    target_dir: Option<&Path>,
    deps_level: usize,
    output_path: &Path,
) -> Result<bool> {
    let manifest_utf8 = Utf8PathBuf::from_path_buf(manifest_path.to_path_buf())
        .map_err(|_| anyhow::anyhow!("manifest path is not valid UTF-8"))?;
    let (metadata, fingerprint) = load_metadata(&manifest_utf8)?;

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
        fingerprint.clone(),
        target_dir,
        deps_level,
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

pub fn ensure_canonical_env(deps_level: usize) -> Result<Option<PathBuf>> {
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
    let out_root = PathBuf::from(out_dir);
    let canonical_path = out_root
        .join("canonical")
        .join(format!("{pkg_name}.canonical.json"));
    let canonical_target_dir = out_root.join("canonical-target");

    ensure_canonical_artifact(
        &manifest_path,
        Some(&canonical_target_dir),
        deps_level,
        &canonical_path,
    )?;

    println!(
        "cargo:rustc-env=SAILS_INTERFACE_CANONICAL={}",
        canonical_path.display()
    );

    Ok(Some(canonical_path))
}

fn get_package_list(metadata: &Metadata, deps_level: usize) -> Result<Vec<&Package>> {
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

fn doc_file_name(package: &Package) -> String {
    package.name.to_lowercase().replace('-', "_")
}

fn get_program_struct_path_from_doc(
    program_package: &Package,
    target_dir: &Utf8PathBuf,
    metadata_fingerprint: &str,
    doc_cache: &mut metadata::DocCache,
    persistent_cache_dir: &Utf8PathBuf,
) -> Result<(String, MetaPathVersion)> {
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
    let json_string = fs::read_to_string(docs_path.as_std_path())?;
    let doc_crate: rustdoc_types::Crate = serde_json::from_str(&json_string)?;

    let (program_meta_id, meta_path_version) = doc_crate
        .paths
        .iter()
        .find_map(|(id, summary)| MetaPathVersion::matches(&summary.path).map(|v| (id, v)))
        .context("failed to find ProgramMeta definition")?;

    let program_struct_path = doc_crate
        .index
        .values()
        .find_map(|idx| try_get_trait_implementation_path(idx, program_meta_id))
        .context("failed to find ProgramMeta implementation")?;
    let program_struct = doc_crate
        .paths
        .get(&program_struct_path.id)
        .context("failed to get Program struct by id")?;
    Ok((program_struct.path.join("::"), meta_path_version))
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
    manifest_path: &Utf8PathBuf,
    target_dir: &Utf8PathBuf,
    docs_path: &Utf8PathBuf,
    metadata_fingerprint: &str,
    doc_cache: &mut metadata::DocCache,
    persistent_cache_dir: &Utf8PathBuf,
) -> Result<()> {
    let cache_path =
        persistent_cache_dir.join(metadata::doc_cache_key(manifest_path, metadata_fingerprint));
    if doc_cache.contains(manifest_path, metadata_fingerprint) && docs_path.as_std_path().exists() {
        return Ok(());
    }

    if cache_path.as_std_path().exists() {
        println!("...using cached doc: {cache_path}");
        fs::copy(cache_path.as_std_path(), docs_path.as_std_path()).with_context(|| {
            format!(
                "failed to copy cached doc {}",
                cache_path.as_std_path().display()
            )
        })?;
        doc_cache.insert(manifest_path, metadata_fingerprint);
        return Ok(());
    }

    println!("...running doc generation for `{manifest_path}`");
    cargo_doc(manifest_path, target_dir)?;
    doc_cache.insert(manifest_path, metadata_fingerprint);

    fs::create_dir_all(persistent_cache_dir.as_std_path())
        .with_context(|| format!("failed to create `{persistent_cache_dir}`"))?;
    if docs_path.as_std_path().exists() {
        fs::copy(docs_path.as_std_path(), cache_path.as_std_path())
            .with_context(|| format!("failed to write doc cache for `{manifest_path}`"))?;
    } else {
        anyhow::bail!(
            "cargo doc for `{}` did not produce `{}`",
            manifest_path,
            docs_path
        );
    }

    Ok(())
}

fn cargo_doc(manifest_path: &Utf8PathBuf, target_dir: &Utf8PathBuf) -> Result<ExitStatus> {
    let cargo_path = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let mut cmd = Command::new(cargo_path);
    cmd.env("RUSTC_BOOTSTRAP", "1")
        .env(
            "RUSTDOCFLAGS",
            "-Z unstable-options --output-format=json --cap-lints=allow",
        )
        .env("__GEAR_WASM_BUILDER_NO_BUILD", "1")
        .stdout(std::process::Stdio::null())
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

    fn try_generate_for_package(
        &self,
        program_struct_path: &str,
        meta_path_version: MetaPathVersion,
        kind: ProgramArtifactKind,
    ) -> Result<Utf8PathBuf> {
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

        let gen_manifest_path = crate_dir.join("Cargo.toml");
        write_file(
            &gen_manifest_path,
            gen_cargo_toml(
                self.program_package,
                sails_package,
                sails_interface_package,
                meta_path_version,
                kind,
                self.workspace_root,
            ),
        )?;

        let out_file = get_artifact_output_path(kind, self.target_dir, self.program_package);
        let main_rs_path = src_dir.join("main.rs");
        write_file(
            &main_rs_path,
            gen_main_rs(kind, program_struct_path, &out_file),
        )?;

        let from_lock = self.workspace_root.join("Cargo.lock");
        let to_lock = crate_dir.join("Cargo.lock");
        let _ = fs::copy(from_lock.as_std_path(), to_lock.as_std_path());

        let res = cargo_run_bin(&gen_manifest_path, &crate_name, self.target_dir);
        fs::remove_dir_all(crate_dir.as_std_path())?;

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

fn gen_cargo_toml(
    program_package: &Package,
    sails_package: &Package,
    sails_interface_package: Option<&Package>,
    meta_path_version: MetaPathVersion,
    kind: ProgramArtifactKind,
    workspace_root: &Utf8PathBuf,
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
    dep_table[&program_package.name] = toml_edit::value(package_table);

    let sails_dep = match meta_path_version {
        MetaPathVersion::V1 => sails_dep_v1(sails_package),
        MetaPathVersion::V2 => sails_dep_v2(sails_package, kind),
    };
    dep_table[&sails_package.name] = toml_edit::value(sails_dep);

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

        let patch_table = manifest
            .entry("patch")
            .or_insert(Item::Table(Table::new()))
            .as_table_mut()
            .expect("patch is a table")
            .entry("crates-io")
            .or_insert(Item::Table(Table::new()))
            .as_table_mut()
            .expect("patch.crates-io is a table");

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

fn sails_dep_v1(sails_package: &Package) -> InlineTable {
    let mut sails_table = InlineTable::new();
    sails_table.insert("package", "sails-idl-gen".into());
    sails_table.insert("version", sails_package.version.to_string().into());
    sails_table
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
    program_struct_path: &str,
    out_path: &Utf8PathBuf,
) -> String {
    match kind {
        ProgramArtifactKind::Idl => format!(
            r#"
#![allow(unexpected_cfgs)]

fn main() {{
    sails_rs::generate_idl_to_file::<{program_struct_path}>(
        std::path::PathBuf::from(r"{out_path}")
    )
    .expect("failed to generate IDL");
}}
"#
        ),
        ProgramArtifactKind::Canonical => format!(
            r#"
#![allow(unexpected_cfgs)]

fn main() {{
    use sails_idl_meta::ProgramMeta;
    use sails_interface_id::canonical::{{
        CanonicalDocument,
        CanonicalHashMeta,
        CANONICAL_HASH_ALGO,
        CANONICAL_SCHEMA,
        CANONICAL_VERSION,
    }};
    use sails_interface_id::runtime::build_canonical_document_from_meta;
    use sails_interface_id::INTERFACE_HASH_DOMAIN_STR;
    use std::collections::BTreeMap;

    let mut services = BTreeMap::new();
    let mut types = BTreeMap::new();
    for (_, service_fn) in <{program_struct_path} as ProgramMeta>::SERVICES {{
        let meta = service_fn();
        let doc = build_canonical_document_from_meta(&meta)
            .expect("failed to build canonical document");
        let (_, _, _, doc_services, doc_types) = doc.into_parts();
        services.extend(doc_services);
        types.extend(doc_types);
    }}

    let document = CanonicalDocument::from_parts(
        CANONICAL_SCHEMA,
        CANONICAL_VERSION,
        CanonicalHashMeta {{
            algo: CANONICAL_HASH_ALGO.to_owned(),
            domain: INTERFACE_HASH_DOMAIN_STR.to_owned(),
        }},
        services,
        types,
    );

    let bytes = document
        .to_bytes()
        .expect("failed to serialize canonical document");
    std::fs::write(std::path::PathBuf::from(r"{out_path}"), bytes)
        .expect("failed to write canonical document");
}}
"#
        ),
    }
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
        .arg("--manifest-path")
        .arg(manifest_path.as_str())
        .arg("--bin")
        .arg(bin_name);

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
        println!("{stderr}");
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
