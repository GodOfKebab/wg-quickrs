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

    let timestamp = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    // Get current git branch
    fn git_info_fn() -> String {
        // get tag name w/ branch name fallback
        let mut git_branch_tag = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Try tag first
        if let Some(tag) = Command::new("git")
            .args(["describe", "--tags", "--exact-match"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
        {
            git_branch_tag = tag;
        }

        // Get short commit SHA (like GitHub)
        let git_commit = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        format!("{}#{}", git_branch_tag, git_commit)
    }
    let git_info = git_info_fn();

    let content = format!(
        r#"
#[macro_export]
macro_rules! wg_quickrs_version {{
    () => {{
        "{wg_quickrs_version}"
    }};
}}

#[macro_export]
macro_rules! build_info {{
    () => {{
        "{git_info}@{timestamp}"
    }};
}}

#[macro_export]
macro_rules! full_version {{
    () => {{
        "version: {wg_quickrs_version} | build: {git_info}@{timestamp}"
    }};
}}
"#
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_macro.rs");
    fs::write(dest_path, content).expect("Could not write version macro");
}
