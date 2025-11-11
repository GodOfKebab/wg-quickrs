use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Empty command")]
    Empty(),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed: {0}")]
    Failed(String),
}
pub type ShellResult<T> = Result<T, ShellError>;

pub fn shell_cmd(args: &[&str]) -> ShellResult<Output> {
    if args.is_empty() {
        return Err(ShellError::Empty());
    }

    log::debug!("[+] {}", args.join(" "));

    let output = Command::new(args[0])
        .args(&args[1..])
        .output()?;
    if !output.stderr.is_empty() {
        log::debug!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        log::warn!("[+] {}", args.join(" "));
        log::warn!("{}", String::from_utf8_lossy(&output.stdout));
        return Err(ShellError::Failed(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    Ok(output)
}
