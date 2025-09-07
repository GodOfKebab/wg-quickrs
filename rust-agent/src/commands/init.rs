use crate::commands::helpers;
use crate::conf::util::ConfUtilError;
use crate::wireguard::cmd::get_public_private_keys;
use crate::{WG_RUSTEZE_CONFIG_FOLDER, conf};
use dialoguer;
use get_if_addrs::{Interface, get_if_addrs};
use ipnetwork::IpNetwork;
use rust_cli::InitOptions;
use rust_wasm::types::{
    Agent, AgentFirewall, AgentVpn, AgentWeb, AgentWebHttp, AgentWebHttps, Config,
    DefaultConnection, DefaultPeer, Defaults, EnabledValue, Network, Password, Peer, Scripts,
};
use rust_wasm::validation::{CheckResult, FieldValue, check_field};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};
use uuid::Uuid;

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
fn primary_ip_interface() -> Option<Interface> {
    match get_if_addrs()
        .unwrap()
        .into_iter()
        .find(|a| !a.is_loopback() && a.ip().is_ipv4())
    {
        Some(addr) => Some(addr),
        None => {
            log::error!("No valid network interface found");
            None
        }
    }
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

fn find_firewall_utility() -> Option<String> {
    let candidates = ["iptables", "pfctl", "nft"];

    for prog in candidates {
        if let Some(path) = find_in_path(prog) {
            return path.to_str().map(|s| s.to_string());
        }
    }

    None
}

fn find_first_cert_server() -> (Option<String>, Option<String>) {
    let config_folder = WG_RUSTEZE_CONFIG_FOLDER.get().unwrap();
    let servers_folder = config_folder.join("certs/servers");

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
        format!("\t[ {}/25]", step)
    } else {
        format!("\t[{}/25]", step)
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

/// Helper to prompt a value with optional default
// fn prompt<T: std::str::FromStr + ToString>(msg: &str, default: Option<T>) -> T {
//     let input = if let Some(d) = default {
//         dialoguer::Input::new()
//             .with_prompt(msg.to_string())
//             .default(d.to_string())
//             .interact_text()
//     } else {
//         dialoguer::Input::new().with_prompt(msg).interact_text()
//     };
//
//     input.unwrap().parse().ok().unwrap()
// }
enum FieldType {
    String,
    EnabledValue,
    Pass,
}
fn prompt<T: std::str::FromStr + ToString>(
    field_name: &str,
    field_type: FieldType,
    msg: &str,
    default: Option<T>,
) -> T {
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
                let result = match field_type {
                    FieldType::String => check_field(
                        field_name,
                        &FieldValue {
                            str: value.clone(),
                            enabled_value: EnabledValue {
                                enabled: false,
                                value: String::new(),
                            },
                        },
                    ),
                    FieldType::EnabledValue => check_field(
                        field_name,
                        &FieldValue {
                            str: String::new(),
                            enabled_value: EnabledValue {
                                enabled: true,
                                value: value.clone(),
                            },
                        },
                    ),
                    FieldType::Pass => CheckResult {
                        status: true,
                        msg: "".to_string(),
                    },
                };

                if result.status {
                    if let Ok(parsed) = value.parse::<T>() {
                        return parsed;
                    } else {
                        eprintln!("Parsing failed. Try again.");
                    }
                } else {
                    eprintln!("ERROR: {}", result.msg);
                }
            }
            Err(_) => {
                eprintln!("Error reading input, please try again.");
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
    field_type: FieldType,
    cli_value: Option<T>,
    cli_option: &str,
    description: &str,
    condition: bool,
    default_value: Option<T>,
) -> T {
    let step_str = step_str(step);
    match cli_value {
        Some(v) => {
            println!(
                "{} Using {} from CLI option '{}': {}",
                step_str, description, cli_option, v
            );
            v
        }
        None => match cli_no_prompt {
            Some(true) => {
                if condition {
                    Default::default()
                } else {
                    panic!("Error: CLI option '{}' is not set", description)
                }
            }
            _ => {
                if condition {
                    prompt(
                        field_name,
                        field_type,
                        &format!("{} {} (CLI option '{}')", step_str, description, cli_option),
                        default_value,
                    )
                } else {
                    Default::default()
                }
            }
        },
    }
}

/// Macro to handle paired "enable + value" options
#[macro_export]
macro_rules! get_init_pair_option {
    (
        $cli_no_prompt:expr,
        $step:expr,
        $field_name:expr,
        $field_type:expr,
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
            $field_type,
            $cli_value,
            $cli_value_option,
            $description_value,
            enabled,
            $default_value,
        );

        (enabled, value)
    }};
}

