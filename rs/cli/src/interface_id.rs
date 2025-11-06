use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::camino::Utf8PathBuf;
use sails_build_support::{ProgramArtifactKind, generate_program_artifact};
use sails_interface_id::{canonical::CanonicalDocument, compute_ids_from_bytes};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::Path,
};

pub fn canonicalize_manifest(manifest: &Path, output: Option<&Path>) -> Result<()> {
    let manifest = to_utf8_path(manifest)?;
    let canonical_path =
        generate_program_artifact(manifest.as_std_path(), None, ProgramArtifactKind::Canonical)?;
    let canonical_bytes = fs::read(&canonical_path)
        .with_context(|| format!("failed to read canonical document {canonical_path}"))?;

    if let Some(out_path) = output {
        fs::write(out_path, &canonical_bytes).with_context(|| {
            let path = out_path.display().to_string();
            format!("failed to write canonical document {path}")
        })?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(&canonical_bytes)?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}

pub fn derive_ids(input: &Path) -> Result<()> {
    let is_idl = input
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("idl"))
        .unwrap_or(false);
    if is_idl {
        bail!(
            "deriving interface IDs directly from `.idl` files is not supported; generate canonical JSON via `sails idl-canonicalize --manifest-path <Cargo.toml>` and pass it with `--canonical-path`"
        );
    }

    let raw = fs::read_to_string(input).with_context(|| {
        let path = input.display().to_string();
        format!("failed to read canonical document {path}")
    })?;
    let canonical = CanonicalDocument::from_json_str(&raw)?;

    if canonical.services().is_empty() {
        let bytes = canonical.to_bytes()?;
        let id = compute_ids_from_bytes(&bytes);
        println!("document -> interface_id=0x{id:016x}");
        return Ok(());
    }

    for (name, service) in canonical.services() {
        let mut single_services = BTreeMap::new();
        single_services.insert(name.clone(), service.clone());
        let single = CanonicalDocument::from_parts(
            canonical.canon_schema(),
            canonical.canon_version(),
            canonical.hash().clone(),
            single_services,
            canonical.types().clone(),
        );
        let bytes = single.to_bytes()?;
        let id = compute_ids_from_bytes(&bytes);
        println!("{name} -> interface_id=0x{id:016x}");
    }

    Ok(())
}

pub fn derive_ids_for_manifest(manifest: &Path) -> Result<()> {
    let manifest = to_utf8_path(manifest)?;
    let canonical_path =
        generate_program_artifact(manifest.as_std_path(), None, ProgramArtifactKind::Canonical)?;
    println!("Generated canonical document at {canonical_path}");
    let canonical_bytes = fs::read(&canonical_path)
        .with_context(|| format!("failed to read canonical document {canonical_path}"))?;
    let canonical_str = String::from_utf8(canonical_bytes.clone())
        .context("canonical document is not valid UTF-8")?;
    let canonical = CanonicalDocument::from_json_str(&canonical_str)?;

    if canonical.services().is_empty() {
        let id = compute_ids_from_bytes(&canonical_bytes);
        println!("document -> interface_id=0x{id:016x}");
        return Ok(());
    }

    for (name, service) in canonical.services() {
        let mut single_services = BTreeMap::new();
        single_services.insert(name.clone(), service.clone());
        let single = CanonicalDocument::from_parts(
            canonical.canon_schema(),
            canonical.canon_version(),
            canonical.hash().clone(),
            single_services,
            canonical.types().clone(),
        );
        let bytes = single.to_bytes()?;
        let id = compute_ids_from_bytes(&bytes);
        println!("{name} -> interface_id=0x{id:016x}");
    }

    Ok(())
}

fn to_utf8_path(path: &Path) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|_| anyhow!("path {} is not valid UTF-8", path.display()))
}
