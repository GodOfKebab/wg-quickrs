// build.rs: generate bash/zsh/... completion scripts and CLI documentation
use clap::{CommandFactory, Parser};
use clap_complete::generate_to;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Generate constants for init options
    {
        #[derive(Parser, Debug)]
        struct InitOptionsWrapper {
            #[command(flatten)]
            opts: wg_quickrs_cli::agent::InitOptions,
        }
        let cmd = InitOptionsWrapper::command();

        let mut const_content = String::new();

        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                if long == "no-prompt" {
                    continue
                }
                let help = arg.get_long_help().unwrap_or_default().to_string().replace('"', "\\\"").replace('\n', "\\n");
                let const_name = long.replace('-', "_").to_uppercase();

                if long.ends_with("line") {
                    const_content.push_str(&format!(
                        r#"
pub const INIT_{const_name}_FLAG: &str = "--{long}";
"#,
                    ));
                } else {

                    const_content.push_str(&format!(
                        r#"
pub const INIT_{const_name}_FLAG: &str = "--{long}";
pub const INIT_{const_name}_HELP: &str = "{help}";
"#,
                    ));
                }
            }
        }

        let dest_path = Path::new(&out_dir).join("init_options_generated.rs");
        fs::write(dest_path, const_content).expect("Could not write init_options_generated.rs");
    }

    // Generate constants for add peer options
    {
        #[derive(Parser, Debug)]
        struct AddPeerOptionsWrapper {
            #[command(flatten)]
            opts: wg_quickrs_cli::config::add::AddPeerOptions,
        }
        let cmd = AddPeerOptionsWrapper::command();

        let mut const_content = String::new();

        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                if long == "no-prompt" {
                    continue
                }
                let help = arg.get_long_help().unwrap_or_default().to_string().replace('"', "\\\"").replace('\n', "\\n");
                let const_name = long.replace('-', "_").to_uppercase();

                if long.ends_with("line") {
                    const_content.push_str(&format!(
                        r#"
pub const ADD_PEER_{const_name}_FLAG: &str = "--{long}";
"#,
                    ));
                } else {

                    const_content.push_str(&format!(
                        r#"
pub const ADD_PEER_{const_name}_FLAG: &str = "--{long}";
pub const ADD_PEER_{const_name}_HELP: &str = "{help}";
"#,
                    ));
                }
            }
        }

        let dest_path = Path::new(&out_dir).join("add_peer_options_generated.rs");
        fs::write(dest_path, const_content).expect("Could not write add_peer_options_generated.rs");
    }

    // Generate constants for add connection options
    {
        #[derive(Parser, Debug)]
        struct AddConnectionOptionsWrapper {
            #[command(flatten)]
            opts: wg_quickrs_cli::config::add::AddConnectionOptions,
        }
        let cmd = AddConnectionOptionsWrapper::command();

        let mut const_content = String::new();

        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                if long == "no-prompt" || long == "first-peer" || long == "second-peer" {
                    continue
                }
                let help = arg.get_long_help().unwrap_or_default().to_string();
                let const_name = long.replace('-', "_").to_uppercase();

                const_content.push_str(&format!(
                    r#"
pub const ADD_CONNECTION_{}_FLAG: &str = "--{}";
pub const ADD_CONNECTION_{}_HELP: &str = "{}";
"#,
                    const_name,
                    long,
                    const_name,
                    help.replace('"', "\\\"").replace('\n', "\\n")
                ));
            }
        }

        let dest_path = Path::new(&out_dir).join("add_connection_options_generated.rs");
        fs::write(dest_path, const_content).expect("Could not write add_connection_options_generated.rs");
    }

    // Generate completion scripts
    let cli = wg_quickrs_cli::Cli::command();
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

    // Generate completion scripts
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

    // Generate markdown documentation
    generate_cli_docs();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

fn generate_cli_docs() {
    let mut cli = wg_quickrs_cli::Cli::command();
    let mut markdown = String::new();

    // 2. Agent commands (with links)
    if let Some(agent_cmd) = cli.find_subcommand_mut("agent") {
        markdown.push_str(&clap_markdown::help_markdown_command_custom(
            agent_cmd,
            &clap_markdown::MarkdownOptions::new().show_footer(false).title("`wg-quickrs agent`".to_string())
        ));
        markdown.push_str("\n---\n\n");
    }

    // 3. Config commands (with links)
    if let Some(config_cmd) = cli.find_subcommand_mut("config") {
        markdown.push_str(&clap_markdown::help_markdown_command_custom(
            config_cmd,
            &clap_markdown::MarkdownOptions::new().title("`wg-quickrs config`".to_string())
        ));
    }

    // Replace "Command Overview" with "Subcommand Overview"
    markdown = markdown.replace("Command Overview", "Subcommand Overview");

    // Prepend # to all lines starting with #
    let mut result = String::new();
    // 1. Main command (no links - just basic info)
    result.push_str(&format!("# `wg-quickrs` ({})\n\n", env!("CARGO_PKG_VERSION")));
    // result.push_str("# `wg-quickrs`\n\n");
    if let Some(about) = cli.get_about() {
        result.push_str(&format!("{}\n\n", about));
    }
    result.push_str(&format!("**Usage:** `{}`\n\n", cli.render_usage()));
    result.push_str("###### **Subcommands:**\n");
    result.push_str("* `agent` — Run agent commands\n");
    result.push_str("* `config` — Edit agent configuration options\n\n");
    result.push_str("###### **Options:**\n\n");
    result.push_str("* `-v`, `--verbose` — Increase verbosity level from Info to Debug\n");
    result.push_str("* `--wg-quickrs-config-folder <WG_QUICKRS_CONFIG_FOLDER>`\n\n");
    result.push_str("**Command Overview:**\n");
    result.push_str("* [`wg-quickrs agent`↴](#wg-quickrs-agent)\n");
    result.push_str("* [`wg-quickrs config`↴](#wg-quickrs-config)\n\n");
    result.push_str("---\n\n");

    let lines: Vec<&str> = markdown.lines().collect();
    for line in lines {
        if line.starts_with("######") {
            result.push_str(line);
            result.push('\n');
        } else if line.starts_with('#') {
            result.push_str(&format!("#{}\n", line));
        } else if line.starts_with("This document contains") {
            // skip the line
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    markdown = result;

    let docs_path = Path::new("../../docs/CLI_REFERENCE.md");
    fs::write(docs_path, markdown).expect("Failed to write markdown documentation");
}
