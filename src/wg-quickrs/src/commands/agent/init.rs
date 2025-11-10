use crate::{WG_QUICKRS_CONFIG_FILE, WG_QUICKRS_CONFIG_FOLDER};
use crate::conf;
use dialoguer;
use get_if_addrs::{Interface, get_if_addrs};
use wg_quickrs_cli::agent::InitOptions;
use wg_quickrs_lib::types::config::*;
use wg_quickrs_lib::types::network::*;
use wg_quickrs_lib::helpers::wg_generate_key;
use std::collections::{BTreeMap};
use std::net::{IpAddr};
use std::path::{PathBuf};
use std::{env, fs};
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;
use wg_quickrs_lib::validation::agent::{parse_and_validate_fw_gateway, parse_and_validate_ipv4_address, parse_and_validate_port, parse_and_validate_tls_file, parse_and_validate_fw_utility};
use wg_quickrs_lib::validation::helpers::firewall_utility_options;
use wg_quickrs_lib::validation::network::{parse_and_validate_conn_persistent_keepalive_period, parse_and_validate_ipv4_subnet, parse_and_validate_network_name, parse_and_validate_peer_address, parse_and_validate_peer_dns_addresses, parse_and_validate_peer_endpoint, parse_and_validate_peer_icon_src, parse_and_validate_peer_kind, parse_and_validate_peer_mtu_value, parse_and_validate_peer_name};
use crate::commands::helpers::*;
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

