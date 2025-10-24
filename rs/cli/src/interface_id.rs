use crate::idlgen::{ProgramArtifactKind, generate_program_artifact};
use anyhow::{Context, Result, anyhow};
use cargo_metadata::camino::Utf8PathBuf;
use sails_interface_id::{
    canonical::{CanonicalDocument, CanonicalizationError, canonicalize},
    compute_ids_from_bytes,
};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::Path,
};

pub fn canonicalize_path(input: &Path, output: Option<&Path>) -> Result<()> {
    let raw = fs::read_to_string(input)
        .with_context(|| format!("failed to read IDL file {}", input.display()))?;
    let canonical_bytes = canonicalize(&raw)?;

    if let Some(out_path) = output {
        fs::write(out_path, &canonical_bytes)
            .with_context(|| format!("failed to write canonical IDL {}", out_path.display()))?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(&canonical_bytes)?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}

pub fn canonicalize_manifest(manifest: &Path, output: Option<&Path>) -> Result<()> {
    let manifest = to_utf8_path(manifest)?;
    let canonical_path =
        generate_program_artifact(manifest.as_path(), None, 1, ProgramArtifactKind::Canonical)?;
    let canonical_bytes =
        fs::read(&canonical_path).with_context(|| format!("failed to read {}", canonical_path))?;

    if let Some(out_path) = output {
        fs::write(out_path, &canonical_bytes)
            .with_context(|| format!("failed to write canonical IDL {}", out_path.display()))?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(&canonical_bytes)?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}

pub fn derive_ids(input: &Path) -> Result<()> {
    let raw = fs::read_to_string(input)
        .with_context(|| format!("failed to read IDL file {}", input.display()))?;
    let (canonical, overrides) = match CanonicalDocument::from_json_str(&raw) {
        Ok(doc) => (doc, BTreeMap::new()),
        Err(CanonicalizationError::InvalidJson(_)) => {
            let doc = CanonicalDocument::from_text_idl(&raw)?;
            let ids = collect_interface_ids(&raw);
            (doc, ids)
        }
        Err(err) => return Err(err.into()),
    };

    if canonical.services.is_empty() {
        let bytes = canonical.to_bytes()?;
        let id = compute_ids_from_bytes(&bytes);
        println!("document -> interface_id=0x{ID:016x}", ID = id);
        return Ok(());
    }

    for (name, service) in canonical.services.iter() {
        let mut single_services = BTreeMap::new();
        single_services.insert(name.clone(), service.clone());
        let single = CanonicalDocument {
            canon_schema: canonical.canon_schema.clone(),
            canon_version: canonical.canon_version.clone(),
            hash: canonical.hash.clone(),
            services: single_services,
            types: canonical.types.clone(),
        };
        let id = if let Some(maybe_id) = overrides.get(name) {
            match maybe_id {
                Some(id) => *id,
                _ => {
                    let bytes = single.to_bytes()?;
                    compute_ids_from_bytes(&bytes)
                }
            }
        } else {
            let bytes = single.to_bytes()?;
            compute_ids_from_bytes(&bytes)
        };
        println!("{} -> interface_id=0x{ID:016x}", name, ID = id);
    }

    Ok(())
}

pub fn derive_ids_for_manifest(manifest: &Path) -> Result<()> {
    let manifest = to_utf8_path(manifest)?;
    let canonical_path =
        generate_program_artifact(manifest.as_path(), None, 1, ProgramArtifactKind::Canonical)?;
    println!("Generated canonical IDL at {}", canonical_path);
    let canonical_bytes =
        fs::read(&canonical_path).with_context(|| format!("failed to read {}", canonical_path))?;
    let canonical_str = String::from_utf8(canonical_bytes.clone())
        .context("canonical document is not valid UTF-8")?;
    let canonical = CanonicalDocument::from_json_str(&canonical_str)?;

    if canonical.services.is_empty() {
        let id = compute_ids_from_bytes(&canonical_bytes);
        println!("document -> interface_id=0x{ID:016x}", ID = id);
        return Ok(());
    }

    for (name, service) in canonical.services.iter() {
        let mut single_services = BTreeMap::new();
        single_services.insert(name.clone(), service.clone());
        let single = CanonicalDocument {
            canon_schema: canonical.canon_schema.clone(),
            canon_version: canonical.canon_version.clone(),
            hash: canonical.hash.clone(),
            services: single_services,
            types: canonical.types.clone(),
        };
        let bytes = single.to_bytes()?;
        let id = compute_ids_from_bytes(&bytes);
        println!("{} -> interface_id=0x{ID:016x}", name, ID = id);
    }

    Ok(())
}

fn collect_interface_ids(input: &str) -> BTreeMap<String, Option<u64>> {
    let mut ids = BTreeMap::new();
    let mut current_service: Option<String> = None;
    let mut brace_depth: i32 = 0;

    for line in input.lines() {
        let trimmed = line.trim();

        if current_service.is_none() {
            if let Some(rest) = trimmed.strip_prefix("service ") {
                let raw_name = rest.split_whitespace().next().unwrap_or_default();
                let name = raw_name.trim_end_matches('{').trim_end_matches(';');
                if !name.is_empty() {
                    ids.entry(name.to_owned()).or_insert(None);
                    current_service = Some(name.to_owned());
                    brace_depth = count_brace_delta(trimmed);
                    continue;
                }
            }
            continue;
        }

        if let Some(service) = current_service.as_ref() {
            if let Some(rest) = trimmed.strip_prefix("///") {
                if let Some(entry) = ids.get_mut(service) {
                    let comment = rest.trim();
                    if let Some(value) = comment.strip_prefix("!@interface_id") {
                        *entry = parse_u64(value);
                    }
                }
            }
            brace_depth += count_brace_delta(trimmed);
            if brace_depth <= 0 {
                current_service = None;
                brace_depth = 0;
            }
        }
    }

    ids
}

fn count_brace_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|&c| c == '{').count() as i32;
    let closes = line.chars().filter(|&c| c == '}').count() as i32;
    opens - closes
}

fn parse_u32(raw: &str) -> Option<u32> {
    let value = raw.trim().trim_start_matches('=').trim();
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u32>().ok()
    }
}

fn parse_u64(raw: &str) -> Option<u64> {
    let value = raw.trim().trim_start_matches('=').trim();
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u64>().ok()
    }
}

fn to_utf8_path(path: &Path) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|_| anyhow!("path {} is not valid UTF-8", path.display()))
}
