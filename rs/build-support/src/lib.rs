//! Build-time helpers for generating canonical interface constants.
//!
//! Build scripts call [`emit_interface_consts`] to capture canonical metadata
//! for a service. The helper expects the crate to provide a host-only
//! `sails_meta_dump` binary that prints [`InterfaceArtifacts`] as JSON after
//! calling the generated `__sails_any_service_meta` hook.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use sails_idl_meta::{
    AnyServiceMeta, CanonicalizationContext, ParentInterface, ServiceUnit, build_service_unit,
    canonical,
    interface::{CanonicalEntry, EntryKind, EntryMeta, build_entry_meta_with_async},
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use toml::Value;

const CONSTS_DIRNAME: &str = "sails_interface_consts";
pub const GENERATED_MANIFEST_FILE: &str = "sails_services_manifest.rs";

#[macro_export]
macro_rules! generated_manifest_file {
    () => {
        "sails_services_manifest.rs"
    };
}

/// Declarative manifest derived from `[package.metadata.sails]` in `Cargo.toml`.
pub struct ServiceMetadata {
    services: Vec<String>,
}

impl ServiceMetadata {
    pub fn load_from_env() -> Result<Self> {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .context("CARGO_MANIFEST_DIR is not set in the environment")?;
        Self::load_from_dir(manifest_dir)
    }

    pub fn load_from_dir(manifest_dir: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = manifest_dir.as_ref().join("Cargo.toml");
        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read manifest at {manifest_path:?}"))?;
        let toml_value: Value =
            toml::from_str(&raw).with_context(|| format!("{manifest_path:?} is not valid TOML"))?;
        let services = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("metadata"))
            .and_then(|meta| meta.get("sails"))
            .and_then(|sails| sails.get("services"))
            .and_then(|services| services.as_array())
            .ok_or_else(|| anyhow!("package.metadata.sails.services is missing"))?
            .iter()
            .map(|item| {
                item.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow!("services entries must be strings"))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { services })
    }

    pub fn service_paths(&self) -> &[String] {
        &self.services
    }

    pub fn into_service_paths(self) -> Vec<String> {
        self.services
    }

    pub fn write_generated_manifest(&self, out_dir: &Path) -> Result<()> {
        let mut buf = String::new();
        buf.push_str("sails_build::service_manifest! {\n");
        buf.push_str("    services: [\n");
        for service in &self.services {
            buf.push_str("        ");
            buf.push_str(service);
            buf.push_str(",\n");
        }
        buf.push_str("    ],\n}");
        let manifest_path = out_dir.join(GENERATED_MANIFEST_FILE);
        write_if_changed(&manifest_path, buf.as_bytes())
    }
}

/// Loads service metadata from `Cargo.toml`, writes the generated manifest file
/// into `OUT_DIR`, and returns the parsed metadata for further use.
pub fn prepare_service_metadata(out_dir: &Path) -> Result<ServiceMetadata> {
    let metadata = ServiceMetadata::load_from_env()?;
    metadata.write_generated_manifest(out_dir)?;
    Ok(metadata)
}

/// Convenience builder for Sails-aware build scripts. Handles rerun directives,
/// conditional wasm builds, and canonical interface emission.
pub struct BuildScript<'a> {
    service_paths: Vec<String>,
    manifest_path: Option<&'a str>,
    sails_crate: &'a str,
    meta_dump_features: &'a [&'a str],
    wasm: Option<WasmBuildConfig<'a>>,
    rerun_src: bool,
    before_emit: Option<fn()>,
}

