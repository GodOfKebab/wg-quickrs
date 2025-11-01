// build.rs: generate bash/zsh/... completion scripts
use clap::{CommandFactory, Parser};
use clap_complete::generate_to;
use wg_quickrs_cli::{Cli, InitOptions};
use std::fs;
use std::path::Path;

fn main() {
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

    // Generate completion scripts
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
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
