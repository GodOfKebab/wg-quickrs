// build.rs: create macros to embed the version info and build time into the executable
use chrono::SecondsFormat;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml::Value as TomlValue;

fn main() {
    // Generate version macros
    let wg_quickrs_version = fs::read_to_string("../wg-quickrs/Cargo.toml")
        .ok()
        .and_then(|content| {
            toml::from_str::<TomlValue>(&content).ok().and_then(|toml| {
                toml.get("package")?
                    .get("version")?
                    .as_str()
                    .map(String::from)
            })
        })
        .unwrap_or_else(|| "unknown".to_string());

    // get branch name
    let git_branch_name = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get short commit SHA (like GitHub)
    let git_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let content = format!(
        r#"
#[macro_export]
macro_rules! wg_quickrs_version {{
    () => {{
        "{wg_quickrs_version}"
    }};
}}

#[macro_export]
macro_rules! build_git_branch_name {{
    () => {{
        "{git_branch_name}"
    }};
}}

#[macro_export]
macro_rules! build_git_commit {{
    () => {{
        "{git_commit}"
    }};
}}

#[macro_export]
macro_rules! build_timestamp {{
    () => {{
        "{timestamp}"
    }};
}}

#[macro_export]
macro_rules! full_version {{
    () => {{
        "version: v{wg_quickrs_version} | build: {git_branch_name}#{git_commit}@{timestamp}"
    }};
}}
"#
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_macro.rs");
    fs::write(dest_path, content).expect("Could not write version macro");
}