impl<'a> BuildScript<'a> {
    pub fn new(service_paths: &'a [&'a str]) -> Self {
        Self {
            service_paths: service_paths.iter().map(|path| path.to_string()).collect(),
            manifest_path: None,
            sails_crate: "sails_rs",
            meta_dump_features: &[],
            wasm: None,
            rerun_src: true,
            before_emit: None,
        }
    }

    pub fn from_service_paths(service_paths: Vec<String>) -> Self {
        Self {
            service_paths,
            manifest_path: None,
            sails_crate: "sails_rs",
            meta_dump_features: &[],
            wasm: None,
            rerun_src: true,
            before_emit: None,
        }
    }

    pub fn manifest_path(mut self, path: &'a str) -> Self {
        self.manifest_path = Some(path);
        self
    }

    pub fn sails_crate_path(mut self, path: &'a str) -> Self {
        self.sails_crate = path;
        self
    }

    pub fn meta_dump_features(mut self, features: &'a [&'a str]) -> Self {
        self.meta_dump_features = features;
        self
    }

    pub fn wasm_build(mut self, config: WasmBuildConfig<'a>) -> Self {
        self.wasm = Some(config);
        self
    }

    pub fn emit_src_rerun(mut self, enabled: bool) -> Self {
        self.rerun_src = enabled;
        self
    }

    pub fn before_emit(mut self, callback: fn()) -> Self {
        self.before_emit = Some(callback);
        self
    }

    /// Executes the configured build steps.
    pub fn run(self) -> Result<()> {
        if self.rerun_src {
            println!("cargo:rerun-if-changed=src");
        }
        if let Some(path) = self.manifest_path {
            println!("cargo:rerun-if-changed={path}");
        }

        let canonical_dump = env::var_os("SAILS_CANONICAL_DUMP").is_some();
        if canonical_dump {
            println!("cargo:rustc-cfg=sails_canonical_dump");
        }
        println!("cargo:rustc-check-cfg=cfg(sails_canonical_dump)");
        println!(
            "cargo:rustc-check-cfg=cfg(feature, values(\"sails-canonical\", \"sails-meta-dump\"))"
        );

        let meta_dump_mode = env::var_os("CARGO_FEATURE_SAILS_META_DUMP").is_some();
        if (canonical_dump || meta_dump_mode) && self.wasm.is_some() {
            write_stub_wasm_binary()?;
        }

        if canonical_dump || meta_dump_mode {
            return Ok(());
        }

        if let Some(cfg) = &self.wasm {
            cfg.maybe_run();
        }

        if env::var_os("CARGO_FEATURE_SAILS_CANONICAL").is_none() {
            return Ok(());
        }

        if self.service_paths.is_empty() {
            eprintln!("[sails-build] SERVICE_PATHS is empty; nothing to canonicalize");
            return Ok(());
        }

        if let Some(callback) = self.before_emit {
            callback();
        }

        let out_dir =
            PathBuf::from(env::var("OUT_DIR").context("OUT_DIR is not set in the environment")?);
        emit_interface_consts_impl(
            Some(&self.service_paths),
            self.sails_crate,
            self.meta_dump_features,
            &out_dir,
        )
    }
}

/// Configuration for optional wasm builds driven by build scripts.
pub struct WasmBuildConfig<'a> {
    gate_env: &'a str,
    skip_env: &'a [&'a str],
    skip_features: &'a [&'a str],
    runner: fn(),
}

impl<'a> WasmBuildConfig<'a> {
    pub fn new(gate_env: &'a str, runner: fn()) -> Self {
        Self {
            gate_env,
            skip_env: &[],
            skip_features: &[],
            runner,
        }
    }

    pub fn skip_env(mut self, vars: &'a [&'a str]) -> Self {
        self.skip_env = vars;
        self
    }

    pub fn skip_features(mut self, features: &'a [&'a str]) -> Self {
        self.skip_features = features;
        self
    }

    fn maybe_run(&self) {
        if self.skip_env.iter().any(|var| env::var_os(var).is_some()) {
            return;
        }
        if self
            .skip_features
            .iter()
            .any(|feat| env::var_os(feat).is_some())
        {
            return;
        }
        if env::var_os(self.gate_env).is_some() {
            (self.runner)();
        }
    }
}

/// Emit canonical interface constants for every service listed in the manifest
/// payload (see [`service_manifest!`]), assuming the crate uses the default
/// `sails_rs` import path.
pub fn emit_interface_consts(out_dir: impl AsRef<Path>) -> Result<()> {
    emit_interface_consts_impl(None, "sails_rs", &[], out_dir)
}

/// Same as [`emit_interface_consts`], but with a custom path to the `sails-rs`
/// crate (for crates that re-export it under another name).
pub fn emit_interface_consts_with_sails_path(
    sails_crate_path: &str,
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    emit_interface_consts_impl(None, sails_crate_path, &[], out_dir)
}

/// Fully configurable variant that allows specifying both the `sails-rs` crate
/// path and extra features required to compile the `sails_meta_dump` binary.
pub fn emit_interface_consts_with_options(
    sails_crate_path: &str,
    meta_dump_features: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    emit_interface_consts_impl(None, sails_crate_path, meta_dump_features, out_dir)
}

