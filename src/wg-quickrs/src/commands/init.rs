use crate::commands::helpers;
use crate::commands::validation::{check_field_enabled_value_agent, check_field_path_agent, check_field_str_agent};
use crate::{WG_QUICKRS_CONFIG_FILE, WG_QUICKRS_CONFIG_FOLDER};
use crate::conf;
use dialoguer;
use get_if_addrs::{Interface, get_if_addrs};
use ipnetwork::IpNetwork;
use wg_quickrs_cli::InitOptions;
use wg_quickrs_wasm::types::{
    Agent, AgentFirewall, AgentVpn, AgentWeb, AgentWebHttp, AgentWebHttps, Config,
    DefaultConnection, DefaultPeer, Defaults, EnabledValue, Network, Password, Peer, Scripts,
};
use wg_quickrs_wasm::timestamp::get_now_timestamp_formatted;
use wg_quickrs_wasm::helpers::wg_generate_key;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};
use uuid::Uuid;
use wg_quickrs_wasm::validation::check_internal_address;

include!(concat!(env!("OUT_DIR"), "/init_options_generated.rs"));

// Get first usable IP from subnet
fn first_ip(subnet: &str) -> String {
    let net: IpNetwork = subnet.parse().expect("Invalid subnet");
    match net {
        IpNetwork::V4(net) => {
            let mut octets = net.network().octets();
            octets[3] += 1; // Add 1 to the last octet
            Ipv4Addr::from(octets).to_string()
        }
        _ => panic!("IPv6 is not supported"),
    }
}

// Get primary IP of the current machine
pub fn get_interfaces() -> Vec<Interface> {
    let mut interfaces: Vec<Interface> = Vec::new();
    for iface in get_if_addrs()
        .unwrap()
        .iter()
        .filter(|a| !a.is_loopback() && a.ip().is_ipv4())
    {
        interfaces.push(iface.clone());
    }
    interfaces
}

// Get primary IP of the current machine
fn primary_ip_interface() -> Option<Interface> {
    get_interfaces().into_iter().next()
}

fn find_in_path(cmd: &str) -> Option<PathBuf> {
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                return Some(full_path);
            }
        }
    }
    None
}

pub fn firewall_utility_options() -> Vec<String> {
    let candidates = ["iptables", "pfctl"];
    let mut ret: Vec<String> = Vec::new();
    for prog in candidates {
        if let Some(path) = find_in_path(prog) {
            ret.push(path.to_str().unwrap().to_string());
        }
    }
    ret
}

fn find_firewall_utility() -> Option<String> {
    if let Some(prog) = firewall_utility_options().into_iter().next() {
        return Some(prog);
    }
    None
}