pub(crate) fn initialize_agent(init_opts: &InitOptions) -> ExitCode {
    if let Err(ConfUtilError::Read(..)) = conf::util::get_config() {
    } else {
        log::error!("wg-rusteze rust-agent is already initialized.");
        return ExitCode::FAILURE;
    }
    log::info!("Initializing wg-rusteze rust-agent...");

    println!("[general network settings 1-2/25]");
    // [1/25] --network-identifier
    let network_identifier = get_init_enabled_value_option(
        init_opts.no_prompt,
        1,
        "",
        FieldType::Pass,
        init_opts.network_identifier.clone(),
        INIT_FLAGS[0],
        INIT_HELPS[0],
        true,
        Some("wg-rusteze".into()),
    );

    // [2/25] --network-subnet
    let network_subnet = get_init_enabled_value_option(
        init_opts.no_prompt,
        2,
        "",
        FieldType::Pass,
        init_opts.network_subnet.clone(),
        INIT_FLAGS[1],
        INIT_HELPS[1],
        true,
        Some("10.0.34.0/24".into()),
    );

    println!("[general network settings complete]");
    println!("[agent settings 3-18/25]");

    let iface_opt = primary_ip_interface();
    let iface_name = iface_opt.as_ref().map(|iface| iface.name.clone());
    let iface_ip = iface_opt.map(|iface| iface.ip().to_string());

    // [3/25] --agent-web-address
    let agent_web_address = get_init_enabled_value_option(
        init_opts.no_prompt,
        3,
        "address",
        FieldType::String,
        init_opts.agent_web_address.clone(),
        INIT_FLAGS[2],
        INIT_HELPS[2],
        true,
        iface_ip.clone(),
    );

    // [4/25] --agent_web_http_enabled & --agent_web_http_port
    let (agent_web_http_enabled, agent_web_http_port) = get_init_pair_option!(
        init_opts.no_prompt,
        4,
        "",
        FieldType::Pass,
        init_opts.agent_web_http_enabled,
        init_opts.agent_web_http_port,
        INIT_FLAGS[3],
        INIT_FLAGS[4],
        INIT_HELPS[3],
        format!("\t{}", INIT_HELPS[4]).as_str(),
        true,
        Some(80)
    );

    // [5/25] --agent_web_https_enabled & --agent_web_https_port
    let (agent_web_https_enabled, agent_web_https_port) = get_init_pair_option!(
        init_opts.no_prompt,
        5,
        "",
        FieldType::Pass,
        init_opts.agent_web_https_enabled,
        init_opts.agent_web_https_port,
        INIT_FLAGS[5],
        INIT_FLAGS[6],
        INIT_HELPS[5],
        format!("\t{}", INIT_HELPS[6]).as_str(),
        true,
        Some(443)
    );

    let (option_cert, option_key) = find_first_cert_server();

    // [5/25] --agent_web_https_tls_cert
    let agent_web_https_tls_cert = get_init_enabled_value_option(
        init_opts.no_prompt,
        5,
        "",
        FieldType::Pass,
        init_opts
            .agent_web_https_tls_cert
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[7],
        format!("\t{}", INIT_HELPS[7]).as_str(),
        agent_web_https_enabled,
        option_cert,
    );

    // [5/25] --agent_web_https_tls_key
    let agent_web_https_tls_key = get_init_enabled_value_option(
        init_opts.no_prompt,
        5,
        "",
        FieldType::Pass,
        init_opts
            .agent_web_https_tls_key
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[8],
        format!("\t{}", INIT_HELPS[8]).as_str(),
        agent_web_https_enabled,
        option_key,
    );

    // [6/25] --agent-enable-web-password
    let agent_web_password_enabled = get_init_bool_option(
        init_opts.no_prompt,
        6,
        init_opts.agent_web_password_enabled,
        INIT_FLAGS[9],
        INIT_HELPS[9],
        true,
    );
    // [6/25] --agent-web-password
    let agent_web_password = match init_opts.agent_web_password.clone() {
        Some(v) => {
            println!(
                "{}  Using password for the web server from CLI argument: ***hidden***",
                step_str(6)
            );
            v.clone()
        }
        _ => match init_opts.no_prompt {
            Some(true) => {
                if agent_web_password_enabled {
                    "".into()
                } else {
                    panic!("Error: {} option is not set", INIT_FLAGS[10])
                }
            }
            _ => {
                if agent_web_password_enabled {
                    dialoguer::Password::new()
                        .with_prompt(format!("{} \t{}", step_str(6), INIT_HELPS[10]))
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

    // [7/25] --agent_vpn_enabled & --agent_vpn_port
    let (agent_vpn_enabled, agent_vpn_port) = get_init_pair_option!(
        init_opts.no_prompt,
        7,
        "",
        FieldType::Pass,
        init_opts.agent_vpn_enabled,
        init_opts.agent_vpn_port,
        INIT_FLAGS[11],
        INIT_FLAGS[12],
        INIT_HELPS[11],
        format!("\t{}", INIT_HELPS[12]).as_str(),
        true,
        Some(51820)
    );

    // [7/25] --agent_vpn_gateway
    let agent_vpn_gateway = get_init_enabled_value_option(
        init_opts.no_prompt,
        7,
        "",
        FieldType::Pass,
        init_opts.agent_vpn_gateway.clone(),
        INIT_FLAGS[13],
        format!("\t{}", INIT_HELPS[13]).as_str(),
        agent_vpn_enabled,
        iface_name,
    );

    // [8/25] --agent_firewall_enabled & --agent_firewall_utility
    let (agent_firewall_enabled, agent_firewall_utility) = get_init_pair_option!(
        init_opts.no_prompt,
        8,
        "",
        FieldType::Pass,
        init_opts.agent_firewall_enabled,
        init_opts
            .agent_firewall_utility
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        INIT_FLAGS[14],
        INIT_FLAGS[15],
        INIT_HELPS[14],
        format!("\t{}", INIT_HELPS[15]).as_str(),
        true,
        find_firewall_utility()
    ); // TODO: auto-detect

    // [9/25] --agent-peer-name
    let agent_peer_name = get_init_enabled_value_option(
        init_opts.no_prompt,
        9,
        "name",
        FieldType::String,
        init_opts.agent_peer_name.clone(),
        INIT_FLAGS[16],
        INIT_HELPS[16],
        true,
        Some("wg-rusteze-host".into()),
    );

    // [10/25] --agent_peer_vpn_public_address
    let agent_peer_vpn_public_address = get_init_enabled_value_option(
        init_opts.no_prompt,
        10,
        "",
        FieldType::Pass,
        init_opts.agent_peer_vpn_public_address.clone(),
        INIT_FLAGS[17],
        INIT_HELPS[17],
        true,
        iface_ip,
    );

    // [11/25] --agent-public-vpn-port
    let agent_peer_vpn_public_port = get_init_enabled_value_option(
        init_opts.no_prompt,
        11,
        "",
        FieldType::Pass,
        init_opts.agent_peer_vpn_public_port,
        INIT_FLAGS[18],
        INIT_HELPS[18],
        true,
        Some(51820),
    ); // TODO: change to endpoint

    // [12/25] --agent_peer_vpn_internal_address
    let agent_peer_vpn_internal_address = get_init_enabled_value_option(
        init_opts.no_prompt,
        12,
        "address",
        FieldType::String,
        init_opts.agent_peer_vpn_internal_address.clone(),
        INIT_FLAGS[19],
        INIT_HELPS[19],
        true,
        Some(first_ip(&network_subnet)),
    );

    // [13/25] --agent_peer_dns_enabled & --agent_peer_dns_server
    let (agent_peer_dns_enabled, agent_peer_dns_server) = get_init_pair_option!(
        init_opts.no_prompt,
        13,
        "dns",
        FieldType::EnabledValue,
        init_opts.agent_peer_dns_enabled,
        init_opts.agent_peer_dns_server.clone(),
        INIT_FLAGS[20],
        INIT_FLAGS[21],
        INIT_HELPS[20],
        format!("\t{}", INIT_HELPS[21]).as_str(),
        true,
        Some("1.1.1.1".into())
    );

    // [14/25] --agent_peer_mtu_enabled & --agent_peer_mtu_value
    let (agent_peer_mtu_enabled, agent_peer_mtu_value) = get_init_pair_option!(
        init_opts.no_prompt,
        14,
        "mtu",
        FieldType::EnabledValue,
        init_opts.agent_peer_mtu_enabled,
        init_opts.agent_peer_mtu_value.clone(),
        INIT_FLAGS[22],
        INIT_FLAGS[23],
        INIT_HELPS[22],
        format!("\t{}", INIT_HELPS[23]).as_str(),
        false,
        Some("1420".into())
    );

    // [15/25] --agent_peer_script_pre_up_enabled & --agent_peer_script_pre_up_line
    let (agent_peer_script_pre_up_enabled, agent_peer_script_pre_up_line) = get_init_pair_option!(
        init_opts.no_prompt,
        15,
        "script",
        FieldType::EnabledValue,
        init_opts.agent_peer_script_pre_up_enabled,
        init_opts.agent_peer_script_pre_up_line.clone(),
        INIT_FLAGS[24],
        INIT_FLAGS[25],
        INIT_HELPS[24],
        format!("\t{}", INIT_HELPS[25]).as_str(),
        false,
        Some("".into())
    );

    // [16/25] --agent_peer_script_post_up_enabled & --agent_peer_script_post_up_line
    let (agent_peer_script_post_up_enabled, agent_peer_script_post_up_line) = get_init_pair_option!(
        init_opts.no_prompt,
        16,
        "script",
        FieldType::EnabledValue,
        init_opts.agent_peer_script_post_up_enabled,
        init_opts.agent_peer_script_post_up_line.clone(),
        INIT_FLAGS[26],
        INIT_FLAGS[27],
        INIT_HELPS[26],
        format!("\t{}", INIT_HELPS[27]).as_str(),
        false,
        Some("".into())
    );

    // [17/25] --agent_peer_script_pre_down_enabled & --agent_peer_script_pre_down_line
    let (agent_peer_script_pre_down_enabled, agent_peer_script_pre_down_line) = get_init_pair_option!(
        init_opts.no_prompt,
        17,
        "script",
        FieldType::EnabledValue,
        init_opts.agent_peer_script_pre_down_enabled,
        init_opts.agent_peer_script_pre_down_line.clone(),
        INIT_FLAGS[28],
        INIT_FLAGS[29],
        INIT_HELPS[28],
        format!("\t{}", INIT_HELPS[29]).as_str(),
        false,
        Some("".into())
    );

    // [18/25] --agent_peer_script_post_down_enabled & --agent_peer_script_post_down_line
    let (agent_peer_script_post_down_enabled, agent_peer_script_post_down_line) = get_init_pair_option!(
        init_opts.no_prompt,
        18,
        "script",
        FieldType::EnabledValue,
        init_opts.agent_peer_script_post_down_enabled,
        init_opts.agent_peer_script_post_down_line.clone(),
        INIT_FLAGS[30],
        INIT_FLAGS[31],
        INIT_HELPS[30],
        format!("\t{}", INIT_HELPS[31]).as_str(),
        false,
        Some("".into())
    );

    println!("[agent settings complete]");
    println!("[new peer/connection default settings 19-25/25]");

    // [19/25] --default_peer_dns_enabled & --default_peer_dns_server
    let (default_peer_dns_enabled, default_peer_dns_server) = get_init_pair_option!(
        init_opts.no_prompt,
        19,
        "dns",
        FieldType::EnabledValue,
        init_opts.default_peer_dns_enabled,
        init_opts.default_peer_dns_server.clone(),
        INIT_FLAGS[32],
        INIT_FLAGS[33],
        INIT_HELPS[32],
        format!("\t{}", INIT_HELPS[33]).as_str(),
        true,
        Some("1.1.1.1".into())
    );

    // [20/25] --default_peer_mtu_enabled & --default_peer_mtu_value
    let (default_peer_mtu_enabled, default_peer_mtu_value) = get_init_pair_option!(
        init_opts.no_prompt,
        20,
        "mtu",
        FieldType::EnabledValue,
        init_opts.default_peer_mtu_enabled,
        init_opts.default_peer_mtu_value.clone(),
        INIT_FLAGS[34],
        INIT_FLAGS[35],
        INIT_HELPS[34],
        format!("\t{}", INIT_HELPS[35]).as_str(),
        false,
        Some("1420".into())
    );

    // [21/25] --default_peer_script_pre_up_enabled & --default_peer_script_pre_up_line
    let (default_peer_script_pre_up_enabled, default_peer_script_pre_up_line) = get_init_pair_option!(
        init_opts.no_prompt,
        21,
        "script",
        FieldType::EnabledValue,
        init_opts.default_peer_script_pre_up_enabled,
        init_opts.default_peer_script_pre_up_line.clone(),
        INIT_FLAGS[36],
        INIT_FLAGS[37],
        INIT_HELPS[36],
        format!("\t{}", INIT_HELPS[37]).as_str(),
        false,
        Some("".into())
    );

    // [22/25] --default_peer_script_post_up_enabled & --default_peer_script_post_up_line
    let (default_peer_script_post_up_enabled, default_peer_script_post_up_line) = get_init_pair_option!(
        init_opts.no_prompt,
        22,
        "script",
        FieldType::EnabledValue,
        init_opts.default_peer_script_post_up_enabled,
        init_opts.default_peer_script_post_up_line.clone(),
        INIT_FLAGS[38],
        INIT_FLAGS[39],
        INIT_HELPS[38],
        format!("\t{}", INIT_HELPS[39]).as_str(),
        false,
        Some("".into())
    );

    // [23/25] --default_peer_script_pre_down_enabled & --default_peer_script_pre_down_line
    let (default_peer_script_pre_down_enabled, default_peer_script_pre_down_line) = get_init_pair_option!(
        init_opts.no_prompt,
        23,
        "script",
        FieldType::EnabledValue,
        init_opts.default_peer_script_pre_down_enabled,
        init_opts.default_peer_script_pre_down_line.clone(),
        INIT_FLAGS[40],
        INIT_FLAGS[41],
        INIT_HELPS[40],
        format!("\t{}", INIT_HELPS[41]).as_str(),
        false,
        Some("".into())
    );

    // [24/25] --default_peer_script_post_down_enabled & --default_peer_script_post_down_line
    let (default_peer_script_post_down_enabled, default_peer_script_post_down_line) = get_init_pair_option!(
        init_opts.no_prompt,
        24,
        "script",
        FieldType::EnabledValue,
        init_opts.default_peer_script_post_down_enabled,
        init_opts.default_peer_script_post_down_line.clone(),
        INIT_FLAGS[42],
        INIT_FLAGS[43],
        INIT_HELPS[42],
        format!("\t{}", INIT_HELPS[43]).as_str(),
        false,
        Some("".into())
    );

    // [25/25] --default_connection_persistent_keepalive_enabled & --default_connection_persistent_keepalive_period
    let (
        default_connection_persistent_keepalive_enabled,
        default_connection_persistent_keepalive_period,
    ) = get_init_pair_option!(
        init_opts.no_prompt,
        25,
        "persistent_keepalive",
        FieldType::EnabledValue,
        init_opts.default_connection_persistent_keepalive_enabled,
        init_opts
            .default_connection_persistent_keepalive_period
            .clone(),
        INIT_FLAGS[44],
        INIT_FLAGS[45],
        INIT_HELPS[44],
        format!("\t{}", INIT_HELPS[45]).as_str(),
        true,
        Some("25".into())
    );

    println!("[new peer/connection default settings complete]");

    println!(
        "✅ This was all the information required to initialize the rust-agent. Finalizing the configuration..."
    );

    let peer_id = Uuid::new_v4().to_string();
    let pub_priv_key = get_public_private_keys().unwrap();
    let now = conf::timestamp::get_now_timestamp_formatted();

    let peer = Peer {
        name: agent_peer_name,
        address: agent_peer_vpn_internal_address,
        public_key: pub_priv_key
            .get("public_key")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .parse()
            .unwrap(),
        private_key: pub_priv_key
            .get("private_key")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .parse()
            .unwrap(),
        created_at: now.clone(),
        updated_at: now.clone(),
        endpoint: EnabledValue {
            enabled: true,
            value: format!("{agent_peer_vpn_public_address}:{agent_peer_vpn_public_port}"),
        },
        dns: EnabledValue {
            enabled: agent_peer_dns_enabled,
            value: agent_peer_dns_server.clone(),
        },
        mtu: EnabledValue {
            enabled: agent_peer_mtu_enabled,
            value: agent_peer_mtu_value,
        },
        scripts: Scripts {
            pre_up: EnabledValue {
                enabled: agent_peer_script_pre_up_enabled,
                value: agent_peer_script_pre_up_line,
            },
            post_up: EnabledValue {
                enabled: agent_peer_script_post_up_enabled,
                value: agent_peer_script_post_up_line,
            },
            pre_down: EnabledValue {
                enabled: agent_peer_script_pre_down_enabled,
                value: agent_peer_script_pre_down_line,
            },
            post_down: EnabledValue {
                enabled: agent_peer_script_post_down_enabled,
                value: agent_peer_script_post_down_line,
            },
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
                gateway: agent_vpn_gateway,
                port: agent_vpn_port,
            },
            firewall: AgentFirewall {
                enabled: agent_firewall_enabled,
                utility: agent_firewall_utility.into(),
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
            leases: vec![],
            updated_at: now,
            defaults: Defaults {
                peer: DefaultPeer {
                    endpoint: EnabledValue {
                        enabled: false,
                        value: "".into(),
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
                        pre_up: EnabledValue {
                            enabled: default_peer_script_pre_up_enabled,
                            value: default_peer_script_pre_up_line,
                        },
                        post_up: EnabledValue {
                            enabled: default_peer_script_post_up_enabled,
                            value: default_peer_script_post_up_line,
                        },
                        pre_down: EnabledValue {
                            enabled: default_peer_script_pre_down_enabled,
                            value: default_peer_script_pre_down_line,
                        },
                        post_down: EnabledValue {
                            enabled: default_peer_script_post_down_enabled,
                            value: default_peer_script_post_down_line,
                        },
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
    println!("✅ Configuration saved to `config.yml`.");

    ExitCode::SUCCESS
}
