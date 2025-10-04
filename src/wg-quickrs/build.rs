// build.rs: create macros to embed the version info and build time into the executable
// and generate bash completion script
use chrono::SecondsFormat;
use clap::{CommandFactory, Parser};
use clap_complete::generate_to;
use wg_quickrs_cli::{Cli, InitOptions};
use std::fs;
use std::path::Path;
use std::process::Command;
use toml::Value as TomlValue;

fn main() {
    // Generate version macros
    let wg_quickrs_version = fs::read_to_string("Cargo.toml")
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

    // flag/help generator for cli
    #[derive(Parser, Debug)]
    struct InitOptionsWrapper {
        #[command(flatten)]
        opts: InitOptions,
    }
    let cmd = InitOptionsWrapper::command();

    let flags = cmd
        .get_arguments()
        .filter_map(|arg| arg.get_long().map(|long| format!("--{}", long)))
        .collect::<Vec<String>>();

    let helps = cmd
        .get_arguments()
        .map(|arg| arg.get_long_help().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("init_options_generated.rs");

    let content = format!(
        r#"
pub const INIT_FLAGS: &[&str] = &{flags:?};
pub const INIT_HELPS: &[&str] = &{helps:?};
        "#,
        flags = flags,
        helps = helps,
    );

    fs::write(dest_path, content).expect("Could not write init_options_generated.rs");

    // Generate bash completion script
    let cli = Cli::command();
    let mut cmd = cli;

    // Create completions directory in OUT_DIR
    let completions_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("completions");
    fs::create_dir_all(&completions_dir).expect("Could not create completions directory");

    // Generate bash completion script
    let shells: &[clap_complete::Shell] = &[
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
        clap_complete::Shell::Elvish,
    ];
    for &shell in shells {
        let _completion_file_path = generate_to(shell, &mut cmd, "wg-quickrs", &completions_dir)
            .expect("Failed to generate bash completion script");
        // .../target/release/completions/...
        // println!("cargo:warning=Generated {} completion script at: {:?}", shell, _completion_file_path);
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../web/package.json");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
