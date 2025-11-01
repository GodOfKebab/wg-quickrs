use crate::commands::helpers;
use crate::{WG_QUICKRS_CONFIG_FILE, WG_QUICKRS_CONFIG_FOLDER};
use crate::conf;
use dialoguer;
use get_if_addrs::{Interface, get_if_addrs};
use wg_quickrs_cli::InitOptions;
use wg_quickrs_wasm::types::config::*;
use wg_quickrs_wasm::types::network::*;
use wg_quickrs_wasm::helpers::wg_generate_key;
use std::collections::{BTreeMap};
use std::net::{IpAddr};
use std::path::{PathBuf};
use std::{env, fs};
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;
use wg_quickrs_wasm::validation::agent::{parse_and_validate_fw_gateway, parse_and_validate_ipv4_address, parse_and_validate_port, parse_and_validate_tls_file, parse_and_validate_fw_utility};
use wg_quickrs_wasm::validation::error::ValidationResult;
use wg_quickrs_wasm::validation::helpers::firewall_utility_options;
use wg_quickrs_wasm::validation::network::{parse_and_validate_conn_persistent_keepalive_period, parse_and_validate_ipv4_subnet, parse_and_validate_network_name, parse_and_validate_peer_address, parse_and_validate_peer_dns_addresses, parse_and_validate_peer_endpoint, parse_and_validate_peer_icon_src, parse_and_validate_peer_kind, parse_and_validate_peer_mtu_value, parse_and_validate_peer_name, parse_and_validate_peer_script};
use crate::conf::network::get_next_available_address;
use crate::conf::util::ConfUtilError;

include!(concat!(env!("OUT_DIR"), "/init_options_generated.rs"));

#[derive(Error, Debug)]
pub enum InitError {
    #[error("wg-quickrs is already initialized at \"{0}\"")]
    AlreadyInitialized(String),
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    ConfUtil(#[from] ConfUtilError),
}

// Get primary IP of the current machine
pub fn get_interfaces() -> Vec<Interface> {
    get_if_addrs()
        .unwrap()
        .into_iter()
        .filter(|a| !a.is_loopback() && a.ip().is_ipv4())
        .collect()
}

fn find_cert_server(config_folder: &PathBuf, web_address: String) -> (Option<PathBuf>, Option<PathBuf>) {
    let servers_folder = config_folder.join("certs/servers");

    if servers_folder.join(&web_address).join("cert.pem").exists()
        && servers_folder.join(&web_address).join("key.pem").exists()
    {
        return (
            Some(
                servers_folder
                    .join(&web_address)
                    .join("cert.pem")
                    .strip_prefix(config_folder).unwrap()
                    .to_path_buf(),
            ),
            Some(
                servers_folder
                    .join(&web_address)
                    .join("key.pem")
                    .strip_prefix(config_folder).unwrap()
                    .to_path_buf(),
            ),
        );
    }

    let mut candidates: Vec<(PathBuf, PathBuf)> = Vec::new();

    if let Ok(entries) = fs::read_dir(&servers_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let cert = path.join("cert.pem");
                let key = path.join("key.pem");

                if cert.exists()
                    && key.exists()
                    && let (Ok(rel_cert), Ok(rel_key)) = (
                        cert.strip_prefix(config_folder),
                        key.strip_prefix(config_folder),
                    )
                {
                    candidates.push((rel_cert.to_path_buf(), rel_key.to_path_buf()));
                }
            }
        }
    }

    // Sort alphabetically by directory name
    candidates.sort_by(|a, b| {
        a.0.parent()
            .and_then(|p| p.file_name())
            .cmp(&b.0.parent().and_then(|p| p.file_name()))
    });

    if let Some((cert, key)) = candidates.into_iter().next() {
        (
            Some(cert),
            Some(key),
        )
    } else {
        (None, None)
    }
}

/// Format step string with padding if single-digit
fn step_str(step: usize) -> String {
    if step < 10 {
        format!("\t[ {}/28]", step)
    } else {
        format!("\t[{}/28]", step)
    }
}

