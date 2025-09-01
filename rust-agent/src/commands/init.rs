use crate::commands::helpers;
use crate::conf;
use crate::conf::util::ConfUtilError;
use crate::wireguard::cmd::get_public_private_keys;
use dialoguer::{Confirm, Input};
use get_if_addrs::get_if_addrs;
use ipnetwork::IpNetwork;
use rust_wasm::types::{
    Agent, AgentVpn, AgentWeb, Config, DefaultConnection, DefaultPeer, Defaults, EnabledValue,
    Network, Password, Peer, Scripts,
};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::ExitCode;
use uuid::Uuid;

// Helper to prompt a value with optional default
fn prompt<T: std::str::FromStr>(msg: &str, default: Option<&str>) -> T {
    let input = if let Some(d) = default {
        Input::new()
            .with_prompt(format!("{msg} (e.g. {d})"))
            .default(d.to_string())
            .interact_text()
    } else {
        Input::new().with_prompt(msg).interact_text()
    };

    input.unwrap().parse().ok().unwrap()
}

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
fn primary_ip() -> Option<String> {
    match get_if_addrs()
        .unwrap()
        .into_iter()
        .find(|a| !a.is_loopback() && a.ip().is_ipv4())
    {
        Some(addr) => Some(addr.ip().to_string()),
        None => {
            log::error!("No valid network interface found");
            None
        }
    }
}

/// Format step string with padding if single-digit
fn step_str(step: usize) -> String {
    if step < 10 {
        format!("\t[ {}/22]", step)
    } else {
        format!("\t[{}/22]", step)
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
            _ => Confirm::new()
                .with_prompt(format!(
                    "{} Enable {} (CLI option '{}')?",
                    step_str, description, cli_option
                ))
                .default(default)
                .interact()
                .unwrap(),
        },
    }
}

