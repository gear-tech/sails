use super::{IdlLoader, IdlSource};
use crate::error::{Error, Result};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::fs;

/// Loads IDL files from git repositories.
///
/// URL format: `git://{host}/{owner}/{repo}/{branch}/{file_path}`
///
/// Example: `!@include: git://github.com/gear-tech/sails/master/examples/demo/client/demo_client.idl`
pub struct GitLoader;

impl IdlLoader for GitLoader {
    fn can_load(&self, path: &str) -> bool {
        path.starts_with("git://")
    }

    fn load(&self, path: &str) -> Result<IdlSource> {
        let content = git_fetch(path)?;
        Ok(IdlSource {
            content,
            id: path.to_string(),
        })
    }

    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String> {
        let base_dir = base_path
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or(base_path);
        let joined = format!("{base_dir}/{include_path}");
        Ok(normalize_git_path(&joined))
    }
}

/// Downloads a file from a git repository and returns its content.
///
/// URL format: `git://{host}/{owner}/{repo}/{branch}/{file_path}`
fn git_fetch(url: &str) -> Result<String> {
    let rest = url
        .strip_prefix("git://")
        .ok_or_else(|| Error::Preprocess(format!("Invalid git URL: {url}")))?;

    let parts: Vec<&str> = rest.splitn(5, '/').collect();
    if parts.len() < 5 {
        return Err(Error::Preprocess(format!(
            "Git URL must be git://host/owner/repo/branch/file_path, got: {url}"
        )));
    }

    let repo_url = format!("https://{}/{}/{}", parts[0], parts[1], parts[2]);
    let branch = parts[3];
    let file_path = parts[4];

    let tmp = tempfile::tempdir()
        .map_err(|e| Error::Preprocess(format!("Failed to create temp dir: {e}")))?;

    git_download::repo(&repo_url)
        .branch_name(branch)
        .add_file(file_path, "out.idl")
        .out_dir(tmp.path())
        .exec()
        .map_err(|e| Error::Preprocess(format!("Failed to fetch git IDL '{url}': {e}")))?;

    fs::read_to_string(tmp.path().join("out.idl"))
        .map_err(|e| Error::Preprocess(format!("Failed to read fetched IDL '{url}': {e}")))
}

/// Resolves a relative include path against a base git:// URL.
///
/// Example: base `git://github.com/org/repo/main/a/b/c.idl`, relative `../d.idl`
/// → `git://github.com/org/repo/main/a/d.idl`
fn normalize_git_path(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    let mut segments: Vec<&str> = Vec::new();
    for seg in rest.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    format!("{}://{}", scheme, segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_resolve_sibling() {
        let loader = GitLoader;
        let result = loader
            .resolve("git://github.com/org/repo/main/a/b/c.idl", "d.idl")
            .unwrap();
        assert_eq!(result, "git://github.com/org/repo/main/a/b/d.idl");
    }

    #[test]
    fn test_git_resolve_parent_dir() {
        let loader = GitLoader;
        let result = loader
            .resolve("git://github.com/org/repo/main/a/b/c.idl", "../d.idl")
            .unwrap();
        assert_eq!(result, "git://github.com/org/repo/main/a/d.idl");
    }

    #[test]
    fn test_git_resolve_two_levels_up() {
        let loader = GitLoader;
        let result = loader
            .resolve(
                "git://github.com/org/repo/main/a/b/c.idl",
                "../../types.idl",
            )
            .unwrap();
        assert_eq!(result, "git://github.com/org/repo/main/types.idl");
    }

    #[test]
    fn test_normalize_no_dots() {
        let url = "git://github.com/org/repo/main/idls/common.idl";
        assert_eq!(normalize_git_path(url), url);
    }

    #[test]
    fn test_normalize_with_dotdot() {
        let url = "git://github.com/org/repo/main/a/../b/c.idl";
        assert_eq!(
            normalize_git_path(url),
            "git://github.com/org/repo/main/b/c.idl"
        );
    }
}
