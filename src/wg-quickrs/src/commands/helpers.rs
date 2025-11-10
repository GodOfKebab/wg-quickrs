use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{rng, RngCore};
use wg_quickrs_lib::types::network::Script;
use wg_quickrs_lib::validation::error::ValidationResult;
use wg_quickrs_lib::validation::network::parse_and_validate_peer_script;

/// Format step string with padding if single-digit
pub fn step_str<const TOTAL: u8>(step: u8) -> String {
    if step < 10 {
        format!("\t[ {}/{}]", step, TOTAL)
    } else {
        format!("\t[{}/{}]", step, TOTAL)
    }
}

/// Handle boolean options
pub fn get_bool(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_value: Option<bool>,
    cli_option: &str,
    description: &str,
    default: bool,
) -> bool {
    if let Some(v) = cli_value {
        println!(
            "{} {} is {} from CLI option '{}'",
            step_str, description, if v { "enabled" } else { "disabled" }, cli_option
        );
        return v;
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", cli_option);
    }

    dialoguer::Confirm::new()
        .with_prompt(format!("{} {} (CLI option '{}')?", step_str, description, cli_option))
        .default(default)
        .interact()
        .unwrap()
}

/// Handle other options
pub fn get_value<T, P>(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_value: Option<String>,
    cli_option: &str,
    description: &str,
    default: Option<String>,
    parse_and_validate_fn: P,
) -> T
where
    P: Fn(&str) -> ValidationResult<T>,
{
    if let Some(v) = cli_value {
        println!("{} Using {} from CLI option '{}': {}", step_str, description, cli_option, v);
        return parse_and_validate_fn(&v).unwrap_or_else(|e| panic!("Error: {}", e));
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", cli_option);
    }

    prompt(
        &format!("{} {} (CLI option '{}')", step_str, description, cli_option),
        default,
        parse_and_validate_fn,
    )
}

/// Helper to prompt a value with optional default and checks
pub fn prompt<T, F>(msg: &str, default: Option<String>, parse_and_validate_fn: F) -> T
where
    F: Fn(&str) -> ValidationResult<T>,
{
    loop {
        let mut input = dialoguer::Input::new().with_prompt(msg);
        if let Some(d) = &default {
            input = input.default(d.clone());
        }

        match input.interact_text() {
            Ok(value) => match parse_and_validate_fn(&value) {
                Ok(r) => return r,
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(_) => eprintln!("ERROR: Error reading input, please try again."),
        }
    }
}

/// Helper to prompt for multiple scripts
pub fn get_scripts(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_enabled: Option<bool>,
    cli_lines: Vec<String>,
    enabled_flag: &str,
    line_flag: &str,
    enabled_help: &str,
    _line_help: &str,
) -> Vec<Script> {
    let mut scripts = Vec::new();

    // Check if scripts are enabled at all
    let scripts_enabled = if let Some(v) = cli_enabled {
        println!(
            "{} {} is {} from CLI option '{}'",
            step_str, enabled_help, if v { "enabled" } else { "disabled" }, enabled_flag
        );
        v
    } else if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", enabled_flag);
    } else {
        dialoguer::Confirm::new()
            .with_prompt(format!("{} {} (CLI option '{}')?", step_str, enabled_help, line_flag))
            .default(false)
            .interact()
            .unwrap()
    };

    if !scripts_enabled {
        return scripts;
    }

    // If CLI lines were provided, add them all and return
    if !cli_lines.is_empty() {
        println!("{} Using {} script line(s) from CLI option '{}'", step_str, cli_lines.len(), line_flag);
        for line in cli_lines {
            let validated_script = parse_and_validate_peer_script(&line)
                .unwrap_or_else(|e| panic!("Error: {}", e));
            scripts.push(Script {
                enabled: true,
                script: validated_script,
            });
        }
        return scripts;
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", line_flag);
    }

    // Prompt for scripts in a loop
    loop {
        let script_line = prompt(
            &format!("\t{} Enter script line (CLI option '{}')", step_str, line_flag),
            None,
            parse_and_validate_peer_script,
        );

        scripts.push(Script {
            enabled: true,
            script: script_line,
        });

        let add_more = dialoguer::Confirm::new()
            .with_prompt(format!("\t{} Add another script?", step_str))
            .default(false)
            .interact()
            .unwrap();

        if !add_more {
            break;
        }
    }

    scripts
}

pub(crate) fn calculate_password_hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = SaltString::encode_b64(&sbytes)?;

    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password.as_ref(), &salt)?.to_string())
}
