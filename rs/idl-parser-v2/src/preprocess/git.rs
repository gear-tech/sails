use super::{IdlLoader, IdlSource};
use crate::error::{Error, Result};
use alloc::format;
use alloc::string::{String, ToString};
use std::path::{Component, Path};
use std::process::{Command, Stdio};

/// Loads IDL files from git repositories.
///
/// URL format: `git://{host}/{owner}/{repo}/{branch_or_tag}[:]{file_path}`
///
/// To support branch names with slashes, use `:` to separate the branch from the file path.
///
/// Examples:
/// - `git://github.com/gear-tech/sails/master/examples/demo.idl`
/// - `git://github.com/gear-tech/sails/feat/git-include:examples/demo.idl`
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
        if include_path.contains("://") || include_path.starts_with('/') {
            return Ok(include_path.to_string());
        }
        let pos = base_path.rfind(['/', ':']).unwrap_or(0);
        Ok(format!("{}{}", &base_path[..pos + 1], include_path))
    }
}

/// Downloads a single file from a git repository and returns its content.
fn git_fetch(url: &str) -> Result<String> {
    check_git_available()?;

    let rest = url
        .strip_prefix("git://")
        .ok_or_else(|| Error::Preprocess(format!("Invalid git URL: {url}")))?;

    // Parse host, owner, repo
    let mut parts = rest.splitn(4, '/');
    let host = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Preprocess("Missing host".to_string()))?;
    let owner = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Preprocess("Missing owner".to_string()))?;
    let repo = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Preprocess("Missing repo".to_string()))?;
    let remainder = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Preprocess("Missing branch and file path".to_string()))?;

    let remainder = remainder.strip_prefix("blob/").unwrap_or(remainder);

    // Split remainder into branch (rev) and file path.
    // Use ':' as a primary separator to support branches with slashes.
    // Fallback to the first '/' if ':' is not present.
    let (rev, file_path) = if let Some((r, p)) = remainder.split_once(':') {
        (r, p)
    } else {
        remainder
            .split_once('/')
            .ok_or_else(|| Error::Preprocess(
                "Missing file path. If your branch name contains slashes, use ':' to separate it from the path (e.g., branch/name:path/to/file.idl)".to_string()
            ))?
    };

    if rev.is_empty() {
        return Err(Error::Preprocess("Missing branch/tag".to_string()));
    }
    if file_path.is_empty() {
        return Err(Error::Preprocess("Missing file path".to_string()));
    }

    if Path::new(file_path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(Error::Preprocess(format!(
            "Path traversal detected in URL: {url}"
        )));
    }

    let repo_url = format!("https://{host}/{owner}/{repo}");
    let tmp = tempfile::tempdir()
        .map_err(|e| Error::Preprocess(format!("Failed to create temp dir: {e}")))?;
    let dir = tmp.path();

    run(dir, &["git", "init"])?;
    run(dir, &["git", "sparse-checkout", "init", "--no-cone"])?;
    run(dir, &["git", "sparse-checkout", "set", file_path])?;
    run(dir, &["git", "remote", "add", "origin", &repo_url])?;

    run(
        dir,
        &[
            "git",
            "fetch",
            "--depth",
            "1",
            "--filter=blob:none",
            "origin",
            rev,
        ],
    )?;

    run(dir, &["git", "checkout", "FETCH_HEAD"])?;

    std::fs::read_to_string(dir.join(file_path))
        .map_err(|e| Error::Preprocess(format!("Failed to read fetched IDL '{url}': {e}")))
}

fn check_git_available() -> Result<()> {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| Error::Preprocess("git is not available in PATH".to_string()))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(Error::Preprocess(
                    "git --version returned non-zero exit code".to_string(),
                ))
            }
        })
}

fn run(dir: &Path, args: &[&str]) -> Result<()> {
    let (cmd, rest) = args
        .split_first()
        .ok_or_else(|| Error::Preprocess("Empty command".to_string()))?;

    let output = Command::new(cmd)
        .args(rest)
        .current_dir(dir)
        .output()
        .map_err(|e| Error::Preprocess(format!("Failed to run `{cmd}`: {e}")))?;

    if !output.status.success() {
        return Err(Error::Preprocess(format!(
            "git command `{}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
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
        assert_eq!(result, "git://github.com/org/repo/main/a/b/../d.idl");
    }

    #[test]
    fn test_git_resolve_absolute_fs_path() {
        let loader = GitLoader;
        let result = loader
            .resolve("git://github.com/org/repo/main/a.idl", "/etc/types.idl")
            .unwrap();
        assert_eq!(result, "/etc/types.idl");
    }

    #[test]
    fn test_git_resolve_different_git_url() {
        let loader = GitLoader;
        let other_url = "git://github.com/other/repo/main/types.idl";
        let result = loader
            .resolve("git://github.com/org/repo/main/a.idl", other_url)
            .unwrap();
        assert_eq!(result, other_url);
    }

    #[test]
    fn test_git_url_parsing_errors() {
        let loader = GitLoader;

        let cases = [
            ("git://", "Missing host"),
            ("git://github.com/", "Missing owner"),
            ("git://github.com/gear-tech/", "Missing repo"),
            (
                "git://github.com/gear-tech/sails/",
                "Missing branch and file path",
            ),
            (
                "git://github.com/gear-tech/sails/:path.idl",
                "Missing branch/tag",
            ),
            (
                "git://github.com/gear-tech/sails/master:",
                "Missing file path",
            ),
        ];

        for (url, expected_err) in cases {
            let err = loader.load(url).unwrap_err().to_string();
            assert!(
                err.contains(expected_err),
                "URL '{}' should fail with '{}', but got '{}'",
                url,
                expected_err,
                err
            );
        }
    }

    #[test]
    fn test_git_url_complex_branch() {
        let loader = GitLoader;
        // Test that resolve works with ':'
        let result = loader
            .resolve("git://github.com/org/repo/feat/v1:a.idl", "b.idl")
            .unwrap();
        assert_eq!(result, "git://github.com/org/repo/feat/v1:b.idl");
    }

    #[test]
    fn test_path_traversal_protection() {
        let loader = GitLoader;
        let url = "git://github.com/org/repo/master/../../etc/passwd";
        let err = loader.load(url).unwrap_err().to_string();
        assert!(err.contains("Path traversal detected"));
    }
}
