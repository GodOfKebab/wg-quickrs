use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use ipnet::Ipv4Net;
use rand::{rng, RngCore};
use std::net::Ipv4Addr;
use wg_quickrs_lib::types::network::{AllowedIPs, Script};
use wg_quickrs_lib::validation::error::ValidationResult;
use wg_quickrs_lib::validation::network::parse_and_validate_peer_script;

/// Format step string with padding if single-digit
pub fn make_step_formatter(total: usize) -> impl Fn(usize) -> String {
    move |step: usize| {
        if step < 10 && total >= 10 {
            format!("\t[ {}/{}]", step, total)
        } else {
            format!("\t[{}/{}]", step, total)
        }
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
#[allow(clippy::too_many_arguments)]
pub fn get_scripts(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_enabled: Option<bool>,
    cli_lines: Vec<String>,
    enabled_flag: &str,
    line_flag: &str,
    enabled_help: &str,
    indentation: &str,
) -> Vec<Script> {
    let mut scripts = Vec::new();

    // Check if scripts are enabled at all
    let scripts_enabled = if let Some(v) = cli_enabled {
        println!(
            "{} {}{} is {} from CLI option '{}'",
            step_str, indentation, enabled_help, if v { "enabled" } else { "disabled" }, enabled_flag
        );
        v
    } else if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", enabled_flag);
    } else {
        dialoguer::Confirm::new()
            .with_prompt(format!("{} {}{} (CLI option '{}')?", step_str, indentation, enabled_help, line_flag))
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
            &format!("{} \t{}Enter script line (CLI option '{}')", step_str, indentation, line_flag),
            None,
            parse_and_validate_peer_script,
        );

        scripts.push(Script {
            enabled: true,
            script: script_line,
        });

        let add_more = dialoguer::Confirm::new()
            .with_prompt(format!("{} {}Add another script?", step_str, indentation))
            .default(false)
            .interact()
            .unwrap();

        if !add_more {
            break;
        }
    }

    scripts
}

/// Helper to prompt for multiple DNS addresses
#[allow(clippy::too_many_arguments)]
pub fn get_dns_addresses(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_enabled: Option<bool>,
    cli_addresses: Vec<Ipv4Addr>,
    enabled_flag: &str,
    addresses_flag: &str,
    enabled_help: &str,
    addresses_help: &str,
) -> Vec<Ipv4Addr> {
    get_dns_addresses_with_defaults(
        cli_no_prompt,
        step_str,
        cli_enabled,
        cli_addresses,
        enabled_flag,
        addresses_flag,
        enabled_help,
        addresses_help,
        vec!["1.1.1.1".parse().unwrap()],
    )
}

/// Helper to prompt for multiple DNS addresses with default values
#[allow(clippy::too_many_arguments)]
pub fn get_dns_addresses_with_defaults(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_enabled: Option<bool>,
    cli_addresses: Vec<Ipv4Addr>,
    enabled_flag: &str,
    addresses_flag: &str,
    enabled_help: &str,
    addresses_help: &str,
    default_addresses: Vec<Ipv4Addr>,
) -> Vec<Ipv4Addr> {
    let mut addresses = Vec::new();

    // Check if DNS is enabled at all
    let dns_enabled = if let Some(v) = cli_enabled {
        println!(
            "{} {} is {} from CLI option '{}'",
            step_str, enabled_help, if v { "enabled" } else { "disabled" }, enabled_flag
        );
        v
    } else if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", enabled_flag);
    } else {
        dialoguer::Confirm::new()
            .with_prompt(format!("{} {} (CLI option '{}')?", step_str, enabled_help, addresses_flag))
            .default(true)
            .interact()
            .unwrap()
    };

    if !dns_enabled {
        return addresses;
    }

    // If CLI addresses were provided, return them directly
    if !cli_addresses.is_empty() {
        let dns_string = cli_addresses.iter()
            .map(|addr| addr.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("{} Using {} DNS address(es) from CLI option '{}': {}",
            step_str, cli_addresses.len(), addresses_flag, dns_string);
        return cli_addresses;
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", addresses_flag);
    }

    // Prompt for DNS addresses in a loop
    let mut dns_address_counter = 0;
    loop {
        let dns_address: Ipv4Addr = prompt(
            &format!("\t{} {} (CLI option '{}')", step_str, addresses_help, addresses_flag),
            if dns_address_counter < default_addresses.len() { Some(default_addresses[dns_address_counter].to_string()) } else { None },
            |s: &str| s.trim().parse().map_err(|_| wg_quickrs_lib::validation::error::ValidationError::NotIPv4Address()),
        );
        addresses.push(dns_address);
        dns_address_counter += 1;

        let add_more = dialoguer::Confirm::new()
            .with_prompt(format!("\t{} Add another DNS address?", step_str))
            .default(dns_address_counter < default_addresses.len())
            .interact()
            .unwrap();

        if !add_more {
            break;
        }
    }

    addresses
}

/// Helper to prompt for multiple allowed IPs (CIDR blocks)
pub fn get_allowed_ips(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_allowed_ips: Vec<Ipv4Net>,
    flag: &str,
    help: &str,
    default_allowed_ips: Vec<Ipv4Net>,
) -> AllowedIPs {
    let mut allowed_ips = Vec::new();

    // If CLI allowed IPs were provided, return them directly
    if !cli_allowed_ips.is_empty() {
        let ips_string = cli_allowed_ips.iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("{} Using {} allowed IPs from CLI option '{}': {}",
            step_str, cli_allowed_ips.len(), flag, ips_string);
        return cli_allowed_ips;
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", flag);
    }

    // Prompt for allowed IPs in a loop
    let mut allowed_ip_counter = 0;
    loop {
        let allowed_ip: Ipv4Net = prompt(
            &format!("\t{} {} (CLI option '{}')", step_str, help, flag),
            if allowed_ip_counter < default_allowed_ips.len() {
                Some(default_allowed_ips[allowed_ip_counter].to_string())
            } else {
                None
            },
            |s: &str| s.trim().parse().map_err(|_| wg_quickrs_lib::validation::error::ValidationError::InvalidAllowedIPs()),
        );
        allowed_ips.push(allowed_ip);
        allowed_ip_counter += 1;

        let add_more = dialoguer::Confirm::new()
            .with_prompt(format!("\t{} Add another allowed IPs?", step_str))
            .default(allowed_ip_counter < default_allowed_ips.len())
            .interact()
            .unwrap();

        if !add_more {
            break;
        }
    }

    allowed_ips
}

pub(crate) fn calculate_password_hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = SaltString::encode_b64(&sbytes)?;

    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password.as_ref(), &salt)?.to_string())
}