fn emit_interface_consts_impl(
    expected_services: Option<&[String]>,
    sails_crate_path: &str,
    meta_dump_features: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .context("CARGO_MANIFEST_DIR is not set in the environment")?,
    );
    let artifacts = run_meta_dump_all(meta_dump_features, &manifest_dir)?;
    if let Some(expected) = expected_services {
        validate_service_artifacts(expected, &artifacts)?;
    }
    if artifacts.is_empty() {
        return Ok(());
    }

    let consts_dir = out_dir.as_ref().join(CONSTS_DIRNAME);
    fs::create_dir_all(&consts_dir).with_context(|| format!("failed to create {consts_dir:?}"))?;

    for entry in &artifacts {
        let rendered = render_consts(&entry.artifacts, sails_crate_path);
        let filename = consts_filename(&entry.service_path);
        let path = consts_dir.join(filename);
        write_if_changed(&path, rendered.as_bytes())?;
    }

    write_manifest(&consts_dir, &artifacts)
}

fn write_stub_wasm_binary() -> Result<()> {
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").context("OUT_DIR is not set in the environment")?);
    let stub = br#"#[allow(unused)]
pub const WASM_BINARY: &[u8] = &[];
#[allow(unused)]
pub const WASM_BINARY_OPT: &[u8] = &[];
"#;
    write_if_changed(&out_dir.join("wasm_binary.rs"), stub)
}

fn run_meta_dump_all(
    meta_dump_features: &[&str],
    manifest_dir: &Path,
) -> Result<Vec<ServiceArtifacts>> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(manifest_dir);
    cmd.arg("run");
    if let Ok(host_target) = env::var("HOST") {
        if !host_target.is_empty() {
            eprintln!("[sails-build] HOST={host_target}; targeting host triple for meta dump");
            cmd.arg("--target").arg(host_target);
        }
    }
    cmd.args(["--bin", "sails_meta_dump", "--quiet"]);
    for feature in meta_dump_features {
        cmd.arg("--features").arg(feature);
    }
    let base_target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let host_target_dir = base_target_dir.join("sails-meta-dump");
    cmd.arg("--target-dir").arg(&host_target_dir);
    cmd.env("CARGO_TARGET_DIR", &host_target_dir);
    cmd.args(["--", "--all"]);
    cmd.env("SAILS_CANONICAL_DUMP", "1");
    let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.is_empty() && !rustflags.ends_with(' ') {
        rustflags.push(' ');
    }
    rustflags.push_str("--cfg sails_canonical_dump");
    cmd.env("RUSTFLAGS", rustflags);
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");

    eprintln!(
        "[sails-build] running sails_meta_dump for all services (features: {:?}) in {} (host target-dir: {})",
        meta_dump_features,
        manifest_dir.display(),
        host_target_dir.display()
    );
    let output = cmd
        .output()
        .context("failed to execute sails_meta_dump helper")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("sails_meta_dump failed: {stderr}");
    }
    eprintln!(
        "[sails-build] sails_meta_dump finished; received {} bytes",
        output.stdout.len()
    );

    parse_meta_dump_stream(&output.stdout)
}

fn parse_meta_dump_stream(bytes: &[u8]) -> Result<Vec<ServiceArtifacts>> {
    let mut artifacts = Vec::new();
    for chunk in bytes.split(|b| *b == b'\n') {
        let line = if let Some(stripped) = chunk.strip_suffix(b"\r") {
            stripped
        } else {
            chunk
        };
        if line.is_empty() {
            continue;
        }
        let record: ServiceArtifactsRecord =
            serde_json::from_slice(line).context("failed to parse sails_meta_dump record")?;
        artifacts.push(ServiceArtifacts {
            service_path: record.service,
            artifacts: record.artifacts,
        });
    }
    Ok(artifacts)
}

