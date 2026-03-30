use crate::error::{Error, Result};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
pub mod fs;
#[cfg(feature = "std")]
pub mod git;

/// The result of loading an IDL source — content and a unique id used for deduplication.
#[derive(Debug)]
pub struct IdlSource {
    pub content: String,
    /// Unique identifier (e.g. canonical file path or full git:// URL).
    pub id: String,
}

/// Trait for loading IDL content from a path or URL.
///
/// Implement this trait to support custom IDL sources (local files, git, HTTP, etc.).
/// A loader is responsible for three things:
/// - loading the raw IDL content (`load`)
/// - resolving relative include paths relative to a base (`resolve`)
pub trait IdlLoader {
    /// Loads the IDL source at `path`, returning its content and a unique id.
    fn load(&self, path: &str) -> Result<IdlSource>;

    /// Resolves a relative `include_path` against `base_path`.
    ///
    /// Returns `None` when this loader does not handle `base_path`.
    fn resolve(&self, base_path: &str, include_path: &str) -> Option<String>;
}

/// Preprocesses the IDL source starting from `path`, resolving `!@include` directives.
///
/// `loaders` are tried in order — the first one that resolves the path is used.
/// Each file (identified by `IdlSource::id`) is included at most once.
pub fn preprocess(path: &str, loaders: &[&dyn IdlLoader]) -> Result<String> {
    let mut visited = BTreeSet::new();
    let mut result = String::new();
    preprocess_recursive(path, loaders, &mut visited, &mut result)?;
    Ok(result)
}

fn preprocess_recursive(
    path: &str,
    loaders: &[&dyn IdlLoader],
    visited: &mut BTreeSet<String>,
    out: &mut String,
) -> Result<()> {
    let loader = loaders
        .iter()
        .find(|loader| loader.resolve(path, path).is_some())
        .ok_or_else(|| Error::Preprocess(alloc::format!("No loader can handle path: {path}")))?;

    let source = loader.load(path)?;

    if !visited.insert(source.id) {
        return Ok(());
    }

    for line in source.content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("!@include:") {
            let include_path = rest.trim().trim_matches(|c| c == '"' || c == '\'');

            if include_path.is_empty() {
                return Err(Error::Preprocess("Invalid include directive".to_string()));
            }

            let next_path = loaders
                .iter()
                .filter_map(|loader| loader.resolve(path, include_path))
                .next()
                .ok_or_else(|| {
                    Error::Preprocess(alloc::format!(
                        "No loader can resolve include '{include_path}' from: {path}"
                    ))
                })?;
            preprocess_recursive(&next_path, loaders, visited, out)?;

            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::format;

    struct MapLoader(BTreeMap<String, String>);

    impl IdlLoader for MapLoader {
        fn load(&self, path: &str) -> Result<IdlSource> {
            let content = self
                .0
                .get(path)
                .cloned()
                .ok_or_else(|| Error::Preprocess(format!("File not found: {path}")))?;
            Ok(IdlSource {
                content,
                id: path.to_string(),
            })
        }

        fn resolve(&self, base_path: &str, include_path: &str) -> Option<String> {
            if let Some(pos) = base_path.rfind('/') {
                Some(format!("{}{}", &base_path[..pos + 1], include_path))
            } else {
                Some(String::from(include_path))
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
        let result = preprocess("main.idl", &[&loader]).unwrap();
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
        let result = preprocess("main.idl", &[&loader]).unwrap();

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
        let result = preprocess("main.idl", &[&loader]).unwrap();

        let doc = crate::parse_idl(&result).expect("Failed to parse preprocessed IDL");

        assert_eq!(doc.globals.len(), 3);
        assert_eq!(doc.services.len(), 2);
        assert!(doc.program.is_some());
    }
}