/// Handle boolean options
fn get_init_bool(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_value: Option<bool>,
    cli_option: &str,
    description: &str,
    default: bool,
) -> bool {
    let step_str = step_str(step);

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
fn get_init_value<T, P>(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_value: Option<String>,
    cli_option: &str,
    description: &str,
    default: Option<String>,
    parse_and_validate_fn: P,
) -> T
where
    P: Fn(&str) -> ValidationResult<T>,
{
    let step_str = step_str(step);

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
fn prompt<T, F>(msg: &str, default: Option<String>, parse_and_validate_fn: F) -> T
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

/// Handle other options
fn get_init_password(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_value: Option<String>,
    cli_option: &str,
    description: &str,
) -> String {
    let step_str = step_str(step);

    if let Some(v) = cli_value {
        println!(
            "{}  Using password for the web server from CLI argument: ***hidden***",
            step_str
        );
        return v;
    }

    if cli_no_prompt == Some(true) {
        panic!("Error: CLI option '{}' is not set", cli_option);
    }

    dialoguer::Password::new()
        .with_prompt(format!("{} {}", step_str, description))
        .interact()
        .unwrap()
}


pub fn initialize_agent(init_opts: &InitOptions) -> Result<(), InitError> {
    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    if file_path.exists() {
        return Err(InitError::AlreadyInitialized(file_path.display().to_string()));
    }
    log::info!("Initializing wg-quickrs...");

    let mut step_counter = 1;
    let mut cli_field_counter = 0;

    println!("[general network settings 1-2/28]");
    // [1/28] --network-identifier
    let network_name = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.network_name.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some("wg-quickrs-home".into()),
        parse_and_validate_network_name,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [2/28] --network-subnet
    let network_subnet = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.network_subnet.clone().map(|o| o.to_string()),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some("10.0.34.0/24".into()),
        parse_and_validate_ipv4_subnet,
    );
    cli_field_counter += 1;
    step_counter += 1;

    println!("[general network settings complete]");
    println!("[agent settings 3-8/28]");

    // Get primary IP of the current machine
    let iface_opt = get_interfaces().into_iter().next();
    let iface_name = iface_opt.as_ref().map(|iface| iface.name.clone());
    let mut iface_ip = iface_opt.and_then(|iface| match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None });

    // [3/28] --agent-web-address
    let agent_web_address = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_web_address.clone().map(|o| o.to_string()),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        iface_ip.map(|o| o.to_string()),
        parse_and_validate_ipv4_address,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [4/28] --agent-web-http-enabled & --agent-web-http-port
    let agent_web_http_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_web_http_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let agent_web_http_port = if agent_web_http_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_web_http_port.map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("80".into()),
            parse_and_validate_port,
        )
    } else {
        // if disabled, use a default port of 80
        80
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [5/28] --agent-web-https-enabled & --agent-web-https-port
    let agent_web_https_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_web_https_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let (agent_web_https_port, agent_web_https_tls_cert, agent_web_https_tls_key) = if agent_web_https_enabled {
        let config_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        let (option_cert, option_key) = find_cert_server(&config_folder, agent_web_address.to_string());

        let port = get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_web_https_port.map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("443".into()),
            parse_and_validate_port,
        );
        let tls_cert = get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_web_https_tls_cert.clone().map(|o| o.display().to_string()),
            INIT_FLAGS[cli_field_counter+1],
            format!("\t{}", INIT_HELPS[cli_field_counter+1]).as_str(),
            option_cert.map(|o| o.display().to_string()),
            move |s: &str| parse_and_validate_tls_file(&config_folder, s),
        );
        let tls_key = get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_web_https_tls_key.clone().map(|o| o.display().to_string()),
            INIT_FLAGS[cli_field_counter+2],
            format!("\t{}", INIT_HELPS[cli_field_counter+2]).as_str(),
            option_key.map(|o| o.display().to_string()),
            move |s: &str| parse_and_validate_tls_file(&config_folder, s),
        );
        (port, tls_cert, tls_key)
    } else {
        // if disabled, use a default port of 443
        (443, Default::default(), Default::default())
    };
    cli_field_counter += 3;
    step_counter += 1;

    // [6/28] --agent-enable-web-password
    let mut agent_web_password_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_web_password_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    // [6/28] --agent-web-password
    let agent_web_password_hash = if agent_web_password_enabled {
        let password = get_init_password(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_web_password.clone(),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
        );
        let password_hash = helpers::calculate_password_hash(password.trim()).unwrap_or_else(|_| {
            eprintln!("unable to calculate password hash, disabling password");
            agent_web_password_enabled = false;
            "".into()
        });
        password_hash
    } else {
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [7/28] --agent-vpn-enabled & --agent-vpn-port
    let agent_vpn_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_vpn_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let agent_vpn_port = if agent_vpn_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_vpn_port.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("51820".into()),
            parse_and_validate_port,
        )
    } else {
        // if disabled, use a default port of 51820
        51820
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [8/28] --agent-firewall-enabled
    let agent_firewall_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_firewall_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let (agent_firewall_utility, agent_firewall_gateway) = if agent_firewall_enabled {
        // [8/28] --agent-firewall-utility
        let utility = get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_firewall_utility.clone().map(|o| o.display().to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            firewall_utility_options().into_iter().next().map(|o| o.display().to_string()),  // the first fw option is the default
            parse_and_validate_fw_utility,
        );
        // [8/28] --agent-firewall-gateway
        let gateway = get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_firewall_gateway.clone(),
            INIT_FLAGS[cli_field_counter+1],
            format!("\t{}", INIT_HELPS[cli_field_counter+1]).as_str(),
            iface_name,
            parse_and_validate_fw_gateway,
        );
        (utility, gateway)
    } else {
        ("".into(), "".into())
    };
    cli_field_counter += 2;
    step_counter += 1;

    println!("[agent settings complete]");
    println!("[peer settings 9-19/28]");

    // [9/28] --agent-peer-name
    let agent_peer_name = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_name.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some("wg-quickrs-host".into()),
        parse_and_validate_peer_name,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [10/28] --agent-peer-vpn-internal-address
    let temp_network = Network {
        name: "".to_string(),
        subnet: network_subnet.clone(),
        this_peer: Default::default(),
        peers: Default::default(),
        connections: Default::default(),
        defaults: Default::default(),
        reservations: Default::default(),
        updated_at: Utc::now(),
    };
    let agent_peer_vpn_internal_address = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_vpn_internal_address.clone().map(|o| o.to_string()),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        get_next_available_address(&network_subnet, &Vec::new()).map(|o| o.to_string()),
        move |s: &str| parse_and_validate_peer_address(s, &temp_network),
    );
    cli_field_counter += 1;
    step_counter += 1;

    // update the address in the recommended endpoint
    for iface in get_interfaces() {
        if agent_firewall_gateway == iface.name {
            iface_ip = match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None };
        }
    }

    // TODO: allow roaming init
    // [11/28] --agent-peer-vpn-endpoint
    let agent_peer_vpn_endpoint = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_vpn_endpoint.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some(format!("{}:51820", iface_ip.unwrap())),
        parse_and_validate_peer_endpoint,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [12/28] --agent-peer-kind
    let agent_peer_kind = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_kind.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some("server".into()),
        parse_and_validate_peer_kind,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [13/28] --agent-peer-icon-enabled & --agent-peer-icon-src
    let agent_peer_icon_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_icon_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_icon_src = if agent_peer_icon_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_icon_src.clone(),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_icon_src,
        )
    } else {
        // if disabled, default to an empty string
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [14/28] --agent-peer-dns-enabled & --agent-peer-dns-server
    let agent_peer_dns_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_dns_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let agent_peer_dns_addresses = if agent_peer_dns_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_dns_addresses.clone(),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("1.1.1.1".into()),
            parse_and_validate_peer_dns_addresses,
        )
    } else {
        // if disabled, default to an empty list
        vec![]
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [15/28] --agent-peer-mtu-enabled & --agent-peer-mtu-value
    let agent_peer_mtu_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_mtu_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_mtu_value = if agent_peer_mtu_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_mtu_value.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("1420".into()),
            parse_and_validate_peer_mtu_value,
        )
    } else {
        // if disabled, default to an mtu of 1420
        1420
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [16/28] --agent-peer-script-pre-up-enabled & --agent-peer-script-pre-up-line
    let agent_peer_script_pre_up_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_script_pre_up_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_script_pre_up_line = if agent_peer_script_pre_up_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_script_pre_up_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [17/28] --agent-peer-script-post-up-enabled & --agent-peer-script-post-up-line
    let agent_peer_script_post_up_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_script_post_up_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_script_post_up_line = if agent_peer_script_post_up_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_script_post_up_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [18/28] --agent-peer-script-pre-down-enabled & --agent-peer-script-pre-down-line
    let agent_peer_script_pre_down_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_script_pre_down_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_script_pre_down_line = if agent_peer_script_pre_down_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_script_pre_down_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [19/28] --agent-peer-script-post-down-enabled & --agent-peer-script-post-down-line
    let agent_peer_script_post_down_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_peer_script_post_down_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let agent_peer_script_post_down_line = if agent_peer_script_post_down_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.agent_peer_script_post_down_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    println!("[peer settings complete]");
    println!("[new peer/connection default settings 20-28/28]");

    // [20/28] --default-peer-kind
    let default_peer_kind = get_init_value(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_kind.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        Some("laptop".into()),
        parse_and_validate_peer_kind,
    );
    cli_field_counter += 1;
    step_counter += 1;

    // [21/28] --default-peer-icon-enabled & --default-peer-icon-src
    let default_peer_icon_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_icon_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_icon_src = if default_peer_icon_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_icon_src.clone(),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_icon_src,
        )
    } else {
        // if disabled, default to an empty string
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [22/28] --default-peer-dns-enabled & --default-peer-dns-server
    let default_peer_dns_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_dns_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let default_peer_dns_addresses = if default_peer_dns_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_dns_addresses.clone(),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("1.1.1.1".into()),
            parse_and_validate_peer_dns_addresses,
        )
    } else {
        // if disabled, default to an empty list
        vec![]
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [23/28] --default-peer-mtu-enabled & --default-peer-mtu-value
    let default_peer_mtu_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_mtu_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_mtu_value = if default_peer_mtu_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_mtu_value.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("1420".into()),
            parse_and_validate_peer_mtu_value,
        )
    } else {
        // if disabled, default to an mtu of 1420
        1420
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [24/28] --default-peer-script-pre-up-enabled & --default-peer-script-pre-up-line
    let default_peer_script_pre_up_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_script_pre_up_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_script_pre_up_line = if default_peer_script_pre_up_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_script_pre_up_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [25/28] --default-peer-script-post-up-enabled & --default-peer-script-post-up-line
    let default_peer_script_post_up_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_script_post_up_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_script_post_up_line = if default_peer_script_post_up_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_script_post_up_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [26/28] --default-peer-script-pre-down-enabled & --default-peer-script-pre-down-line
    let default_peer_script_pre_down_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_script_pre_down_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_script_pre_down_line = if default_peer_script_pre_down_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_script_pre_down_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [27/28] --default-peer-script-post-down-enabled & --default-peer-script-post-down-line
    let default_peer_script_post_down_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_peer_script_post_down_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        false,
    );
    cli_field_counter += 1;
    let default_peer_script_post_down_line = if default_peer_script_post_down_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_peer_script_post_down_line.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            None,
            parse_and_validate_peer_script,
        )
    } else {
        // if disabled, default to an empty list
        "".into()
    };
    cli_field_counter += 1;
    step_counter += 1;

    // [28/28] --default-connection-persistent-keepalive-enabled & --default-connection-persistent-keepalive-period
    let default_connection_persistent_keepalive_enabled = get_init_bool(
        init_opts.no_prompt,
        step_counter,
        init_opts.default_connection_persistent_keepalive_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;
    let default_connection_persistent_keepalive_period = if default_connection_persistent_keepalive_enabled {
        get_init_value(
            init_opts.no_prompt,
            step_counter,
            init_opts.default_connection_persistent_keepalive_period.clone().map(|o| o.to_string()),
            INIT_FLAGS[cli_field_counter],
            format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
            Some("25".into()),
            parse_and_validate_conn_persistent_keepalive_period,
        )
    } else {
        // if disabled, default to a period of 25
        25
    };

    println!("[new peer/connection default settings complete]");

    println!(
        "✅ This was all the information required to initialize wg-quickrs. Finalizing the configuration..."
    );

    let peer_id = Uuid::new_v4();
    let now = Utc::now();

    let mut config = Config {
        agent: Agent {
            web: AgentWeb {
                address: agent_web_address,
                http: AgentWebHttp {
                    enabled: agent_web_http_enabled,
                    port: agent_web_http_port,
                },
                https: AgentWebHttps {
                    enabled: agent_web_https_enabled,
                    port: agent_web_https_port,
                    tls_cert: agent_web_https_tls_cert,
                    tls_key: agent_web_https_tls_key,
                },
                password: Password {
                    enabled: agent_web_password_enabled,
                    hash: agent_web_password_hash,
                },
            },
            vpn: AgentVpn {
                enabled: agent_vpn_enabled,
                port: agent_vpn_port,
            },
            firewall: AgentFirewall {
                enabled: agent_firewall_enabled,
                utility: agent_firewall_utility,
                gateway: agent_firewall_gateway,
            },
        },
        network: Network {
            name: network_name.to_string(),
            subnet: network_subnet.clone(),
            this_peer: peer_id.clone(),
            peers: {
                let mut map = BTreeMap::new();
                map.insert(peer_id.clone(), Peer {
                    name: agent_peer_name.to_string(),
                    address: agent_peer_vpn_internal_address,
                    endpoint: Endpoint {
                        enabled: true,
                        address: agent_peer_vpn_endpoint.0,
                        port: agent_peer_vpn_endpoint.1,
                    },
                    kind: agent_peer_kind.to_string(),
                    icon: Icon {
                        enabled: agent_peer_icon_enabled,
                        src: agent_peer_icon_src,
                    },
                    private_key: wg_generate_key(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                    dns: Dns {
                        enabled: agent_peer_dns_enabled,
                        addresses: agent_peer_dns_addresses,
                    },
                    mtu: Mtu {
                        enabled: agent_peer_mtu_enabled,
                        value: agent_peer_mtu_value,
                    },
                    scripts: Scripts {
                        pre_up: vec![Script{ enabled: agent_peer_script_pre_up_enabled, script: agent_peer_script_pre_up_line }],
                        post_up: vec![Script{ enabled: agent_peer_script_post_up_enabled, script: agent_peer_script_post_up_line }],
                        pre_down: vec![Script{ enabled: agent_peer_script_pre_down_enabled, script: agent_peer_script_pre_down_line }],
                        post_down: vec![Script{ enabled: agent_peer_script_post_down_enabled, script: agent_peer_script_post_down_line }],
                    },
                });
                map
            },
            connections: BTreeMap::new(),
            reservations: BTreeMap::new(),
            updated_at: now,
            defaults: Defaults {
                peer: DefaultPeer {
                    endpoint: Endpoint {
                        enabled: false,
                        address: EndpointAddress::None,
                        port: 51820,
                    },
                    kind: default_peer_kind.to_string(),
                    icon: Icon{
                        enabled: default_peer_icon_enabled,
                        src: default_peer_icon_src,
                    },
                    dns: Dns {
                        enabled: default_peer_dns_enabled,
                        addresses: default_peer_dns_addresses,
                    },
                    mtu: Mtu {
                        enabled: default_peer_mtu_enabled,
                        value: default_peer_mtu_value,
                    },
                    scripts: Scripts {
                        pre_up: vec![Script{ enabled: default_peer_script_pre_up_enabled, script: default_peer_script_pre_up_line }],
                        post_up: vec![Script{ enabled: default_peer_script_post_up_enabled, script: default_peer_script_post_up_line }],
                        pre_down: vec![Script{ enabled: default_peer_script_pre_down_enabled, script: default_peer_script_pre_down_line }],
                        post_down: vec![Script{ enabled: default_peer_script_post_down_enabled, script: default_peer_script_post_down_line }],
                    },
                },
                connection: DefaultConnection {
                    persistent_keepalive: PersistentKeepalive {
                        enabled: default_connection_persistent_keepalive_enabled,
                        period: default_connection_persistent_keepalive_period,
                    },
                },
            },
        },
    };

    conf::util::set_config(&mut config)?;
    println!(
        "✅ Configuration saved to {}",
        WG_QUICKRS_CONFIG_FILE.get().unwrap().display()
    );

    Ok(())
}