/// Handle enabled value options
fn get_init_enabled_value_option<T: std::str::FromStr + std::fmt::Display + Clone + Default>(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_value: Option<T>,
    cli_option: &str,
    description: &str,
    condition: bool,
    default_value: Option<&str>,
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

/// Handle paired "enable + value" options
fn get_init_pair_option<T: std::str::FromStr + std::fmt::Display + Clone + Default>(
    cli_no_prompt: Option<bool>,
    step: usize,
    cli_enable: Option<bool>,
    cli_value: Option<T>,
    cli_enable_option: &str,
    cli_value_option: &str,
    description_enable: &str,
    description_value: &str,
    default_enable: bool,
    default_value: Option<&str>,
) -> (bool, T) {
    let enabled = get_init_bool_option(
        cli_no_prompt,
        step,
        cli_enable,
        cli_enable_option,
        description_enable,
        default_enable,
    );

    let value = get_init_enabled_value_option(
        cli_no_prompt,
        step,
        cli_value,
        cli_value_option,
        description_value,
        enabled,
        default_value,
    );

    (enabled, value)
}

pub(crate) fn initialize_agent(
    cli_network_identifier: Option<String>,
    cli_network_subnet: Option<String>,
    cli_agent_peer_name: Option<String>,
    cli_agent_local_address: Option<String>,
    cli_agent_local_web_port: Option<u16>,
    cli_agent_local_vpn_port: Option<u16>,
    cli_agent_public_address: Option<String>,
    cli_agent_public_vpn_port: Option<u16>,
    cli_agent_internal_vpn_address: Option<String>,
    cli_agent_use_tls: Option<bool>,
    cli_agent_enable_web_password: Option<bool>,
    cli_agent_web_password: Option<String>,
    cli_agent_enable_dns: Option<bool>,
    cli_agent_dns_server: Option<String>,
    cli_agent_enable_mtu: Option<bool>,
    cli_agent_mtu_value: Option<String>,
    cli_agent_enable_script_pre_up: Option<bool>,
    cli_agent_script_pre_up_line: Option<String>,
    cli_agent_enable_script_post_up: Option<bool>,
    cli_agent_script_post_up_line: Option<String>,
    cli_agent_enable_script_pre_down: Option<bool>,
    cli_agent_script_pre_down_line: Option<String>,
    cli_agent_enable_script_post_down: Option<bool>,
    cli_agent_script_post_down_line: Option<String>,
    cli_default_enable_dns: Option<bool>,
    cli_default_dns_server: Option<String>,
    cli_default_enable_mtu: Option<bool>,
    cli_default_mtu_value: Option<String>,
    cli_default_enable_script_pre_up: Option<bool>,
    cli_default_script_pre_up_line: Option<String>,
    cli_default_enable_script_post_up: Option<bool>,
    cli_default_script_post_up_line: Option<String>,
    cli_default_enable_script_pre_down: Option<bool>,
    cli_default_script_pre_down_line: Option<String>,
    cli_default_enable_script_post_down: Option<bool>,
    cli_default_script_post_down_line: Option<String>,
    cli_default_enable_persistent_keepalive: Option<bool>,
    cli_default_persistent_keepalive_period: Option<String>,
    cli_no_prompt: Option<bool>,
) -> ExitCode {
    if let Err(ConfUtilError::Read(..)) = conf::util::get_config() {
    } else {
        log::error!("wg-rusteze rust-agent is already initialized.");
        return ExitCode::FAILURE;
    }
    log::info!("Initializing wg-rusteze rust-agent...");

    println!("[general network settings 1-2/22]");
    // [1/22] --network-identifier
    let network_identifier = get_init_enabled_value_option(
        cli_no_prompt,
        1,
        cli_network_identifier,
        "--network-identifier",
        "Enter VPN network's identifier",
        true,
        Some("wg-rusteze"),
    );

    // [2/22] --network-subnet
    let network_subnet = get_init_enabled_value_option(
        cli_no_prompt,
        2,
        cli_network_subnet,
        "--network-subnet",
        "Enter VPN network's CIDR subnet mask",
        true,
        Some("10.0.34.0/24"),
    );

    println!("[general network settings complete]");
    println!("[agent settings 3-15/22]");

    // [3/22] --agent-peer-name
    let agent_peer_name = get_init_enabled_value_option(
        cli_no_prompt,
        3,
        cli_agent_peer_name,
        "--agent-peer-name",
        "Enter agent's peer name",
        true,
        Some("wg-rusteze-host"),
    );

    // [4/22] --agent-local-address
    let agent_local_address = get_init_enabled_value_option(
        cli_no_prompt,
        4,
        cli_agent_local_address,
        "--agent-local-address",
        "Enter agent's local IPv4 address for the web server to bind",
        true,
        primary_ip().as_deref(),
    );

    // [5/22] --agent-local-web-port
    let agent_local_web_port = get_init_enabled_value_option(
        cli_no_prompt,
        5,
        cli_agent_local_web_port,
        "--agent-local-web-port",
        "Enter agent's local web port for the web server to bind",
        true,
        Some("8080"),
    );

    // [6/22] --agent-local-vpn-port
    let agent_local_vpn_port = get_init_enabled_value_option(
        cli_no_prompt,
        6,
        cli_agent_local_vpn_port,
        "--agent-local-vpn-port",
        "Enter agent's local vpn port for the vpn server to bind",
        true,
        Some("8080"),
    );

    // [7/22] --agent-public-address
    let agent_public_address = get_init_enabled_value_option(
        cli_no_prompt,
        7,
        cli_agent_public_address,
        "--agent-public-address",
        "Enter agent's publicly accessible IPv4 address to be used in the VPN endpoint advertisement",
        true,
        primary_ip().as_deref(),
    );

    // [8/22] --agent-public-vpn-port
    let agent_public_vpn_port = get_init_enabled_value_option(
        cli_no_prompt,
        8,
        cli_agent_public_vpn_port,
        "--agent-public-vpn-port",
        "Enter agent's publicly accessible port to be used in the VPN endpoint advertisement",
        true,
        Some("51820"),
    );

    // [7/22] --agent-internal-vpn-address
    let agent_internal_vpn_address = get_init_enabled_value_option(
        cli_no_prompt,
        7,
        cli_agent_internal_vpn_address,
        "--agent-internal-vpn-address",
        "Enter agent's internal IPv4 address for VPN network",
        true,
        Some(&*first_ip(&network_subnet)),
    );

    // [8/22] --agent-use-tls
    let agent_use_tls = get_init_bool_option(
        cli_no_prompt,
        8,
        cli_agent_use_tls,
        "--agent-use-tls",
        "TLS for this agent's web server",
        true,
    );

    // [9/22] --agent-enable-web-password
    let agent_enable_web_password = get_init_bool_option(
        cli_no_prompt,
        9,
        cli_agent_enable_web_password,
        "--agent-enable-web-password",
        "password for this agent's web server",
        true,
    );
    // [9/22] --agent-web-password
    let agent_web_password = match cli_agent_web_password {
        Some(v) => {
            println!(
                "{}  Using password for the web server from CLI argument: ***hidden***",
                step_str(9)
            );
            v.clone()
        }
        _ => match cli_no_prompt {
            Some(true) => {
                if agent_enable_web_password {
                    "".into()
                } else {
                    panic!("Error: --agent-web-password option is not set")
                }
            }
            _ => {
                if agent_enable_web_password {
                    dialoguer::Password::new()
                        .with_prompt(format!(
                            "{} \tEnter password for this agent's web server",
                            step_str(9)
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
            if agent_enable_web_password {
                p
            } else {
                "".into()
            }
        }
        Err(e) => {
            return e;
        }
    };

    // [10/22] --agent-enable-dns & --agent-dns-server
    let (agent_enable_dns, agent_dns_server) = get_init_pair_option(
        cli_no_prompt,
        10,
        cli_agent_enable_dns,
        cli_agent_dns_server,
        "--agent-enable-dns",
        "--agent-dns-server",
        "DNS server field for this agent",
        "\tEnter DNS server for this agent",
        true,
        Some("1.1.1.1"),
    );

    // [11/22] --agent-enable-mtu & --agent-mtu-value
    let (agent_enable_mtu, agent_mtu_value) = get_init_pair_option(
        cli_no_prompt,
        11,
        cli_agent_enable_mtu,
        cli_agent_mtu_value,
        "--agent-enable-mtu",
        "--agent-mtu-value",
        "MTU value field for this agent",
        "\tEnter MTU value for this agent",
        false,
        Some("1420"),
    );

    // [12/22] --agent-enable-script-pre-up & --agent-script-pre-up-line
    let (agent_enable_script_pre_up, agent_script_pre_up_line) = get_init_pair_option(
        cli_no_prompt,
        12,
        cli_agent_enable_script_pre_up,
        cli_agent_script_pre_up_line,
        "--agent-enable-script-pre-up",
        "--agent-script-pre-up-line",
        "PreUp scripting field for this agent",
        "\tEnter PreUp scripting line for this agent",
        false,
        Some("TODO"),
    );

    // [13/22] --agent-enable-script-post-up & --agent-script-post-up-line
    let (agent_enable_script_post_up, agent_script_post_up_line) = get_init_pair_option(
        cli_no_prompt,
        13,
        cli_agent_enable_script_post_up,
        cli_agent_script_post_up_line,
        "--agent-enable-script-post-up",
        "--agent-script-post-up-line",
        "PostUp scripting field for this agent",
        "\tEnter PostUp scripting line for this agent",
        false,
        Some("TODO"),
    );

    // [14/22] --agent-enable-script-pre-down & --agent-script-pre-down-line
    let (agent_enable_script_pre_down, agent_script_pre_down_line) = get_init_pair_option(
        cli_no_prompt,
        14,
        cli_agent_enable_script_pre_down,
        cli_agent_script_pre_down_line,
        "--agent-enable-script-pre-down",
        "--agent-script-pre-down-line",
        "PreDown scripting field for this agent",
        "\tEnter PreDown scripting line for this agent",
        false,
        Some("TODO"),
    );

    // [15/22] --agent-enable-script-post-down & --agent-script-post-down-line
    let (agent_enable_script_post_down, agent_script_post_down_line) = get_init_pair_option(
        cli_no_prompt,
        15,
        cli_agent_enable_script_post_down,
        cli_agent_script_post_down_line,
        "--agent-enable-script-post-down",
        "--agent-script-post-down-line",
        "PostDown scripting field for this agent",
        "\tEnter PostDown scripting line for this agent",
        false,
        Some("TODO"),
    );

    println!("[agent settings complete]");
    println!("[new peer/connection default settings 16-22/22]");

    // [16/22] --default-enable-dns & --default-dns-server
    let (default_enable_dns, default_dns_server) = get_init_pair_option(
        cli_no_prompt,
        16,
        cli_default_enable_dns,
        cli_default_dns_server,
        "--default-enable-dns",
        "--default-dns-server",
        "DNS field for new peers by default",
        "\tEnter DNS server for new peers by default",
        true,
        Some("1.1.1.1"),
    );

    // [17/22] --default-enable-mtu & --default-mtu-value
    let (default_enable_mtu, default_mtu_value) = get_init_pair_option(
        cli_no_prompt,
        17,
        cli_default_enable_mtu,
        cli_default_mtu_value,
        "--default-enable-mtu",
        "--default-mtu-value",
        "MTU field for new peers by default",
        "\tEnter MTU value for new peers by default",
        false,
        Some("1420"),
    );

    // [18/22] --default-enable-script-pre-up & --default-script-pre-up-line
    let (default_enable_script_pre_up, default_script_pre_up_line) = get_init_pair_option(
        cli_no_prompt,
        18,
        cli_default_enable_script_pre_up,
        cli_default_script_pre_up_line,
        "--default-enable-script-pre-up",
        "--default-script-pre-up-line",
        "PreUp scripting field for new peers by default",
        "\tEnter PreUp scripting line for new peers by default",
        false,
        Some("TODO"),
    );

    // [19/22] --default-enable-script-post-up & --default-script-post-up-line
    let (default_enable_script_post_up, default_script_post_up_line) = get_init_pair_option(
        cli_no_prompt,
        19,
        cli_default_enable_script_post_up,
        cli_default_script_post_up_line,
        "--default-enable-script-post-up",
        "--default-script-post-up-line",
        "PostUp scripting field for this default",
        "\tEnter PostUp scripting line for this default",
        false,
        Some("TODO"),
    );

    // [20/22] --default-enable-script-pre-down & --default-script-pre-down-line
    let (default_enable_script_pre_down, default_script_pre_down_line) = get_init_pair_option(
        cli_no_prompt,
        20,
        cli_default_enable_script_pre_down,
        cli_default_script_pre_down_line,
        "--default-enable-script-pre-down",
        "--default-script-pre-down-line",
        "PreDown scripting field for this default",
        "\tEnter PreDown scripting line for this default",
        false,
        Some("TODO"),
    );

    // [21/22] --default-enable-script-post-down & --default-script-post-down-line
    let (default_enable_script_post_down, default_script_post_down_line) = get_init_pair_option(
        cli_no_prompt,
        21,
        cli_default_enable_script_post_down,
        cli_default_script_post_down_line,
        "--default-enable-script-post-down",
        "--default-script-post-down-line",
        "PostDown scripting field for this default",
        "\tEnter PostDown scripting line for this default",
        false,
        Some("TODO"),
    );

    // [22/22] --default-enable-persistent-keepalive & --default-persistent-keepalive-period
    let (default_enable_persistent_keepalive, default_persistent_keepalive_period) =
        get_init_pair_option(
            cli_no_prompt,
            22,
            cli_default_enable_persistent_keepalive,
            cli_default_persistent_keepalive_period,
            "--default-enable-persistent-keepalive",
            "--default-persistent-keepalive-period",
            "PersistentKeepalive field for new connections by default",
            "\tEnter PersistentKeepalive period (seconds) for new connections by default",
            true,
            Some("25"),
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
        address: agent_internal_vpn_address,
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
            value: format!("{agent_public_address}:{agent_public_vpn_port}"),
        },
        dns: EnabledValue {
            enabled: agent_enable_dns,
            value: agent_dns_server.clone(),
        },
        mtu: EnabledValue {
            enabled: agent_enable_mtu,
            value: agent_mtu_value,
        },
        scripts: Scripts {
            pre_up: EnabledValue {
                enabled: agent_enable_script_pre_up,
                value: agent_script_pre_up_line,
            },
            post_up: EnabledValue {
                enabled: agent_enable_script_post_up,
                value: agent_script_post_up_line,
            },
            pre_down: EnabledValue {
                enabled: agent_enable_script_pre_down,
                value: agent_script_pre_down_line,
            },
            post_down: EnabledValue {
                enabled: agent_enable_script_post_down,
                value: agent_script_post_down_line,
            },
        },
    };

    let mut config = Config {
        agent: Agent {
            address: agent_local_address,
            web: AgentWeb {
                port: agent_local_web_port,
                use_tls: agent_use_tls,
                password: Password {
                    enabled: agent_enable_web_password,
                    hash: agent_web_password_hash,
                },
            },
            vpn: AgentVpn {
                port: agent_local_vpn_port,
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
                        enabled: default_enable_dns,
                        value: default_dns_server,
                    },
                    mtu: EnabledValue {
                        enabled: default_enable_mtu,
                        value: default_mtu_value,
                    },
                    scripts: Scripts {
                        pre_up: EnabledValue {
                            enabled: default_enable_script_pre_up,
                            value: default_script_pre_up_line,
                        },
                        post_up: EnabledValue {
                            enabled: default_enable_script_post_up,
                            value: default_script_post_up_line,
                        },
                        pre_down: EnabledValue {
                            enabled: default_enable_script_pre_down,
                            value: default_script_pre_down_line,
                        },
                        post_down: EnabledValue {
                            enabled: default_enable_script_post_down,
                            value: default_script_post_down_line,
                        },
                    },
                },
                connection: DefaultConnection {
                    persistent_keepalive: EnabledValue {
                        enabled: default_enable_persistent_keepalive,
                        value: default_persistent_keepalive_period,
                    },
                },
            },
        },
    };

    conf::util::set_config(&mut config).expect("Failed to write config.yml");
    println!("✅ Configuration saved to `config.yml`.");

    ExitCode::SUCCESS
}