fn render_consts(artifacts: &InterfaceArtifacts, sails_crate_path: &str) -> String {
    let mut buf = String::new();
    buf.push_str("// @generated by sails-build. DO NOT EDIT.\n");
    buf.push_str(&format!(
        "use {sails_crate_path}::meta::{{EntryKind, EntryMeta}};\n\n"
    ));
    buf.push_str(&format!(
        "pub const INTERFACE_ID: u64 = {interface_id:#018x};\n\n",
        interface_id = artifacts.interface_id,
    ));
    buf.push_str("pub const ENTRY_META: &[EntryMeta<'static>] = &[\n");
    for entry in &artifacts.entry_meta {
        buf.push_str("    EntryMeta::borrowed(");
        buf.push_str(&format!(
            "{name:?}, {entry_id}u16, EntryKind::{kind}, {is_async}),\n",
            name = entry.name,
            entry_id = entry.entry_id,
            kind = entry.kind.as_variant(),
            is_async = entry.is_async,
        ));
    }
    buf.push_str("];\n\n");
    buf.push_str("pub const CANONICAL_INTERFACE_JSON: &[u8] = &[\n");
    buf.push_str(&format_byte_array(artifacts.canonical_json.as_ref()));
    buf.push_str("];\n");
    buf
}

fn format_byte_array(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    const PER_LINE: usize = 12;
    let mut buf = String::new();
    for (idx, byte) in bytes.iter().enumerate() {
        if idx % PER_LINE == 0 {
            buf.push_str("    ");
        }
        buf.push_str(&format!("0x{byte:02X}, "));
        if idx % PER_LINE == PER_LINE - 1 {
            buf.push('\n');
        }
    }
    if bytes.len() % PER_LINE != 0 {
        buf.push('\n');
    }
    buf
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<()> {
    let should_write = match fs::read(path) {
        Ok(existing) => existing != contents,
        Err(_) => true,
    };
    if should_write {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("failed to create {parent:?}"))?;
        }
        fs::write(path, contents)
            .with_context(|| format!("failed to write generated data to {path:?}"))?;
    }
    Ok(())
}

fn consts_filename(service_path: &str) -> String {
    let trimmed = service_path.trim();
    let without_generics = if let Some(idx) = trimmed.find('<') {
        &trimmed[..idx]
    } else {
        trimmed
    };
    let stem = without_generics
        .rsplit("::")
        .next()
        .unwrap_or(without_generics)
        .trim();
    let stem = stem
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    format!("{stem}.rs")
}

fn write_manifest(consts_dir: &Path, artifacts: &[ServiceArtifacts]) -> Result<()> {
    let manifest_path = consts_dir.join("manifest.json");
    let records: Vec<ServiceArtifactsRecord> = artifacts
        .iter()
        .map(|entry| ServiceArtifactsRecord {
            service: entry.service_path.clone(),
            artifacts: entry.artifacts.clone(),
        })
        .collect();
    let data =
        serde_json::to_vec_pretty(&records).context("failed to serialize canonical manifest")?;
    write_if_changed(&manifest_path, &data)
}

/// Canonical artifacts emitted by the helper binary and consumed by build
/// scripts. The same structure can be serialized or deserialized via JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceArtifacts {
    pub interface_id: u64,
    #[serde(with = "serde_bytes")]
    pub canonical_json: ByteBuf,
    pub entry_meta: Vec<EntryRecord>,
    #[serde(default, skip_serializing_if = "DiagnosticsRecord::is_empty")]
    pub diagnostics: DiagnosticsRecord,
}

