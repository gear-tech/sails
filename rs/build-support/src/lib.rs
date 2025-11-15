//! Build-time helpers for generating canonical interface constants.
//!
//! Build scripts call [`emit_interface_consts`] to capture canonical metadata
//! for a service. The helper expects the crate to provide a host-only
//! `sails_meta_dump` binary that prints [`InterfaceArtifacts`] as JSON after
//! calling the generated `__sails_any_service_meta` hook.

use std::{
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

const CONSTS_DIRNAME: &str = "sails_interface_consts";

/// Emit canonical interface constants for every service listed in the
/// `sails_services!` manifest, assuming the crate uses the default `sails_rs`
/// import path.
pub fn emit_interface_consts(out_dir: impl AsRef<Path>) -> Result<()> {
    emit_interface_consts_with_options("sails_rs", &[], out_dir)
}

/// Same as [`emit_interface_consts`], but with a custom path to the `sails-rs`
/// crate (for crates that re-export it under another name).
pub fn emit_interface_consts_with_sails_path(
    sails_crate_path: &str,
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    emit_interface_consts_with_options(sails_crate_path, &[], out_dir)
}

/// Fully configurable variant that allows specifying both the `sails-rs` crate
/// path and extra features required to compile the `sails_meta_dump` binary.
pub fn emit_interface_consts_with_options(
    sails_crate_path: &str,
    meta_dump_features: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .context("CARGO_MANIFEST_DIR is not set in the environment")?,
    );
    let artifacts = run_meta_dump_all(meta_dump_features, &manifest_dir)?;
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
        let line = if let Some(stripped) = chunk.strip_suffix(&[b'\r']) {
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
        "use {path}::meta::{{EntryKind, EntryMeta}};\n\n",
        path = sails_crate_path
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRecord {
    pub name: String,
    pub entry_id: u16,
    pub kind: EntryKindRepr,
    pub is_async: bool,
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
/// ```no_run
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
    normalize_service_name(candidate) == normalize_service_name(query)
}

fn normalize_service_name(name: &str) -> &str {
    fn trim_leading_colons(mut input: &str) -> &str {
        while let Some(stripped) = input.strip_prefix("::") {
            input = stripped;
        }
        input
    }

    let mut normalized = name.trim();
    normalized = trim_leading_colons(normalized);
    if let Some(stripped) = normalized.strip_prefix("crate::") {
        normalized = stripped;
    }
    trim_leading_colons(normalized)
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
/// ```no_run
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
