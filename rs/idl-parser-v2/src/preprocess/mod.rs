use crate::error::{Error, Result};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
pub mod fs;
#[cfg(feature = "std")]
pub mod git;

/// The result of loading an IDL source — content and a unique id used for deduplication.
pub struct IdlSource {
    pub content: String,
    /// Unique identifier (e.g. canonical file path or full git:// URL).
    pub id: String,
}

/// Trait for loading IDL content from a path or URL.
///
/// Implement this trait to support custom IDL sources (local files, git, HTTP, etc.).
/// A loader is responsible for three things:
/// - declaring which paths it can handle (`can_load`)
/// - loading the raw IDL content (`load`)
/// - resolving relative include paths relative to a base (`resolve`)
pub trait IdlLoader {
    /// Returns `true` if this loader can handle the given path or URL.
    fn can_load(&self, path: &str) -> bool;

    /// Loads the IDL source at `path`, returning its content and a unique id.
    fn load(&self, path: &str) -> Result<IdlSource>;

    /// Resolves a relative `include_path` against `base_path`.
    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String>;
}

/// Preprocesses the IDL source starting from `path`, resolving `!@include` directives.
///
/// `loaders` are tried in order — the first one where `can_load(path)` returns `true` is used.
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
    let source = find_loader(loaders, path)?.load(path)?;

    if !visited.insert(source.id) {
        return Ok(());
    }

    source
        .content
        .lines()
        .try_fold((), |_, line| -> Result<()> {
            let trimmed = line.trim();

            if trimmed.starts_with("!@include:") {
                let include_path = trimmed
                    .strip_prefix("!@include:")
                    .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
                    .ok_or_else(|| Error::Preprocess("Invalid include directive".to_string()))?;

                let next_path = find_loader(loaders, path)?.resolve(path, include_path)?;
                preprocess_recursive(&next_path, loaders, visited, out)?;

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

fn find_loader<'a>(loaders: &[&'a dyn IdlLoader], path: &str) -> Result<&'a dyn IdlLoader> {
    for &loader in loaders {
        if loader.can_load(path) {
            return Ok(loader);
        }
    }
    Err(Error::Preprocess(alloc::format!(
        "No loader can handle path: {path}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::format;

    struct MapLoader(BTreeMap<String, String>);

    impl IdlLoader for MapLoader {
        fn can_load(&self, _path: &str) -> bool {
            true
        }

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
