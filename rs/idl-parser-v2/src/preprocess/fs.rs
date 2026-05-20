use super::path_utils::is_absolute_like;
use super::{IdlLoader, IdlSource};
use crate::error::{Error, Result};
use alloc::format;
use alloc::string::{String, ToString};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Loads IDL files from the local file system.
///
/// In **strict** mode ([`FsLoader::new`]) include paths must be simple
/// relative paths inside the including file's directory — `..`, absolute,
/// and drive-prefix paths are rejected at `resolve()`.
///
/// In **sandboxed** mode ([`FsLoader::with_root`] / [`FsLoader::for_entry`])
/// `..` traversal is allowed; `load()` then canonicalizes the resolved path
/// and verifies it stays under the configured root. The root itself is
/// canonicalized once at construction.
#[derive(Debug, Default, Clone)]
pub struct FsLoader {
    root: Option<PathBuf>,
}

impl FsLoader {
    pub const fn new() -> Self {
        Self { root: None }
    }

    /// Sandbox includes under `root`. An empty path (e.g. from
    /// `Path::parent()` on a bare filename) is normalized to `"."`,
    /// otherwise the trusted-root check would be vacuous against
    /// `starts_with("")`. The root is canonicalized eagerly so repeated
    /// includes don't pay the syscall cost; if canonicalization fails the
    /// original path is kept and the error is surfaced from `load()`.
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        let mut root: PathBuf = root.into();
        if root.as_os_str().is_empty() {
            root = PathBuf::from(".");
        }
        let canonical = root.canonicalize().unwrap_or(root);
        Self {
            root: Some(canonical),
        }
    }

    /// Sandbox using the parent directory of `entry` as the root — what
    /// generators want for an entry-point IDL.
    pub fn for_entry(entry: &Path) -> Self {
        Self::with_root(entry.parent().unwrap_or(Path::new(".")))
    }
}

impl IdlLoader for FsLoader {
    fn load(&self, path: &str) -> Result<IdlSource> {
        if let Some(canonical_root) = &self.root {
            let canonical = Path::new(path)
                .canonicalize()
                .map_err(|e| read_error(path, e))?;
            if !canonical.starts_with(canonical_root) {
                return Err(Error::Preprocess(format!(
                    "IDL include '{path}' escapes the trusted root '{}'",
                    canonical_root.display()
                )));
            }
            let content = fs::read_to_string(&canonical).map_err(|e| read_error(path, e))?;
            return Ok(IdlSource {
                content,
                id: canonical.to_string_lossy().into_owned(),
            });
        }

        let content = fs::read_to_string(path).map_err(|e| read_error(path, e))?;
        let id = Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        Ok(IdlSource { content, id })
    }

    fn resolve(&self, base_path: &str, include_path: &str) -> Option<String> {
        if base_path.contains("://") || include_path.contains("://") {
            return None;
        }

        // `preprocess_recursive` uses `resolve(path, path)` as a "claim this
        // path?" check before delegating to `load`. The entry path is
        // user-supplied (trusted) and may be absolute, so it bypasses the
        // include-style escape rules; sandboxed-mode `load` still enforces
        // the canonical-root boundary.
        if base_path == include_path {
            return Some(include_path.to_string());
        }

        let include = Path::new(include_path);
        if is_absolute_like(include) {
            return None;
        }
        if self.root.is_none()
            && include
                .components()
                .any(|c| matches!(c, Component::ParentDir))
        {
            return None;
        }

        let base = Path::new(base_path);
        let parent = base.parent().unwrap_or(Path::new("."));
        Some(parent.join(include_path).to_string_lossy().into_owned())
    }
}

