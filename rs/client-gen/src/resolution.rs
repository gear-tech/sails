use anyhow::Result;
use sails_idl_parser_v2::preprocess::{self, IdlLoader};
use std::fs;
use std::path::{Path, PathBuf};

struct FsLoader {
    base_dir: PathBuf,
}

impl IdlLoader for FsLoader {
    fn load(&self, path: &str) -> Result<(String, u64), String> {
        let full_path = self.base_dir.join(path);

        let content = fs::read_to_string(&full_path).map_err(|e| {
            format!(
                "Failed to read include '{}' (base_dir: '{}'): {}",
                path,
                self.base_dir.display(),
                e
            )
        })?;

        let hash = xxhash_rust::xxh3::xxh3_64(content.as_bytes());
        Ok((content, hash))
    }

    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String, String> {
        let base_path = Path::new(base_path);
        let parent = base_path.parent().unwrap_or(Path::new(""));
        let resolved = parent.join(include_path);
        Ok(resolved.to_string_lossy().into_owned())
    }
}

pub fn resolve_idl_from_path(path: &Path) -> Result<String> {
    let parent_dir = path.parent().unwrap_or(Path::new("."));
    let loader = FsLoader {
        base_dir: parent_dir.to_path_buf(),
    };

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid IDL path: {}", path.display()))?;

    preprocess::preprocess(filename, &loader).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_resolve_idl_from_path() {
        let path = Path::new("tests/idls/recursive_main.idl");
        let result = resolve_idl_from_path(path).unwrap();

        assert!(result.contains("service Leaf"));
        assert!(result.contains("service Middle"));
        assert!(result.contains("service Main"));
    }

    #[test]
    fn test_resolve_nested_idl() {
        let path = Path::new("tests/idls/nested/main.idl");
        let result = resolve_idl_from_path(path).expect("Failed to resolve nested IDL");

        assert!(result.contains("service A"));
        assert!(result.contains("service B"));
        assert!(result.contains("service Main"));

        // Check that Common struct was included exactly ONCE
        let common_count = result.matches("struct Common").count();
        assert_eq!(
            common_count, 1,
            "struct Common should be included only once, but found {}",
            common_count
        );
    }
}
