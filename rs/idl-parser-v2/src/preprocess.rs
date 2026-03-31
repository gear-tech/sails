use crate::error::{Error, Result};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

/// Trait for loading IDL content from a path.
pub trait IdlLoader {
    /// Unique identifier for the IDL content (e.g. canonical path or git URL).
    type Id: Ord + Clone;

    /// Load the content and its unique identifier of the IDL file at the given path.
    fn load(&self, path: &str) -> Result<(String, Self::Id)>;

    /// Resolve a relative include path based on the current file path.
    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String>;
}

/// Preprocesses the IDL source, starting from the given path,
/// resolving `!@include` directives recursively.
///
/// Each file (identified by `IdlLoader::Id`) is included at most once.
pub fn preprocess<L: IdlLoader>(path: &str, loader: &L) -> Result<String> {
    let mut visited = BTreeSet::new();
    let mut result = String::new();
    preprocess_recursive(path, loader, &mut visited, &mut result)?;
    Ok(result)
}

fn preprocess_recursive<L: IdlLoader>(
    path: &str,
    loader: &L,
    visited: &mut BTreeSet<L::Id>,
    out: &mut String,
) -> Result<()> {
    let (src, unique_id) = loader.load(path)?;

    if !visited.insert(unique_id) {
        // If already visited, we just return to prevent duplication/cycle
        return Ok(());
    }

    src.lines().try_fold((), |_, line| -> Result<()> {
        let trimmed = line.trim();

        if trimmed.starts_with("!@include:") {
            let next_path = trimmed
                .strip_prefix("!@include:")
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
                .ok_or_else(|| Error::Preprocess("Invalid include directive".to_string()))
                .and_then(|include_path| loader.resolve(path, include_path))?;

            preprocess_recursive(&next_path, loader, visited, out)?;

            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
        Ok(())
    })?;

    Ok(())
}

#[cfg(feature = "std")]
pub mod fs {
    use super::IdlLoader;
    use crate::error::{Error, Result};
    use alloc::format;
    use alloc::string::{String, ToString};
    use std::fs;
    use std::path::Path;

    /// A simple loader that reads IDL files from the local file system.
    pub struct FsLoader;

    impl IdlLoader for FsLoader {
        type Id = String;

        fn load(&self, path: &str) -> Result<(String, Self::Id)> {
            let content = fs::read_to_string(path).map_err(|e| {
                Error::Preprocess(format!("Failed to read IDL file at '{}': {}", path, e))
            })?;
            let id = Path::new(path)
                .canonicalize()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string());
            Ok((content, id))
        }

        fn resolve(&self, base_path: &str, include_path: &str) -> Result<String> {
            let base = Path::new(base_path);
            let parent = base.parent().unwrap_or(Path::new("."));
            let resolved = parent.join(include_path);
            Ok(resolved.to_string_lossy().into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::format;

    struct MapLoader(BTreeMap<String, String>);

    impl IdlLoader for MapLoader {
        type Id = String;

        fn load(&self, path: &str) -> Result<(String, Self::Id)> {
            let content = self
                .0
                .get(path)
                .cloned()
                .ok_or_else(|| Error::Preprocess(format!("File not found: {path}")))?;

            Ok((content, path.to_string()))
        }

        fn resolve(&self, base_path: &str, include_path: &str) -> Result<String> {
            if let Some(pos) = base_path.rfind('/') {
                Ok(format!("{}{}", &base_path[..pos + 1], include_path))
            } else {
                Ok(String::from(include_path))
            }
        }
    }

    #[test]
    fn test_preprocess_recursive() {
        let mut files = BTreeMap::new();
        files.insert("leaf.idl".into(), "service Leaf {}".into());
        files.insert(
            "middle.idl".into(),
            "!@include: leaf.idl\nservice Middle {}".into(),
        );
        files.insert(
            "main.idl".into(),
            "!@include: middle.idl\nservice Main {}".into(),
        );

        let loader = MapLoader(files);
        let result = preprocess("main.idl", &loader).unwrap();
        assert!(result.contains("service Leaf"));
        assert!(result.contains("service Middle"));
        assert!(result.contains("service Main"));
    }

    #[test]
    fn test_preprocess_duplicate_prevented() {
        let mut files = BTreeMap::new();
        files.insert("common.idl".into(), "struct Common {}".into());
        files.insert("a.idl".into(), "!@include: common.idl\nservice A {}".into());
        files.insert("b.idl".into(), "!@include: common.idl\nservice B {}".into());
        files.insert(
            "main.idl".into(),
            "!@include: a.idl\n!@include: b.idl".into(),
        );

        let loader = MapLoader(files);
        let result = preprocess("main.idl", &loader).unwrap();

        // Count occurrences of "struct Common"
        let count = result.matches("struct Common").count();
        assert_eq!(count, 1); // Should be included only once
    }

    #[test]
    fn test_preprocess_complex_includes() {
        let mut files = BTreeMap::new();
        files.insert(
            "common.idl".into(),
            r#"!@sails: 0.1.0
            !@author: gear

            service CommonSvc {
                types {
                    struct Common {
                        id: u64,
                    }
                }
            }"#
            .into(),
        );
        files.insert(
            "service_a.idl".into(),
            r#"!@include: common.idl

            service ServiceA {
                functions {
                    Do(c: u64);
                }
            }"#
            .into(),
        );
        files.insert(
            "main.idl".into(),
            r#"!@sails: 0.1.0
            !@include: service_a.idl

            program Main {
                services {
                    ServiceA: ServiceA,
                }
            }"#
            .into(),
        );

        let loader = MapLoader(files);
        let result = preprocess("main.idl", &loader).unwrap();

        let doc = crate::parse_idl(&result).expect("Failed to parse preprocessed IDL");

        assert_eq!(doc.globals.len(), 3);
        assert_eq!(doc.services.len(), 2);
        assert!(doc.program.is_some());
    }
}
