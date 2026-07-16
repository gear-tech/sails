use anyhow::{Context, Result};
use std::{
    env, io,
    path::Path,
    process::{Command, Stdio},
    process::{ExitStatus, Output},
};

pub trait ExitStatusExt: Sized {
    fn exit_result(self) -> io::Result<()>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_result(self) -> io::Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(io::Error::from(io::ErrorKind::Other))
        }
    }
}

pub trait OutputExt: Sized {
    fn exit_result(self) -> io::Result<Self>;
}

impl OutputExt for Output {
    fn exit_result(self) -> io::Result<Self> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(io::Error::from(io::ErrorKind::Other))
        }
    }
}

pub fn git_show_current_branch<P: AsRef<Path>>(target_dir: P) -> Result<String> {
    let git_command = git_command();
    let mut cmd = Command::new(git_command);
    cmd.stdout(Stdio::piped())
        .arg("-C")
        .arg(target_dir.as_ref())
        .arg("branch")
        .arg("--show-current");

    let output = cmd
        .output()?
        .exit_result()
        .context("failed to get current git branch")?;
    let git_branch_name = String::from_utf8(output.stdout)?;

    Ok(git_branch_name.trim().into())
}

pub fn git_command() -> String {
    env::var("GIT").unwrap_or("git".into())
}