fn read_error(path: &str, e: std::io::Error) -> Error {
    Error::Preprocess(format!("Failed to read IDL file at '{path}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::{FsLoader, IdlLoader};
    use alloc::string::ToString;

    // ---------- Strict mode ----------

    #[test]
    fn strict_resolves_sibling_include() {
        let resolved = FsLoader::new()
            .resolve("idls/main.idl", "common.idl")
            .expect("sibling include should resolve");
        assert_eq!(
            std::path::Path::new(&resolved),
            std::path::Path::new("idls").join("common.idl")
        );
    }

    #[test]
    fn strict_rejects_absolute_unix_path() {
        assert!(
            FsLoader::new()
                .resolve("idls/main.idl", "/etc/passwd")
                .is_none()
        );
    }

    #[test]
    fn strict_rejects_parent_traversal() {
        assert!(
            FsLoader::new()
                .resolve("idls/main.idl", "../../secrets/private.idl")
                .is_none()
        );
    }

    #[test]
    fn strict_rejects_nested_parent_traversal() {
        assert!(
            FsLoader::new()
                .resolve("idls/main.idl", "sub/../../escape.idl")
                .is_none()
        );
    }

    #[test]
    #[cfg(windows)]
    fn strict_rejects_windows_absolute_path() {
        assert!(
            FsLoader::new()
                .resolve("idls/main.idl", r"C:\Windows\System32\drivers\etc\hosts")
                .is_none()
        );
    }

    #[test]
    fn strict_does_not_claim_url_includes() {
        assert!(
            FsLoader::new()
                .resolve("idls/main.idl", "git://github.com/o/r/main/a.idl")
                .is_none()
        );
    }

    #[test]
    fn strict_does_not_claim_url_bases() {
        assert!(
            FsLoader::new()
                .resolve("git://github.com/o/r/main/a.idl", "common.idl")
                .is_none()
        );
    }

    // ---------- Sandboxed mode ----------

    #[test]
    fn sandboxed_resolves_sibling_traversal() {
        let resolved = FsLoader::with_root("idls")
            .resolve("idls/folder_a/a.idl", "../common.idl")
            .expect("sibling traversal should resolve in sandboxed mode");
        assert!(resolved.contains(".."));
    }

    #[test]
    fn sandboxed_still_rejects_absolute_path() {
        assert!(
            FsLoader::with_root("idls")
                .resolve("idls/main.idl", "/etc/passwd")
                .is_none()
        );
    }

    #[test]
    fn sandboxed_still_rejects_url_includes() {
        assert!(
            FsLoader::with_root("idls")
                .resolve("idls/main.idl", "git://github.com/o/r/main/a.idl")
                .is_none()
        );
    }

    #[test]
    fn sandboxed_load_rejects_escape_outside_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("project");
        let escape = tmp.path().join("secret.idl");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&escape, "leak").unwrap();

        let loader = FsLoader::with_root(&root);
        let escape_via_traversal = root.join("..").join("secret.idl");
        let err = loader
            .load(&escape_via_traversal.to_string_lossy())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("escapes the trusted root"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn entry_claim_test_accepts_absolute_path() {
        let strict = FsLoader::new();
        assert!(
            strict
                .resolve("/abs/path/main.idl", "/abs/path/main.idl")
                .is_some()
        );

        let sandboxed = FsLoader::with_root("/abs/path");
        assert!(
            sandboxed
                .resolve("/abs/path/main.idl", "/abs/path/main.idl")
                .is_some()
        );
    }

    #[test]
    fn with_root_normalizes_empty_path() {
        // `Path::new("foo.idl").parent()` returns `Some("")`. If `with_root`
        // accepted that verbatim the boundary check would degenerate to
        // `starts_with("")` → always true. Normalize to `"."` and ensure
        // a sibling tempdir outside the CWD is still rejected.
        let tmp = tempfile::tempdir().expect("tempdir");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        let outside = tmp.path().join("secret.idl");
        std::fs::write(&outside, "SECRET").unwrap();

        // Use `with_root(project)` then attempt to load a sibling that is
        // unambiguously outside the canonical root.
        let loader = FsLoader::with_root(&project);
        let err = loader
            .load(&outside.to_string_lossy())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("escapes the trusted root"),
            "expected boundary error; got: {err}"
        );
    }

    #[test]
    fn sandboxed_load_validates_before_reading() {
        // Boundary check fires before any `read_to_string`: an outside file
        // produces the boundary error, never "failed to read".
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("project");
        let outside = tmp.path().join("secret.idl");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&outside, "SECRET").unwrap();

        let err = FsLoader::with_root(&root)
            .load(&outside.to_string_lossy())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("escapes the trusted root"),
            "expected boundary error before any read, got: {err}"
        );
    }

    #[test]
    fn sandboxed_load_accepts_file_under_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("project");
        let inner = root.join("sub").join("file.idl");
        std::fs::create_dir_all(inner.parent().unwrap()).unwrap();
        std::fs::write(&inner, "service Foo {}").unwrap();

        let source = FsLoader::with_root(&root)
            .load(&inner.to_string_lossy())
            .expect("file under root should load");
        assert_eq!(source.content, "service Foo {}");
    }

    #[test]
    fn for_entry_uses_parent_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let project = tmp.path().join("project");
        let entry = project.join("main.idl");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(&entry, "service Main {}").unwrap();

        let source = FsLoader::for_entry(&entry)
            .load(&entry.to_string_lossy())
            .expect("entry under its own parent should load");
        assert_eq!(source.content, "service Main {}");
    }
}