fn find_cert_server(web_address: String) -> (Option<String>, Option<String>) {
    let config_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
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
                    .to_string_lossy()
                    .into(),
            ),
            Some(
                servers_folder
                    .join(&web_address)
                    .join("key.pem")
                    .strip_prefix(config_folder).unwrap()
                    .to_string_lossy()
                    .into(),
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
            Some(cert.to_string_lossy().into()),
            Some(key.to_string_lossy().into()),
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
fn get_init_bool_option(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_value: Option<bool>,
    cli_option: &str,
    description: &str,
    default: bool,
) -> bool {
    let step_str = step_str(step);
    match cli_value {
        Some(v) => {
            println!(
                "{} {} is {} from CLI option '{}'",
                step_str,
                description,
                if v { "enabled" } else { "disabled" },
                cli_option
            );
            v
        }
        None => match cli_no_prompt {
            Some(true) => panic!("Error: CLI option '{}' is not set", cli_option),
            _ => dialoguer::Confirm::new()
                .with_prompt(format!(
                    "{} {} (CLI option '{}')?",
                    step_str, description, cli_option
                ))
                .default(default)
                .interact()
                .unwrap(),
        },
    }
}

/// Helper to prompt a value with optional default and checks
fn prompt<T: std::str::FromStr + ToString>(field_name: &str, msg: &str, default: Option<T>, network: Option<Network>) -> T {
    loop {
        let input = if let Some(d) = &default {
            dialoguer::Input::new()
                .with_prompt(msg.to_string())
                .default(d.to_string())
                .interact_text()
        } else {
            dialoguer::Input::new()
                .with_prompt(msg.to_string())
                .interact_text()
        };

        match input {
            Ok(value) => {
                let result = match field_name {
                    "endpoint" | "icon" | "dns" | "mtu" | "script" | "pre_up" | "post_up" | "pre_down" | "post_down" | "persistent_keepalive" => check_field_enabled_value_agent(field_name, &EnabledValue{
                        enabled: true,
                        value: value.clone(),
                    }),
                    "path" | "firewall-utility" => check_field_path_agent(field_name, &PathBuf::from(value.clone())),
                    "internal-address" => check_internal_address(&value.to_string(), &network.clone().unwrap()),
                    _ => check_field_str_agent(field_name, &value),
                };

                if result.status {
                    if let Ok(parsed) = value.parse::<T>() {
                        return parsed;
                    } else {
                        println!("ERROR: Parsing failed. Try again.");
                    }
                } else {
                    println!("ERROR: {}", result.msg);
                }
            }
            Err(_) => {
                println!("ERROR: Error reading input, please try again.");
                continue;
            }
        }
    }
}

/// Handle enabled value options
#[allow(clippy::too_many_arguments)]
fn get_init_enabled_value_option<T: std::str::FromStr + std::fmt::Display + Clone + Default>(
    cli_no_prompt: Option<bool>,
    step: usize,
    field_name: &str,
    cli_value: Option<T>,
    cli_option: &str,
    description: &str,
    condition: bool,
    default_value: Option<T>,
    network: Option<Network>,
) -> T {
    let step_str = step_str(step);
    match cli_value {
        Some(v) => {
            let result = match field_name {
                "endpoint" | "icon" | "dns" | "mtu" | "script" | "pre_up" | "post_up" | "pre_down" | "post_down" | "persistent_keepalive" => check_field_enabled_value_agent(field_name, &EnabledValue{
                    enabled: true,
                    value: v.to_string(),
                }),
                "path" | "firewall-utility" => check_field_path_agent(field_name, &PathBuf::from(v.to_string())),
                "internal-address" => check_internal_address(&v.to_string(), &network.unwrap()),
                _ => check_field_str_agent(field_name, &v.to_string()),
            };

            if !result.status {
                panic!("Error: CLI option '{}={}' is invalid: {}", cli_option, v, result.msg)
            }

            println!(
                "{} Using {} from CLI option '{}': {}",
                step_str, description, cli_option, v
            );
            v
        }
        None => match cli_no_prompt {
            Some(true) => {
                if condition {
                    panic!("Error: CLI option '{}' is not set", cli_option)
                } else {
                    Default::default()
                }
            }
            _ => {
                if condition {
                    prompt(
                        field_name,
                        &format!("{} {} (CLI option '{}')", step_str, description, cli_option),
                        default_value,
                        network
                    )
                } else {
                    Default::default()
                }
            }
        },
    }
}

/// Macro to handle paired "enable + value" options
macro_rules! get_init_pair_option {
    (
        $cli_no_prompt:expr,
        $step:expr,
        $field_name:expr,
        $cli_enable:expr,
        $cli_value:expr,
        $cli_enable_option:expr,
        $cli_value_option:expr,
        $description_enable:expr,
        $description_value:expr,
        $default_enable:expr,
        $default_value:expr
    ) => {{
        let enabled = get_init_bool_option(
            $cli_no_prompt,
            $step,
            $cli_enable,
            $cli_enable_option,
            $description_enable,
            $default_enable,
        );

        let value = get_init_enabled_value_option(
            $cli_no_prompt,
            $step,
            $field_name,
            $cli_value,
            $cli_value_option,
            $description_value,
            enabled,
            $default_value,
            None
        );

        (enabled, value)
    }};
}

pub fn initialize_agent(init_opts: &InitOptions) -> ExitCode {
    if let Err(conf::util::ConfUtilError::Read(..)) = conf::util::get_config() {
    } else {
        log::error!("wg-quickrs wg-quickrs is already initialized.");
        return ExitCode::FAILURE;
    }
    log::info!("Initializing wg-quickrs...");

    let mut step_counter = 1;
    let mut cli_field_counter = 0;

    println!("[general network settings 1-2/28]");
    // [1/28] --network-identifier
    let network_identifier = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "identifier",
        init_opts.network_identifier.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some("wg-quickrs-home".into()),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [2/28] --network-subnet
    let network_subnet = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "subnet",
        init_opts.network_subnet.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some("10.0.34.0/24".into()),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    println!("[general network settings complete]");
    println!("[agent settings 3-8/28]");

    let iface_opt = primary_ip_interface();
    let iface_name = iface_opt.as_ref().map(|iface| iface.name.clone());
    let mut iface_ip = iface_opt.map(|iface| iface.ip().to_string());

    // [3/28] --agent-web-address
    let agent_web_address = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "generic-address",
        init_opts.agent_web_address.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        iface_ip.clone(),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [4/28] --agent-web-http-enabled & --agent-web-http-port
    let (agent_web_http_enabled, mut agent_web_http_port) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "port",
        init_opts.agent_web_http_enabled,
        init_opts.agent_web_http_port,
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some(80)
    );
    // if disabled, use a default port of 80
    if !agent_web_http_enabled {
        agent_web_http_port = 80;
    }
    step_counter += 1;
    cli_field_counter += 2;

    // [5/28] --agent-web-https-enabled & --agent-web-https-port
    let (agent_web_https_enabled, mut agent_web_https_port) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "port",
        init_opts.agent_web_https_enabled,
        init_opts.agent_web_https_port,
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some(443)
    );
    // if disabled, use a default port of 443
    if !agent_web_https_enabled {
        agent_web_https_port = 443;
    }
    cli_field_counter += 2;

    let (option_cert, option_key) = find_cert_server(agent_web_address.clone());

    // [5/28] --agent-web-https-tls-cert
    let agent_web_https_tls_cert = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "path",
        init_opts
            .agent_web_https_tls_cert
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
        agent_web_https_enabled,
        option_cert,
        None
    );
    cli_field_counter += 1;

    // [5/28] --agent-web-https-tls-key
    let agent_web_https_tls_key = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "path",
        init_opts
            .agent_web_https_tls_key
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
        agent_web_https_enabled,
        option_key,
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [6/28] --agent-enable-web-password
    let agent_web_password_enabled = get_init_bool_option(
        init_opts.no_prompt,
        step_counter,
        init_opts.agent_web_password_enabled,
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
    );
    cli_field_counter += 1;

    // [6/28] --agent-web-password
    let agent_web_password = match init_opts.agent_web_password.clone() {
        Some(v) => {
            println!(
                "{}  Using password for the web server from CLI argument: ***hidden***",
                step_str(step_counter)
            );
            v.clone()
        }
        _ => match init_opts.no_prompt {
            Some(true) => {
                if agent_web_password_enabled {
                    panic!("Error: {} option is not set", INIT_FLAGS[cli_field_counter])
                } else {
                    "".into()
                }
            }
            _ => {
                if agent_web_password_enabled {
                    dialoguer::Password::new()
                        .with_prompt(format!(
                            "{} \t{}",
                            step_str(step_counter),
                            INIT_HELPS[cli_field_counter]
                        ))
                        .interact()
                        .unwrap()
                } else {
                    "".into()
                }
            }
        },
    };
    let agent_web_password_hash = match helpers::calculate_password_hash(agent_web_password.trim())
    {
        Ok(p) => {
            if agent_web_password_enabled {
                p
            } else {
                "".into()
            }
        }
        Err(e) => {
            return e;
        }
    };
    step_counter += 1;
    cli_field_counter += 1;

    // [7/28] --agent-vpn-enabled & --agent-vpn-port
    let (agent_vpn_enabled, mut agent_vpn_port) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "port",
        init_opts.agent_vpn_enabled,
        init_opts.agent_vpn_port,
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some(51820)
    );
    // if disabled, use a default port of 51820
    if !agent_vpn_enabled {
        agent_vpn_port = 51820;
    }
    step_counter += 1;
    cli_field_counter += 2;

    // [8/28] --agent-firewall-enabled & --agent-firewall-utility
    let (agent_firewall_enabled, agent_firewall_utility) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "firewall-utility",
        init_opts.agent_firewall_enabled,
        init_opts
            .agent_firewall_utility
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        find_firewall_utility()
    );
    cli_field_counter += 2;

    // [8/28] --agent-firewall-gateway
    let agent_firewall_gateway = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "firewall-gateway",
        init_opts.agent_firewall_gateway.clone(),
        INIT_FLAGS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter]).as_str(),
        agent_firewall_enabled,
        iface_name,
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    println!("[agent settings complete]");
    println!("[peer settings 9-19/28]");

    // [9/28] --agent-peer-name
    let agent_peer_name = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "name",
        init_opts.agent_peer_name.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some("wg-quickrs-host".into()),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // update the address in the recommended endpoint
    for interface in get_interfaces() {
        if agent_firewall_gateway.clone() == interface.name.clone() {
            iface_ip = Some(interface.ip().to_string());
        }
    }

    // [10/28] --agent-peer-vpn-internal-address
    let agent_peer_vpn_internal_address = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "internal-address",
        init_opts.agent_peer_vpn_internal_address.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some(first_ip(&network_subnet)),
        Some(Network {
            identifier: "".to_string(),
            subnet: network_subnet.clone(),
            this_peer: "".to_string(),
            peers: Default::default(),
            connections: Default::default(),
            defaults: Default::default(),
            reservations: Default::default(),
            updated_at: "".to_string(),
        })
    );
    step_counter += 1;
    cli_field_counter += 1;

    // TODO: allow roaming init
    // [11/28] --agent-peer-vpn-endpoint
    let agent_peer_vpn_endpoint = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "endpoint",
        init_opts.agent_peer_vpn_endpoint.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        format!("{}:51820", iface_ip.unwrap()).into(),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [12/28] --agent-peer-kind
    let agent_peer_kind = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "kind",
        init_opts.agent_peer_kind.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some("server".into()),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [13/28] --agent-peer-icon-enabled & --agent-peer-icon-src
    let (agent_peer_icon_enabled, agent_peer_icon_src) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "icon",
        init_opts.agent_peer_icon_enabled,
        init_opts.agent_peer_icon_src.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [14/28] --agent-peer-dns-enabled & --agent-peer-dns-server
    let (agent_peer_dns_enabled, agent_peer_dns_server) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "dns",
        init_opts.agent_peer_dns_enabled,
        init_opts.agent_peer_dns_server.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some("1.1.1.1".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [15/28] --agent-peer-mtu-enabled & --agent-peer-mtu-value
    let (agent_peer_mtu_enabled, agent_peer_mtu_value) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "mtu",
        init_opts.agent_peer_mtu_enabled,
        init_opts.agent_peer_mtu_value.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("1420".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [16/28] --agent-peer-script-pre-up-enabled & --agent-peer-script-pre-up-line
    let (_, agent_peer_script_pre_up_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.agent_peer_script_pre_up_enabled,
        init_opts.agent_peer_script_pre_up_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut agent_peer_script_pre_up_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in agent_peer_script_pre_up_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        agent_peer_script_pre_up_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [17/28] --agent-peer-script-post-up-enabled & --agent-peer-script-post-up-line
    let (_, agent_peer_script_post_up_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.agent_peer_script_post_up_enabled,
        init_opts.agent_peer_script_post_up_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut agent_peer_script_post_up_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in agent_peer_script_post_up_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        agent_peer_script_post_up_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [18/28] --agent-peer-script-pre-down-enabled & --agent-peer-script-pre-down-line
    let (_, agent_peer_script_pre_down_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.agent_peer_script_pre_down_enabled,
        init_opts.agent_peer_script_pre_down_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut agent_peer_script_pre_down_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in agent_peer_script_pre_down_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        agent_peer_script_pre_down_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [19/28] --agent-peer-script-post-down-enabled & --agent-peer-script-post-down-line
    let (_, agent_peer_script_post_down_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.agent_peer_script_post_down_enabled,
        init_opts.agent_peer_script_post_down_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut agent_peer_script_post_down_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in agent_peer_script_post_down_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        agent_peer_script_post_down_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    println!("[peer settings complete]");
    println!("[new peer/connection default settings 20-28/28]");

    // [20/28] --default-peer-kind
    let default_peer_kind = get_init_enabled_value_option(
        init_opts.no_prompt,
        step_counter,
        "kind",
        init_opts.default_peer_kind.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_HELPS[cli_field_counter],
        true,
        Some("laptop".into()),
        None
    );
    step_counter += 1;
    cli_field_counter += 1;

    // [21/28] --default-peer-icon-enabled & --default-peer-icon-src
    let (default_peer_icon_enabled, default_peer_icon_src) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "icon",
        init_opts.default_peer_icon_enabled,
        init_opts.default_peer_icon_src.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [22/28] --default-peer-dns-enabled & --default-peer-dns-server
    let (default_peer_dns_enabled, default_peer_dns_server) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "dns",
        init_opts.default_peer_dns_enabled,
        init_opts.default_peer_dns_server.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some("1.1.1.1".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [23/28] --default-peer-mtu-enabled & --default-peer-mtu-value
    let (default_peer_mtu_enabled, default_peer_mtu_value) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "mtu",
        init_opts.default_peer_mtu_enabled,
        init_opts.default_peer_mtu_value.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("1420".into())
    );
    step_counter += 1;
    cli_field_counter += 2;

    // [24/28] --default-peer-script-pre-up-enabled & --default-peer-script-pre-up-line
    let (_, default_peer_script_pre_up_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.default_peer_script_pre_up_enabled,
        init_opts.default_peer_script_pre_up_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut default_peer_script_pre_up_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in default_peer_script_pre_up_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        default_peer_script_pre_up_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [25/28] --default-peer-script-post-up-enabled & --default-peer-script-post-up-line
    let (_, default_peer_script_post_up_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.default_peer_script_post_up_enabled,
        init_opts.default_peer_script_post_up_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut default_peer_script_post_up_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in default_peer_script_post_up_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        default_peer_script_post_up_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [26/28] --default-peer-script-pre-down-enabled & --default-peer-script-pre-down-line
    let (_, default_peer_script_pre_down_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.default_peer_script_pre_down_enabled,
        init_opts.default_peer_script_pre_down_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut default_peer_script_pre_down_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in default_peer_script_pre_down_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        default_peer_script_pre_down_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [27/28] --default-peer-script-post-down-enabled & --default-peer-script-post-down-line
    let (_, default_peer_script_post_down_string_lines) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "script",
        init_opts.default_peer_script_post_down_enabled,
        init_opts.default_peer_script_post_down_line.clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        false,
        Some("".into())
    );
    step_counter += 1;
    cli_field_counter += 2;
    let mut default_peer_script_post_down_lines: Vec<EnabledValue> = Vec::new();
    for script_string_line in default_peer_script_post_down_string_lines
        .split(";")
        .filter(|&x| !x.is_empty())
    {
        default_peer_script_post_down_lines.push(EnabledValue {
            enabled: true,
            value: format!("{script_string_line};"),
        })
    }

    // [28/28] --default-connection-persistent-keepalive-enabled & --default-connection-persistent-keepalive-period
    let (
        default_connection_persistent_keepalive_enabled,
        default_connection_persistent_keepalive_period,
    ) = get_init_pair_option!(
        init_opts.no_prompt,
        step_counter,
        "persistent_keepalive",
        init_opts.default_connection_persistent_keepalive_enabled,
        init_opts
            .default_connection_persistent_keepalive_period
            .clone(),
        INIT_FLAGS[cli_field_counter],
        INIT_FLAGS[cli_field_counter + 1],
        INIT_HELPS[cli_field_counter],
        format!("\t{}", INIT_HELPS[cli_field_counter + 1]).as_str(),
        true,
        Some("25".into())
    );

    println!("[new peer/connection default settings complete]");

    println!(
        "✅ This was all the information required to initialize wg-quickrs. Finalizing the configuration..."
    );

    let peer_id = Uuid::new_v4().to_string();
    let now = get_now_timestamp_formatted();

    let peer = Peer {
        name: agent_peer_name,
        address: agent_peer_vpn_internal_address,
        endpoint: EnabledValue {
            enabled: true,
            value: agent_peer_vpn_endpoint,
        },
        kind: agent_peer_kind,
        icon: EnabledValue {
            enabled: agent_peer_icon_enabled,
            value: agent_peer_icon_src,
        },
        private_key: wg_generate_key(),
        created_at: now.clone(),
        updated_at: now.clone(),
        dns: EnabledValue {
            enabled: agent_peer_dns_enabled,
            value: agent_peer_dns_server.clone(),
        },
        mtu: EnabledValue {
            enabled: agent_peer_mtu_enabled,
            value: agent_peer_mtu_value,
        },
        scripts: Scripts {
            pre_up: agent_peer_script_pre_up_lines,
            post_up: agent_peer_script_post_up_lines,
            pre_down: agent_peer_script_pre_down_lines,
            post_down: agent_peer_script_post_down_lines,
        },
    };

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
                    tls_cert: agent_web_https_tls_cert.into(),
                    tls_key: agent_web_https_tls_key.into(),
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
                utility: agent_firewall_utility.into(),
                gateway: agent_firewall_gateway,
            },
        },
        network: Network {
            identifier: network_identifier,
            subnet: network_subnet.clone(),
            this_peer: peer_id.clone(),
            peers: {
                let mut map = HashMap::new();
                map.insert(peer_id.clone(), peer);
                map
            },
            connections: HashMap::new(),
            reservations: HashMap::new(),
            updated_at: now,
            defaults: Defaults {
                peer: DefaultPeer {
                    endpoint: EnabledValue {
                        enabled: false,
                        value: "".into(),
                    },
                    kind: default_peer_kind,
                    icon: EnabledValue {
                        enabled: default_peer_icon_enabled,
                        value: default_peer_icon_src,
                    },
                    dns: EnabledValue {
                        enabled: default_peer_dns_enabled,
                        value: default_peer_dns_server,
                    },
                    mtu: EnabledValue {
                        enabled: default_peer_mtu_enabled,
                        value: default_peer_mtu_value,
                    },
                    scripts: Scripts {
                        pre_up: default_peer_script_pre_up_lines,
                        post_up: default_peer_script_post_up_lines,
                        pre_down: default_peer_script_pre_down_lines,
                        post_down: default_peer_script_post_down_lines,
                    },
                },
                connection: DefaultConnection {
                    persistent_keepalive: EnabledValue {
                        enabled: default_connection_persistent_keepalive_enabled,
                        value: default_connection_persistent_keepalive_period,
                    },
                },
            },
        },
    };

    conf::util::set_config(&mut config).expect("Failed to write config.yml");
    println!(
        "✅ Configuration saved to {}",
        WG_QUICKRS_CONFIG_FILE.get().unwrap().display()
    );

    ExitCode::SUCCESS
}