/// Handle other options
fn get_init_password(
    cli_no_prompt: Option<bool>,
    step_str: String,
    cli_value: Option<String>,
    cli_option: &str,
    description: &str,
) -> String {
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
    let step_str = make_step_formatter(28);

    println!("[general network settings 1-2/28]");
    // [1/28] --network-identifier
    let network_name = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.network_name.clone(),
        INIT_NETWORK_NAME_FLAG,
        INIT_NETWORK_NAME_HELP,
        Some("wg-quickrs-home".into()),
        parse_and_validate_network_name,
    );
    step_counter += 1;

    // [2/28] --network-subnet
    let network_subnet = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.network_subnet.clone().map(|o| o.to_string()),
        INIT_NETWORK_SUBNET_FLAG,
        INIT_NETWORK_SUBNET_HELP,
        Some("10.0.34.0/24".into()),
        parse_and_validate_ipv4_subnet,
    );
    step_counter += 1;

    println!("[general network settings complete]");
    println!("[agent settings 3-8/28]");

    // Get primary IP of the current machine
    let iface_opt = get_interfaces().into_iter().next();
    let iface_name = iface_opt.as_ref().map(|iface| iface.name.clone());
    let mut iface_ip = iface_opt.and_then(|iface| match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None });

    // [3/28] --agent-web-address
    let agent_web_address = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_address.clone().map(|o| o.to_string()),
        INIT_AGENT_WEB_ADDRESS_FLAG,
        INIT_AGENT_WEB_ADDRESS_HELP,
        iface_ip.map(|o| o.to_string()),
        parse_and_validate_ipv4_address,
    );
    step_counter += 1;

    // [4/28] --agent-web-http-enabled & --agent-web-http-port
    let agent_web_http_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_http_enabled,
        INIT_AGENT_WEB_HTTP_ENABLED_FLAG,
        INIT_AGENT_WEB_HTTP_ENABLED_HELP,
        true,
    );
    let agent_web_http_port = if agent_web_http_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_http_port.map(|o| o.to_string()),
            INIT_AGENT_WEB_HTTP_PORT_FLAG,
            format!("\t{}", INIT_AGENT_WEB_HTTP_PORT_HELP).as_str(),
            Some("80".into()),
            parse_and_validate_port,
        )
    } else {
        // if disabled, use a default port of 80
        80
    };
    step_counter += 1;

    // [5/28] --agent-web-https-enabled & --agent-web-https-port
    let agent_web_https_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_https_enabled,
        INIT_AGENT_WEB_HTTPS_ENABLED_FLAG,
        INIT_AGENT_WEB_HTTPS_ENABLED_HELP,
        true,
    );
    let (agent_web_https_port, agent_web_https_tls_cert, agent_web_https_tls_key) = if agent_web_https_enabled {
        let config_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        let (option_cert, option_key) = find_cert_server(&config_folder, agent_web_address.to_string());

        let port = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_https_port.map(|o| o.to_string()),
            INIT_AGENT_WEB_HTTPS_PORT_FLAG,
            format!("\t{}", INIT_AGENT_WEB_HTTPS_PORT_HELP).as_str(),
            Some("443".into()),
            parse_and_validate_port,
        );
        let tls_cert = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_https_tls_cert.clone().map(|o| o.display().to_string()),
            INIT_AGENT_WEB_HTTPS_TLS_CERT_FLAG,
            format!("\t{}", INIT_AGENT_WEB_HTTPS_TLS_CERT_HELP).as_str(),
            option_cert.map(|o| o.display().to_string()),
            move |s: &str| parse_and_validate_tls_file(&config_folder, s),
        );
        let tls_key = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_https_tls_key.clone().map(|o| o.display().to_string()),
            INIT_AGENT_WEB_HTTPS_TLS_KEY_FLAG,
            format!("\t{}", INIT_AGENT_WEB_HTTPS_TLS_KEY_HELP).as_str(),
            option_key.map(|o| o.display().to_string()),
            move |s: &str| parse_and_validate_tls_file(&config_folder, s),
        );
        (port, tls_cert, tls_key)
    } else {
        // if disabled, use a default port of 443
        (443, Default::default(), Default::default())
    };
    step_counter += 1;

    // [6/28] --agent-enable-web-password
    let mut agent_web_password_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_password_enabled,
        INIT_AGENT_WEB_PASSWORD_ENABLED_FLAG,
        INIT_AGENT_WEB_PASSWORD_ENABLED_HELP,
        true,
    );
    // [6/28] --agent-web-password
    let agent_web_password_hash = if agent_web_password_enabled {
        let password = get_init_password(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_password.clone(),
            INIT_AGENT_WEB_PASSWORD_FLAG,
            format!("\t{}", INIT_AGENT_WEB_PASSWORD_HELP).as_str(),
        );
        let password_hash = calculate_password_hash(password.trim()).unwrap_or_else(|_| {
            eprintln!("unable to calculate password hash, disabling password");
            agent_web_password_enabled = false;
            "".into()
        });
        password_hash
    } else {
        "".into()
    };
    step_counter += 1;

    // [7/28] --agent-vpn-enabled & --agent-vpn-port
    let agent_vpn_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_vpn_enabled,
        INIT_AGENT_VPN_ENABLED_FLAG,
        INIT_AGENT_VPN_ENABLED_HELP,
        true,
    );
    let agent_vpn_port = if agent_vpn_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_vpn_port.clone().map(|o| o.to_string()),
            INIT_AGENT_VPN_PORT_FLAG,
            format!("\t{}", INIT_AGENT_VPN_PORT_HELP).as_str(),
            Some("51820".into()),
            parse_and_validate_port,
        )
    } else {
        // if disabled, use a default port of 51820
        51820
    };
    step_counter += 1;

    // [8/28] --agent-firewall-enabled
    let agent_firewall_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_firewall_enabled,
        INIT_AGENT_FIREWALL_ENABLED_FLAG,
        INIT_AGENT_FIREWALL_ENABLED_HELP,
        true,
    );
    let (agent_firewall_utility, agent_firewall_gateway) = if agent_firewall_enabled {
        // [8/28] --agent-firewall-utility
        let utility = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_firewall_utility.clone().map(|o| o.display().to_string()),
            INIT_AGENT_FIREWALL_UTILITY_FLAG,
            format!("\t{}", INIT_AGENT_FIREWALL_UTILITY_HELP).as_str(),
            firewall_utility_options().into_iter().next().map(|o| o.display().to_string()),  // the first fw option is the default
            parse_and_validate_fw_utility,
        );
        // [8/28] --agent-firewall-gateway
        let gateway = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_firewall_gateway.clone(),
            INIT_AGENT_FIREWALL_GATEWAY_FLAG,
            format!("\t{}", INIT_AGENT_FIREWALL_GATEWAY_HELP).as_str(),
            iface_name,
            parse_and_validate_fw_gateway,
        );
        (utility, gateway)
    } else {
        ("".into(), "".into())
    };
    step_counter += 1;

    println!("[agent settings complete]");
    println!("[peer settings 9-19/28]");

    // [9/28] --agent-peer-name
    let agent_peer_name = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_name.clone(),
        INIT_AGENT_PEER_NAME_FLAG,
        INIT_AGENT_PEER_NAME_HELP,
        Some("wg-quickrs-host".into()),
        parse_and_validate_peer_name,
    );
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
    let agent_peer_vpn_internal_address = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_vpn_internal_address.clone().map(|o| o.to_string()),
        INIT_AGENT_PEER_VPN_INTERNAL_ADDRESS_FLAG,
        INIT_AGENT_PEER_VPN_INTERNAL_ADDRESS_HELP,
        get_next_available_address(&network_subnet, &Vec::new()).map(|o| o.to_string()),
        move |s: &str| parse_and_validate_peer_address(s, &temp_network),
    );
    step_counter += 1;

    // update the address in the recommended endpoint
    for iface in get_interfaces() {
        if agent_firewall_gateway == iface.name {
            iface_ip = match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None };
        }
    }

    // TODO: allow roaming init
    // [11/28] --agent-peer-vpn-endpoint
    let agent_peer_vpn_endpoint = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_vpn_endpoint.clone(),
        INIT_AGENT_PEER_VPN_ENDPOINT_FLAG,
        INIT_AGENT_PEER_VPN_ENDPOINT_HELP,
        Some(format!("{}:51820", iface_ip.unwrap())),
        parse_and_validate_peer_endpoint,
    );
    step_counter += 1;

    // [12/28] --agent-peer-kind
    let agent_peer_kind = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_kind.clone(),
        INIT_AGENT_PEER_KIND_FLAG,
        INIT_AGENT_PEER_KIND_HELP,
        Some("server".into()),
        parse_and_validate_peer_kind,
    );
    step_counter += 1;

    // [13/28] --agent-peer-icon-enabled & --agent-peer-icon-src
    let agent_peer_icon_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_icon_enabled,
        INIT_AGENT_PEER_ICON_ENABLED_FLAG,
        INIT_AGENT_PEER_ICON_ENABLED_HELP,
        false,
    );
    let agent_peer_icon_src = if agent_peer_icon_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_icon_src.clone(),
            INIT_AGENT_PEER_ICON_SRC_FLAG,
            format!("\t{}", INIT_AGENT_PEER_ICON_SRC_HELP).as_str(),
            None,
            parse_and_validate_peer_icon_src,
        )
    } else {
        // if disabled, default to an empty string
        "".into()
    };
    step_counter += 1;

    // [14/28] --agent-peer-dns-enabled & --agent-peer-dns-server
    let agent_peer_dns_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_dns_enabled,
        INIT_AGENT_PEER_DNS_ENABLED_FLAG,
        INIT_AGENT_PEER_DNS_ENABLED_HELP,
        true,
    );
    let agent_peer_dns_addresses = if agent_peer_dns_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_dns_addresses.clone(),
            INIT_AGENT_PEER_DNS_ADDRESSES_FLAG,
            format!("\t{}", INIT_AGENT_PEER_DNS_ADDRESSES_HELP).as_str(),
            Some("1.1.1.1".into()),
            parse_and_validate_peer_dns_addresses,
        )
    } else {
        // if disabled, default to an empty list
        vec![]
    };
    step_counter += 1;

    // [15/28] --agent-peer-mtu-enabled & --agent-peer-mtu-value
    let agent_peer_mtu_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_mtu_enabled,
        INIT_AGENT_PEER_MTU_ENABLED_FLAG,
        INIT_AGENT_PEER_MTU_ENABLED_HELP,
        false,
    );
    let agent_peer_mtu_value = if agent_peer_mtu_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_mtu_value.clone().map(|o| o.to_string()),
            INIT_AGENT_PEER_MTU_VALUE_FLAG,
            format!("\t{}", INIT_AGENT_PEER_MTU_VALUE_HELP).as_str(),
            Some("1420".into()),
            parse_and_validate_peer_mtu_value,
        )
    } else {
        // if disabled, default to an mtu of 1420
        1420
    };
    step_counter += 1;

    // [16/28] --agent-peer-script-pre-up-enabled & --agent-peer-script-pre-up-line
    let agent_peer_script_pre_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_pre_up_enabled,
        init_opts.agent_peer_script_pre_up_line.clone(),
        INIT_AGENT_PEER_SCRIPT_PRE_UP_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_UP_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_UP_ENABLED_HELP,
        INIT_AGENT_PEER_SCRIPT_PRE_UP_LINE_HELP,
    );
    step_counter += 1;

    // [17/28] --agent-peer-script-post-up-enabled & --agent-peer-script-post-up-line
    let agent_peer_script_post_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_post_up_enabled,
        init_opts.agent_peer_script_post_up_line.clone(),
        INIT_AGENT_PEER_SCRIPT_POST_UP_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_UP_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_UP_ENABLED_HELP,
        INIT_AGENT_PEER_SCRIPT_POST_UP_LINE_HELP,
    );
    step_counter += 1;

    // [18/28] --agent-peer-script-pre-down-enabled & --agent-peer-script-pre-down-line
    let agent_peer_script_pre_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_pre_down_enabled,
        init_opts.agent_peer_script_pre_down_line.clone(),
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_ENABLED_HELP,
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_LINE_HELP,
    );
    step_counter += 1;

    // [19/28] --agent-peer-script-post-down-enabled & --agent-peer-script-post-down-line
    let agent_peer_script_post_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_post_down_enabled,
        init_opts.agent_peer_script_post_down_line.clone(),
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_ENABLED_HELP,
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_LINE_HELP,
    );
    step_counter += 1;

    println!("[peer settings complete]");
    println!("[new peer/connection default settings 20-28/28]");

    // [20/28] --default-peer-kind
    let default_peer_kind = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_kind.clone(),
        INIT_DEFAULT_PEER_KIND_FLAG,
        INIT_DEFAULT_PEER_KIND_HELP,
        Some("laptop".into()),
        parse_and_validate_peer_kind,
    );
    step_counter += 1;

    // [21/28] --default-peer-icon-enabled & --default-peer-icon-src
    let default_peer_icon_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_icon_enabled,
        INIT_DEFAULT_PEER_ICON_ENABLED_FLAG,
        INIT_DEFAULT_PEER_ICON_ENABLED_HELP,
        false,
    );
    let default_peer_icon_src = if default_peer_icon_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_icon_src.clone(),
            INIT_DEFAULT_PEER_ICON_SRC_FLAG,
            format!("\t{}", INIT_DEFAULT_PEER_ICON_SRC_HELP).as_str(),
            None,
            parse_and_validate_peer_icon_src,
        )
    } else {
        // if disabled, default to an empty string
        "".into()
    };
    step_counter += 1;

    // [22/28] --default-peer-dns-enabled & --default-peer-dns-server
    let default_peer_dns_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_dns_enabled,
        INIT_DEFAULT_PEER_DNS_ENABLED_FLAG,
        INIT_DEFAULT_PEER_DNS_ENABLED_HELP,
        true,
    );
    let default_peer_dns_addresses = if default_peer_dns_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_dns_addresses.clone(),
            INIT_DEFAULT_PEER_DNS_ADDRESSES_FLAG,
            format!("\t{}", INIT_DEFAULT_PEER_DNS_ADDRESSES_HELP).as_str(),
            Some("1.1.1.1".into()),
            parse_and_validate_peer_dns_addresses,
        )
    } else {
        // if disabled, default to an empty list
        vec![]
    };
    step_counter += 1;

    // [23/28] --default-peer-mtu-enabled & --default-peer-mtu-value
    let default_peer_mtu_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_mtu_enabled,
        INIT_DEFAULT_PEER_MTU_ENABLED_FLAG,
        INIT_DEFAULT_PEER_MTU_ENABLED_HELP,
        false,
    );
    let default_peer_mtu_value = if default_peer_mtu_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_mtu_value.clone().map(|o| o.to_string()),
            INIT_DEFAULT_PEER_MTU_VALUE_FLAG,
            format!("\t{}", INIT_DEFAULT_PEER_MTU_VALUE_HELP).as_str(),
            Some("1420".into()),
            parse_and_validate_peer_mtu_value,
        )
    } else {
        // if disabled, default to an mtu of 1420
        1420
    };
    step_counter += 1;

    // [24/28] --default-peer-script-pre-up-enabled & --default-peer-script-pre-up-line
    let default_peer_script_pre_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_pre_up_enabled,
        init_opts.default_peer_script_pre_up_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_ENABLED_HELP,
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_LINE_HELP,
    );
    step_counter += 1;

    // [25/28] --default-peer-script-post-up-enabled & --default-peer-script-post-up-line
    let default_peer_script_post_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_post_up_enabled,
        init_opts.default_peer_script_post_up_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_ENABLED_HELP,
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_LINE_HELP,
    );
    step_counter += 1;

    // [26/28] --default-peer-script-pre-down-enabled & --default-peer-script-pre-down-line
    let default_peer_script_pre_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_pre_down_enabled,
        init_opts.default_peer_script_pre_down_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_ENABLED_HELP,
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_LINE_HELP,
    );
    step_counter += 1;

    // [27/28] --default-peer-script-post-down-enabled & --default-peer-script-post-down-line
    let default_peer_script_post_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_post_down_enabled,
        init_opts.default_peer_script_post_down_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_ENABLED_HELP,
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_LINE_HELP,
    );
    step_counter += 1;

    // [28/28] --default-connection-persistent-keepalive-enabled & --default-connection-persistent-keepalive-period
    let default_connection_persistent_keepalive_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_connection_persistent_keepalive_enabled,
        INIT_DEFAULT_CONNECTION_PERSISTENT_KEEPALIVE_ENABLED_FLAG,
        format!("\t{}", INIT_DEFAULT_CONNECTION_PERSISTENT_KEEPALIVE_ENABLED_HELP).as_str(),
        true,
    );
    let default_connection_persistent_keepalive_period = if default_connection_persistent_keepalive_enabled {
        get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_connection_persistent_keepalive_period.clone().map(|o| o.to_string()),
            INIT_DEFAULT_CONNECTION_PERSISTENT_KEEPALIVE_PERIOD_FLAG,
            format!("\t{}", INIT_DEFAULT_CONNECTION_PERSISTENT_KEEPALIVE_PERIOD_HELP).as_str(),
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
                        address: agent_peer_vpn_endpoint,
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
                        pre_up: agent_peer_script_pre_up,
                        post_up: agent_peer_script_post_up,
                        pre_down: agent_peer_script_pre_down,
                        post_down: agent_peer_script_post_down,
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
                        pre_up: default_peer_script_pre_up,
                        post_up: default_peer_script_post_up,
                        pre_down: default_peer_script_pre_down,
                        post_down: default_peer_script_post_down,
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