#[derive(Debug, Clone)]
struct ServiceArtifacts {
    service_path: String,
    artifacts: InterfaceArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceArtifactsRecord {
    service: String,
    artifacts: InterfaceArtifacts,
}

fn validate_service_artifacts(expected: &[String], artifacts: &[ServiceArtifacts]) -> Result<()> {
    let expected_set = expected
        .iter()
        .map(|path| normalized_service_name(path))
        .collect::<BTreeSet<_>>();
    let actual_set = artifacts
        .iter()
        .map(|entry| normalized_service_name(&entry.service_path))
        .collect::<BTreeSet<_>>();

    if expected_set == actual_set {
        return Ok(());
    }

    if !expected_set.is_subset(&actual_set) {
        eprintln!("[sails-build] missing services in sails_meta_dump output:");
        for name in expected_set.difference(&actual_set) {
            eprintln!("  - {name}");
        }
    }

    if !actual_set.is_subset(&expected_set) {
        eprintln!("[sails-build] unexpected services returned by sails_meta_dump:");
        for name in actual_set.difference(&expected_set) {
            eprintln!("  - {name}");
        }
    }

    bail!("service manifest differs from sails_meta_dump registry")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRecord {
    pub name: String,
    pub entry_id: u16,
    pub kind: EntryKindRepr,
    pub is_async: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticsRecord {
    pub type_bindings: BTreeMap<String, String>,
    pub type_display_names: BTreeMap<String, String>,
}

impl DiagnosticsRecord {
    fn from_envelope(envelope: &canonical::CanonicalEnvelope) -> Self {
        let mut display_names = BTreeMap::new();
        for (type_id, ty) in &envelope.types {
            if let Some(name) = &ty.display_name {
                display_names.insert(type_id.clone(), name.clone());
            }
        }
        Self {
            type_bindings: envelope.type_bindings.clone(),
            type_display_names: display_names,
        }
    }

    fn is_empty(&self) -> bool {
        self.type_bindings.is_empty() && self.type_display_names.is_empty()
    }
}

impl EntryRecord {
    fn from_entry(meta: EntryMeta<'static>) -> Self {
        Self {
            name: meta.name.into_owned(),
            entry_id: meta.entry_id,
            kind: EntryKindRepr::from(meta.kind),
            is_async: meta.is_async,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKindRepr {
    Command,
    Query,
    Event,
}

impl EntryKindRepr {
    fn as_variant(&self) -> &'static str {
        match self {
            EntryKindRepr::Command => "Command",
            EntryKindRepr::Query => "Query",
            EntryKindRepr::Event => "Event",
        }
    }
}

impl From<EntryKind> for EntryKindRepr {
    fn from(kind: EntryKind) -> Self {
        match kind {
            EntryKind::Command => EntryKindRepr::Command,
            EntryKind::Query => EntryKindRepr::Query,
            EntryKind::Event => EntryKindRepr::Event,
        }
    }
}

/// Canonicalize a service using [`AnyServiceMeta`] plus an async lookup helper.
///
/// The caller is responsible for providing a closure that reports whether a
/// canonical entry represents an async handler. The service crate can source
/// that information from the generated metadata helpers.
pub fn canonicalize_service_with_async(
    service_name: &str,
    meta: &AnyServiceMeta,
    mut async_lookup: impl FnMut(&CanonicalEntry) -> bool,
) -> Result<InterfaceArtifacts> {
    let (service_unit, parent_units) = build_service_unit_with_parents(service_name, meta)?;
    let parent_refs = build_parent_refs(&parent_units);
    let context = if parent_refs.is_empty() {
        CanonicalizationContext::default()
    } else {
        CanonicalizationContext::with_parents(parent_refs.as_slice())
    };
    let result = canonical::compute_interface_id(&service_unit, &context)
        .map_err(|err| anyhow!("canonicalization failed: {err}"))?;

    let entry_meta_vec = build_entry_meta_with_async(&result.envelope, |entry| async_lookup(entry))
        .map_err(|err| anyhow!("entry metadata generation failed: {err}"))?;

    Ok(InterfaceArtifacts {
        interface_id: result.interface_id,
        canonical_json: ByteBuf::from(result.canonical_json),
        entry_meta: entry_meta_vec
            .into_iter()
            .map(EntryRecord::from_entry)
            .collect(),
        diagnostics: DiagnosticsRecord::from_envelope(&result.envelope),
    })
}

fn build_service_unit_with_parents(
    service_name: &str,
    meta: &AnyServiceMeta,
) -> Result<(ServiceUnit, Vec<ParentUnit>)> {
    let mut parent_units = Vec::new();
    for base in meta.base_services() {
        let base_name = short_type_name(base.type_name()).to_string();
        let (base_unit, base_parent_units) = build_service_unit_with_parents(&base_name, base)?;
        let base_refs = build_parent_refs(&base_parent_units);
        let base_ctx = if base_refs.is_empty() {
            CanonicalizationContext::default()
        } else {
            CanonicalizationContext::with_parents(base_refs.as_slice())
        };
        let result = canonical::compute_interface_id(&base_unit, &base_ctx)
            .map_err(|err| anyhow!("canonicalization failed for parent {base_name}: {err}"))?;
        parent_units.push(ParentUnit {
            service: base_unit,
            interface_id: result.interface_id,
        });
    }

    let service_unit = build_service_unit(service_name, meta)
        .map_err(|err| anyhow!("failed to build service AST: {err}"))?;
    Ok((service_unit, parent_units))
}

fn build_parent_refs<'a>(parents: &'a [ParentUnit]) -> Vec<ParentInterface<'a>> {
    parents
        .iter()
        .map(|parent| ParentInterface::new(&parent.service, parent.interface_id))
        .collect()
}

fn short_type_name(full: &str) -> &str {
    full.rsplit("::").next().unwrap_or(full)
}

#[derive(Clone)]
struct ParentUnit {
    service: ServiceUnit,
    interface_id: u64,
}

/// Describes a service that the host meta-dump binary can canonicalize.
pub struct DumpService {
    pub name: &'static str,
    pub make_artifacts: fn() -> Result<InterfaceArtifacts>,
}

impl DumpService {
    pub const fn new(
        name: &'static str,
        make_artifacts: fn() -> Result<InterfaceArtifacts>,
    ) -> Self {
        Self {
            name,
            make_artifacts,
        }
    }
}

/// Runs the canonicalization CLI (used by `sails_meta_dump`).
///
/// ```ignore
/// use sails_build::{run_meta_dump_cli, service_dump};
/// use sails_idl_meta::interface::{CanonicalEntry, EntryKind};
///
/// fn main() -> anyhow::Result<()> {
///     run_meta_dump_cli(&[service_dump!(
///         crate::MyService,
///         |entry: &CanonicalEntry| matches!((entry.name.as_str(), entry.kind), ("DoThis", EntryKind::Command) => true, _ => false)
///     )])
/// }
/// ```
pub fn run_meta_dump_cli(services: &[DumpService]) -> Result<()> {
    let mut service_name: Option<String> = None;
    let mut args = env::args().skip(1);
    let mut dump_all = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--service" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--service requires a value"))?;
                service_name = Some(value);
            }
            "--all" => {
                dump_all = true;
            }
            "--list" => {
                for svc in services {
                    println!("{}", svc.name);
                }
                return Ok(());
            }
            "--help" | "-h" => {
                print_usage(services);
                return Ok(());
            }
            other => bail!("unrecognized argument `{other}`; pass --help for usage"),
        }
    }

    if dump_all {
        if service_name.is_some() {
            bail!("--all cannot be combined with --service");
        }
        dump_all_services(services)
    } else {
        let selected = if let Some(name) = service_name {
            services
                .iter()
                .find(|svc| service_name_matches(svc.name, &name))
                .ok_or_else(|| anyhow!("unknown service `{name}`"))?
        } else if services.len() == 1 {
            &services[0]
        } else {
            bail!("multiple services available; pass --service <path> or --all");
        };

        let artifacts = (selected.make_artifacts)()?;
        serde_json::to_writer(io::stdout().lock(), &artifacts)
            .context("failed to serialize canonical artifacts")?;
        Ok(())
    }
}

