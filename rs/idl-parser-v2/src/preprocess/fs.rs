use super::{IdlLoader, IdlSource};
use crate::error::{Error, Result};
use alloc::format;
use alloc::string::{String, ToString};
use std::fs;
use std::path::Path;

/// Loads IDL files from the local file system.
pub struct FsLoader;

impl IdlLoader for FsLoader {
    fn can_load(&self, path: &str) -> bool {
        !path.starts_with("git://")
    }

    fn load(&self, path: &str) -> Result<IdlSource> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Preprocess(format!("Failed to read IDL file at '{path}': {e}")))?;
        let id = Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        Ok(IdlSource { content, id })
    }

    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String> {
        if include_path.contains("://") || include_path.starts_with('/') {
            return Ok(include_path.to_string());
        }
        let base = Path::new(base_path);
        let parent = base.parent().unwrap_or(Path::new("."));
        let resolved = parent.join(include_path);
        Ok(resolved.to_string_lossy().into_owned())
    }
}
