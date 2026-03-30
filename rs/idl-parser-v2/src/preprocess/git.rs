use super::{IdlLoader, IdlSource};
use crate::error::{Error, Result};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::Path;
use std::process::{Command, Stdio};

/// Loads IDL files from git repositories.
///
/// URL format: `git://{host}/{owner}/{repo}/{branch_or_tag}/{file_path}`
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

/// Downloads a single file from a git repository and returns its content.
///
/// URL format: `git://{host}/{owner}/{repo}/{branch_or_tag}/{file_path}`
fn git_fetch(url: &str) -> Result<String> {
    check_git_available()?;

    let rest = url
        .strip_prefix("git://")
        .ok_or_else(|| Error::Preprocess(format!("Invalid git URL: {url}")))?;

    let parts: Vec<&str> = rest.splitn(5, '/').collect();
    if parts.len() < 5 {
        return Err(Error::Preprocess(format!(
            "Git URL must be git://host/owner/repo/branch_or_tag/file_path, got: {url}"
        )));
    }

    let repo_url = format!("https://{}/{}/{}", parts[0], parts[1], parts[2]);
    let rev = parts[3];
    let file_path = parts[4];
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("out.idl");

    let tmp = tempfile::tempdir()
        .map_err(|e| Error::Preprocess(format!("Failed to create temp dir: {e}")))?;
    let dir = tmp.path();

    run(dir, &["git", "init"])?;
    run(dir, &["git", "remote", "add", "origin", &repo_url])?;
    run(dir, &["git", "config", "core.sparseCheckout", "true"])?;

    std::fs::write(dir.join(".git/info/sparse-checkout"), file_path)
        .map_err(|e| Error::Preprocess(format!("Failed to write sparse-checkout: {e}")))?;

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

    let out_path = dir.join(file_name);
    let actual_path = if out_path.exists() {
        out_path
    } else {
        dir.join(file_path)
    };

    std::fs::read_to_string(&actual_path)
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
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

/// Resolves `..` and `.` segments in a git:// URL.
///
/// Example: `git://github.com/org/repo/main/a/../b/c.idl` → `git://github.com/org/repo/main/b/c.idl`
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