fn dump_all_services(services: &[DumpService]) -> Result<()> {
    let mut stdout = io::stdout().lock();
    for svc in services {
        let artifacts = (svc.make_artifacts)()?;
        let record = ServiceArtifactsRecord {
            service: svc.name.to_string(),
            artifacts,
        };
        serde_json::to_writer(&mut stdout, &record)
            .context("failed to serialize canonical artifacts stream")?;
        stdout
            .write_all(b"\n")
            .context("failed to write canonical artifacts delimiter")?;
    }
    Ok(())
}

fn service_name_matches(candidate: &str, query: &str) -> bool {
    normalized_service_name(candidate) == normalized_service_name(query)
}

fn normalized_service_name(name: &str) -> String {
    fn trim_leading_colons(mut input: &str) -> &str {
        while let Some(stripped) = input.strip_prefix("::") {
            input = stripped;
        }
        input
    }

    let compact = name
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let mut normalized = trim_leading_colons(compact.as_str());
    if let Some(stripped) = normalized.strip_prefix("crate::") {
        normalized = stripped;
    }
    normalized = trim_leading_colons(normalized);
    normalized.to_string()
}

fn print_usage(services: &[DumpService]) {
    let mut stderr = io::stderr().lock();
    writeln!(
        &mut stderr,
        "Usage: sails_meta_dump [--service <path> | --all] [--list]"
    )
    .ok();
    writeln!(&mut stderr, "\nAvailable services:").ok();
    for svc in services {
        writeln!(&mut stderr, "  - {}", svc.name).ok();
    }
}

/// Helper macro for defining a [`DumpService`] entry inside the binary.
///
/// ```ignore
/// use sails_build::{run_meta_dump_cli, service_dump};
/// use sails_idl_meta::interface::{CanonicalEntry, EntryKind};
///
/// fn main() -> anyhow::Result<()> {
///     let services = [service_dump!(
///         crate::MyService,
///         |entry: &CanonicalEntry| entry.kind == EntryKind::Query
///     )];
///     run_meta_dump_cli(&services)
/// }
/// ```
#[macro_export]
macro_rules! service_dump {
    ($service:path, $async_lookup:expr) => {{
        fn __sails_dump_fn() -> ::anyhow::Result<$crate::InterfaceArtifacts> {
            let meta = <$service>::__sails_any_service_meta();
            $crate::canonicalize_service_with_async(stringify!($service), &meta, $async_lookup)
        }
        $crate::DumpService::new(stringify!($service), __sails_dump_fn)
    }};
}

#[derive(Clone, Copy)]
pub struct ServiceManifest {
    pub paths: &'static [&'static str],
    pub registry: &'static [DumpService],
}

impl ServiceManifest {
    pub const fn new(paths: &'static [&'static str], registry: &'static [DumpService]) -> Self {
        Self { paths, registry }
    }
}

/// Expands a manifest payload into a [`ServiceManifest`], suitable for runtime
/// helpers such as `sails_meta_dump`.
///
/// This macro accepts either a plain list of service paths or a block payload.
/// Typical projects point this macro at the generated manifest emitted by
/// [`prepare_service_metadata`], which writes the raw payload to
/// `$OUT_DIR/sails_services_manifest.rs`. Defining a local
/// `sails_services_manifest!` macro is still supported for projects that prefer
/// handwritten manifests, but most consumers simply
/// `include!(concat!(env!("OUT_DIR"), "/", sails_build::GENERATED_MANIFEST_FILE))`.
///
/// ```ignore
/// macro_rules! sails_services_manifest {
///     ($($tt:tt)*) => {
///         sails_build::service_manifest!($($tt)*)
///     };
/// }
///
/// const SERVICE_MANIFEST: sails_build::ServiceManifest =
///     include!(concat!(env!("OUT_DIR"), "/", sails_build::GENERATED_MANIFEST_FILE));
/// ```
#[macro_export]
macro_rules! service_manifest {
    ($($tt:tt)*) => {{
        $crate::__sails_parse_service_manifest!(@manifest $($tt)*)
    }};
}

/// Expands a manifest payload into a static slice of service path strings. This
/// form is lightweight enough to use inside build scripts, because it never
/// references the actual service types.
///
/// When using [`prepare_service_metadata`], this macro primarily serves internal
/// use within the generated manifest file, so most end users no longer invoke it
/// directly.
#[macro_export]
macro_rules! service_paths {
    ($($tt:tt)*) => {{
        $crate::__sails_parse_service_manifest!(@paths $($tt)*)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sails_parse_service_manifest {
    (@$mode:ident { $($tt:tt)* }) => {
        $crate::__sails_parse_service_manifest!(@$mode $($tt)*)
    };
    (@$mode:ident
        $(type $alias:ident = $ty:ty;)*
        services: [
            $($path:path),* $(,)?
        ] $(,)?
    ) => {
        $crate::__sails_emit_service_manifest!(
            @$mode
            [ $( ($alias $ty) )* ]
            [ $( $path ),* ]
        )
    };
    (@$mode:ident $($path:path),* $(,)?) => {
        $crate::__sails_emit_service_manifest!(@$mode [] [ $($path),* ])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sails_emit_service_manifest {
    (@manifest [$(($alias:ident $ty:ty))*] [ $($path:path),* ]) => {{
        $(#[allow(dead_code)] type $alias = $ty;)*
        $crate::ServiceManifest::new(
            &[$(stringify!($path)),*],
            &[
                $(
                    $crate::service_dump!(
                        $path,
                        |entry| <$path>::__sails_entry_async(entry)
                    )
                ),*
            ],
        )
    }};
    (@paths [$(($alias:ident $ty:ty))*] [ $($path:path),* ]) => {{
        const __SAILS_PATHS: &[&str] = &[$(stringify!($path)),*];
        __SAILS_PATHS
    }};
}

/// Backwards-compatible alias for [`service_manifest!`]. Prefer calling
/// [`service_manifest!`] and [`service_paths!`] directly so build scripts can
/// include the same manifest payload as runtime helpers.
#[macro_export]
macro_rules! sails_services {
    ($($tt:tt)*) => {{
        $crate::service_manifest!($($tt)*)
    }};
}
