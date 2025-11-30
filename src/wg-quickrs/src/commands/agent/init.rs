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
use rand::{RngCore};
use thiserror::Error;
use uuid::Uuid;
use wg_quickrs_lib::validation::agent::{parse_and_validate_fw_gateway, parse_and_validate_ipv4_address, parse_and_validate_port, parse_and_validate_tls_file, parse_and_validate_fw_utility, parse_and_validate_wg_tool, parse_and_validate_wg_userspace_binary};
use wg_quickrs_lib::validation::helpers::{firewall_utility_options, wg_tool_options, wg_userspace_options};
use wg_quickrs_lib::validation::network::{parse_and_validate_amnezia_h, parse_and_validate_amnezia_jc, parse_and_validate_amnezia_jmax, parse_and_validate_amnezia_jmin, parse_and_validate_amnezia_s1, parse_and_validate_amnezia_s1_s2, parse_and_validate_conn_persistent_keepalive_period, parse_and_validate_ipv4_subnet, parse_and_validate_network_name, parse_and_validate_peer_address, parse_and_validate_peer_endpoint, parse_and_validate_peer_icon_src, parse_and_validate_peer_kind, parse_and_validate_peer_mtu_value, parse_and_validate_peer_name, validate_amnezia_enabled, validate_amnezia_jmin_jmax};
use crate::commands::helpers::*;
use crate::conf::util::ConfUtilError;

include!(concat!(env!("OUT_DIR"), "/init_options_generated.rs"));

#[derive(Error, Debug)]
pub enum AgentInitError {
    #[error("wg-quickrs is already initialized at \"{0}\"")]
    AlreadyInitialized(String),
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    ConfUtil(#[from] ConfUtilError),
}

// Get network interfaces of the current machine
pub fn get_interfaces() -> Vec<Interface> {
    get_if_addrs()
        .unwrap_or_else(|e| {
            log::warn!("Failed to get network interfaces: {}", e);
            Vec::new()
        })
        .into_iter()
        .filter(|a| !a.is_loopback() && a.ip().is_ipv4())
        .collect()
}

// Get network interface recommendation for the current machine
pub fn recommend_interface() -> Option<Interface> {
    default_net::get_default_interface()
        .ok()
        .and_then(|gw| get_interfaces().into_iter().find(|i| gw.name == i.name))
        .or_else(|| {
            log::warn!("Failed to get default gateway, falling back to first interface");
            get_interfaces().into_iter().next()
        })
}

// Generate recommended HTTP/HTTPS firewall scripts based on utility
// HTTP/HTTPS only need pre_up and post_down (not post_up or pre_down)
fn generate_http_firewall_scripts(utility: &str) -> HttpScripts {
    let utility_name = std::path::Path::new(utility)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if utility_name == "iptables" {
        // PreUp scripts - 1 item to allow HTTP/HTTPS port
        let pre_up = vec![
            Script {
                enabled: true,
                script: "iptables -I INPUT -p tcp --dport \"$PORT\" -j ACCEPT;".to_string(),
            },
        ];

        // PostDown scripts - 1 item to remove HTTP/HTTPS port rule
        let post_down = vec![
            Script {
                enabled: true,
                script: "iptables -D INPUT -p tcp --dport \"$PORT\" -j ACCEPT;".to_string(),
            },
        ];

        HttpScripts {
            pre_up,
            post_down,
        }
    } else {
        // Skip pfctl
        HttpScripts::default()
    }
}

// Generate recommended firewall scripts based on utility
// Only GATEWAY is substituted during init, all other variables are runtime variables
fn generate_firewall_scripts(utility: &str, gateway: &str) -> Scripts {
    let utility_name = std::path::Path::new(utility)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if utility_name == "iptables" {
        // PostUp scripts - 5 separate items
        let post_up = vec![
            Script {
                enabled: true,
                script: format!("iptables -t nat -I POSTROUTING -s \"$WG_SUBNET\" -o \"{}\" -j MASQUERADE;", gateway),
            },
            Script {
                enabled: true,
                script: "iptables -I INPUT -p udp -m udp --dport \"$WG_PORT\" -j ACCEPT;".to_string(),
            },
            Script {
                enabled: true,
                script: "iptables -I FORWARD -i \"$WG_INTERFACE\" -j ACCEPT;".to_string(),
            },
            Script {
                enabled: true,
                script: "iptables -I FORWARD -o \"$WG_INTERFACE\" -j ACCEPT;".to_string(),
            },
            #[cfg(not(feature = "docker"))]
            Script {
                enabled: true,
                script: "sysctl -w net.ipv4.ip_forward=1;".to_string(),
            },
        ];

        // PostDown scripts - 5 separate items
        let post_down = vec![
            Script {
                enabled: true,
                script: format!("iptables -t nat -D POSTROUTING -s \"$WG_SUBNET\" -o \"{}\" -j MASQUERADE;", gateway),
            },
            Script {
                enabled: true,
                script: "iptables -D INPUT -p udp -m udp --dport \"$WG_PORT\" -j ACCEPT;".to_string(),
            },
            Script {
                enabled: true,
                script: "iptables -D FORWARD -i \"$WG_INTERFACE\" -j ACCEPT;".to_string(),
            },
            Script {
                enabled: true,
                script: "iptables -D FORWARD -o \"$WG_INTERFACE\" -j ACCEPT;".to_string(),
            },
            #[cfg(not(feature = "docker"))]
            Script {
                enabled: true,
                script: "sysctl -w net.ipv4.ip_forward=0;".to_string(),
            },
        ];

        Scripts {
            pre_up: vec![],
            post_up,
            pre_down: vec![],
            post_down,
        }
    } else if utility_name == "pfctl" {
        let pf_vars = format!(
            r#"PF_CONF="/etc/pf.conf";
NAT_RULE="nat on {gateway} from $WG_SUBNET to any -> {gateway}";
"#
        );
        // PostUp scripts - 4 separate items
        let post_up = vec![
            Script {
                enabled: true,
                script: format!(
                    r#"{pf_vars}
awk "/^nat/ {{print; print \"$NAT_RULE\"; next}}1" "$PF_CONF" > "$PF_CONF.new";

if ! grep -qxF "$NAT_RULE" "$PF_CONF.new"; then
  echo "Error: could NOT configure firewall because there are no existing NAT rules at $PF_CONF. See notes at docs/notes/macos-firewall.md" >&2;
  rm -f "$PF_CONF.new";
  exit 1;
fi

mv "$PF_CONF" "$PF_CONF.bak";
mv "$PF_CONF.new" "$PF_CONF";"#
                ),
            },
            Script {
                enabled: true,
                script: "pfctl -f /etc/pf.conf;".to_string(),
            },
            Script {
                enabled: true,
                script: "pfctl -e || true;".to_string(),
            },
            Script {
                enabled: true,
                script: "sysctl -w net.inet.ip.forwarding=1;".to_string(),
            },
        ];

        // PostDown scripts - 3 separate items
        let post_down = vec![
            Script {
                enabled: true,
                script: format!(
                    r#"{pf_vars}
awk -v line="$NAT_RULE" '$0 != line' "$PF_CONF" > "$PF_CONF.new";

mv "$PF_CONF" "$PF_CONF.bak";
mv "$PF_CONF.new" "$PF_CONF";"#,
                ),
            },
            Script {
                enabled: true,
                script: "pfctl -d || true;".to_string(),
            },
            Script {
                enabled: true,
                script: "sysctl -w net.inet.ip.forwarding=0;".to_string(),
            },
        ];

        Scripts {
            pre_up: vec![],
            post_up,
            pre_down: vec![],
            post_down,
        }
    } else {
        // Unknown utility, return empty scripts
        Scripts::default()
    }
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


pub fn initialize_agent(init_opts: &InitOptions) -> Result<(), AgentInitError> {
    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    if file_path.exists() {
        return Err(AgentInitError::AlreadyInitialized(file_path.display().to_string()));
    }
    log::info!("Initializing wg-quickrs agent...");
    
    let mut step_counter = 1;
    let step_str = make_step_formatter(31);

    println!("[general network settings 1-2/31]");
    // [1/31] --network-identifier
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

    // [2/31] --network-subnet
    let network_subnet = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.network_subnet.map(|o| o.to_string()),
        INIT_NETWORK_SUBNET_FLAG,
        INIT_NETWORK_SUBNET_HELP,
        Some("10.0.34.0/24".into()),
        parse_and_validate_ipv4_subnet,
    );
    step_counter += 1;

    println!("[general network settings complete]");
    println!("[agent settings 3-9/31]");

    // Get primary IP of the current machine
    let iface_opt = recommend_interface();
    let iface_name = iface_opt.as_ref().map(|iface| iface.name.clone());
    let mut iface_ip = iface_opt.and_then(|iface| match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None });

    // [3/31] --agent-web-address
    let agent_web_address = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_address.map(|o| o.to_string()),
        INIT_AGENT_WEB_ADDRESS_FLAG,
        INIT_AGENT_WEB_ADDRESS_HELP,
        iface_ip.map(|o| o.to_string()),
        parse_and_validate_ipv4_address,
    );
    step_counter += 1;

    // [4/31] --agent-web-http-enabled & --agent-web-http-port
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

    // [5/31] --agent-web-https-enabled & --agent-web-https-port
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
        let (option_cert, option_key) = find_cert_server(config_folder, agent_web_address.to_string());

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
            move |s: &str| parse_and_validate_tls_file(config_folder, s),
        );
        let tls_key = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_https_tls_key.clone().map(|o| o.display().to_string()),
            INIT_AGENT_WEB_HTTPS_TLS_KEY_FLAG,
            format!("\t{}", INIT_AGENT_WEB_HTTPS_TLS_KEY_HELP).as_str(),
            option_key.map(|o| o.display().to_string()),
            move |s: &str| parse_and_validate_tls_file(config_folder, s),
        );
        (port, tls_cert, tls_key)
    } else {
        // if disabled, use a default port of 443
        (443, Default::default(), Default::default())
    };
    step_counter += 1;

    // [6/31] --agent-enable-web-password
    let mut agent_web_password_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_web_password_enabled,
        INIT_AGENT_WEB_PASSWORD_ENABLED_FLAG,
        INIT_AGENT_WEB_PASSWORD_ENABLED_HELP,
        true,
    );
    // [6/31] --agent-web-password
    let agent_web_password_hash = if agent_web_password_enabled {
        let password = get_init_password(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_web_password.clone(),
            INIT_AGENT_WEB_PASSWORD_FLAG,
            format!("\t{}", INIT_AGENT_WEB_PASSWORD_HELP).as_str(),
        );
        
        calculate_password_hash(password.trim()).unwrap_or_else(|_| {
            eprintln!("unable to calculate password hash, disabling password");
            agent_web_password_enabled = false;
            "".into()
        })
    } else {
        "".into()
    };
    step_counter += 1;

    // [7/31] --agent-vpn-enabled & --agent-vpn-port & --agent-vpn-wg & --agent-vpn-wg-userspace-enabled & --agent-vpn-wg-userspace-binary
    let agent_vpn_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_vpn_enabled,
        INIT_AGENT_VPN_ENABLED_FLAG,
        INIT_AGENT_VPN_ENABLED_HELP,
        true,
    );
    let (agent_vpn_port, agent_vpn_wg, agent_vpn_wg_userspace_enabled, agent_vpn_wg_userspace_binary) = if agent_vpn_enabled {
        // --agent-vpn-port
        let agent_vpn_port = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_vpn_port.map(|o| o.to_string()),
            INIT_AGENT_VPN_PORT_FLAG,
            format!("\t{}", INIT_AGENT_VPN_PORT_HELP).as_str(),
            Some("51820".into()),
            parse_and_validate_port,
        );

        // --agent-vpn-wg
        let agent_vpn_wg = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_vpn_wg.clone().map(|o| o.display().to_string()),
            INIT_AGENT_VPN_WG_FLAG,
            format!("\t{}", INIT_AGENT_VPN_WG_HELP).as_str(),
            wg_tool_options().into_iter().next().map(|o| o.display().to_string()),
            parse_and_validate_wg_tool,
        );

        // --agent-vpn-wg-userspace-enabled & --agent-vpn-wg-userspace-binary
        let agent_vpn_wg_userspace_enabled = get_bool(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_vpn_wg_userspace_enabled,
            INIT_AGENT_VPN_WG_USERSPACE_ENABLED_FLAG,
            format!("\t{}", INIT_AGENT_VPN_WG_USERSPACE_ENABLED_HELP).as_str(),
            !cfg!(target_os = "linux") || agent_vpn_wg.ends_with("awg"),  // if on linux, disable userspace to use kernel module
        );
        let agent_vpn_wg_userspace_binary = if agent_vpn_wg_userspace_enabled {
            get_value(
                init_opts.no_prompt,
                step_str(step_counter),
                init_opts.agent_vpn_wg_userspace_binary.clone().map(|o| o.display().to_string()),
                INIT_AGENT_VPN_WG_USERSPACE_BINARY_FLAG,
                format!("\t\t{}", INIT_AGENT_VPN_WG_USERSPACE_BINARY_HELP).as_str(),
                wg_userspace_options().into_iter().next().map(|o| o.display().to_string()),
                parse_and_validate_wg_userspace_binary,
            )
        } else {
            // if disabled, leave it empty
            PathBuf::new()
        };

        (agent_vpn_port, agent_vpn_wg, agent_vpn_wg_userspace_enabled, agent_vpn_wg_userspace_binary)
    } else {
        // if disabled, use a default port of 51820 and empty wg settings
        (51820, PathBuf::new(), false, PathBuf::new())
    };
    step_counter += 1;

    // AmneziaWG obfuscation parameters (only if using amneziawg)
    // [8/31] --network-amnezia-enabled
    let network_amnezia_enabled = match validate_amnezia_enabled(true, &agent_vpn_wg) {
        Ok(_) => {
            get_bool(
                init_opts.no_prompt,
                step_str(step_counter),
                init_opts.network_amnezia_enabled,
                INIT_NETWORK_AMNEZIA_ENABLED_FLAG,
                INIT_NETWORK_AMNEZIA_ENABLED_HELP,
                true,
            )
        }
        Err(e) => {
            println!("{} Not using amnezia ({}), skipping...", step_str(step_counter), e);
            false
        }
    };

    let amnezia_network_parameters = if network_amnezia_enabled {
        // --network-amnezia-s1
        let network_amnezia_s1 = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.network_amnezia_s1.map(|o| o.to_string()),
            INIT_NETWORK_AMNEZIA_S1_FLAG,
            INIT_NETWORK_AMNEZIA_S1_HELP,
            Some("55".into()),
            parse_and_validate_amnezia_s1,
        );

        // --network-amnezia-s2
        let network_amnezia_s2 = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.network_amnezia_s2.map(|o| o.to_string()),
            INIT_NETWORK_AMNEZIA_S2_FLAG,
            INIT_NETWORK_AMNEZIA_S2_HELP,
            Some("155".into()),
            move |s: &str| {
                parse_and_validate_amnezia_s1_s2(&network_amnezia_s1.to_string(), s)
            },
        );

        // --network-amnezia-h-random & --network-amnezia-h1-4
        let network_amnezia_h_random = get_bool(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.network_amnezia_h_random,
            INIT_NETWORK_AMNEZIA_H_RANDOM_FLAG,
            INIT_NETWORK_AMNEZIA_H_RANDOM_HELP,
            true,
        );

        let (network_amnezia_h1, network_amnezia_h2, network_amnezia_h3, network_amnezia_h4) =
            if network_amnezia_h_random {
                // Generate random values
                (rand::rng().next_u32(), rand::rng().next_u32(), rand::rng().next_u32(), rand::rng().next_u32())
            } else {
                // Prompt for individual values
                let network_amnezia_h1 = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.network_amnezia_h1.map(|o| o.to_string()),
                    INIT_NETWORK_AMNEZIA_H1_FLAG,
                    INIT_NETWORK_AMNEZIA_H1_HELP,
                    Some(rand::rng().next_u32().to_string()),
                    parse_and_validate_amnezia_h,
                );

                let network_amnezia_h2 = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.network_amnezia_h2.map(|o| o.to_string()),
                    INIT_NETWORK_AMNEZIA_H2_FLAG,
                    INIT_NETWORK_AMNEZIA_H2_HELP,
                    Some(rand::rng().next_u32().to_string()),
                    parse_and_validate_amnezia_h,
                );

                let network_amnezia_h3 = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.network_amnezia_h3.map(|o| o.to_string()),
                    INIT_NETWORK_AMNEZIA_H3_FLAG,
                    INIT_NETWORK_AMNEZIA_H3_HELP,
                    Some(rand::rng().next_u32().to_string()),
                    parse_and_validate_amnezia_h,
                );

                let network_amnezia_h4 = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.network_amnezia_h4.map(|o| o.to_string()),
                    INIT_NETWORK_AMNEZIA_H4_FLAG,
                    INIT_NETWORK_AMNEZIA_H4_HELP,
                    Some(rand::rng().next_u32().to_string()),
                    parse_and_validate_amnezia_h,
                );

                (network_amnezia_h1, network_amnezia_h2, network_amnezia_h3, network_amnezia_h4)
            };

        AmneziaNetworkParameters { enabled: true, s1: network_amnezia_s1, s2: network_amnezia_s2, h1: network_amnezia_h1, h2: network_amnezia_h2, h3: network_amnezia_h3, h4: network_amnezia_h4 }
    } else {
        AmneziaNetworkParameters {
            enabled: false,
            s1: 55,
            s2: 155,
            h1: rand::rng().next_u32(),
            h2: rand::rng().next_u32(),
            h3: rand::rng().next_u32(),
            h4: rand::rng().next_u32(),
        }
    };
    step_counter += 1;

    // [9/31] --agent-firewall-enabled
    let agent_firewall_enabled = get_bool(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_firewall_enabled,
        INIT_AGENT_FIREWALL_ENABLED_FLAG,
        INIT_AGENT_FIREWALL_ENABLED_HELP,
        true,
    );

    let (http_firewall_scripts, https_firewall_scripts, vpn_firewall_scripts) = if agent_firewall_enabled {
        // HTTP firewall
        let agent_firewall_configure_http = get_bool(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_firewall_configure_http,
            INIT_AGENT_FIREWALL_CONFIGURE_HTTP_FLAG,
            format!("\t{}", INIT_AGENT_FIREWALL_CONFIGURE_HTTP_HELP).as_str(),
            true,
        );

        let http_scripts = if agent_firewall_configure_http {
            let agent_firewall_http_automated = get_bool(
                init_opts.no_prompt,
                step_str(step_counter),
                init_opts.agent_firewall_http_automated,
                INIT_AGENT_FIREWALL_HTTP_AUTOMATED_FLAG,
                format!("\t\t{}", INIT_AGENT_FIREWALL_HTTP_AUTOMATED_HELP).as_str(),
                true,
            );

            if agent_firewall_http_automated {
                let agent_firewall_utility = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_utility.clone().map(|o| o.display().to_string()),
                    INIT_AGENT_FIREWALL_UTILITY_FLAG,
                    format!("\t\t\t{}", INIT_AGENT_FIREWALL_UTILITY_HELP).as_str(),
                    firewall_utility_options().into_iter().next().map(|o| o.display().to_string()),
                    parse_and_validate_fw_utility,
                );
                let scripts = generate_http_firewall_scripts(&agent_firewall_utility.display().to_string());
                println!("\t\t\t✓ HTTP firewall: {} pre-up, {} post-down script(s)",
                         scripts.pre_up.len(), scripts.post_down.len());
                scripts
            } else {
                // Manual setup using get_scripts helper
                let agent_firewall_http_pre_up = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_http_pre_up_enabled,
                    init_opts.agent_firewall_http_pre_up_line.clone(),
                    INIT_AGENT_FIREWALL_HTTP_PRE_UP_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_HTTP_PRE_UP_LINE_FLAG,
                    INIT_AGENT_FIREWALL_HTTP_PRE_UP_ENABLED_HELP,
                    "\t\t",
                );

                let agent_firewall_http_post_down = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_http_post_down_enabled,
                    init_opts.agent_firewall_http_post_down_line.clone(),
                    INIT_AGENT_FIREWALL_HTTP_POST_DOWN_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_HTTP_POST_DOWN_LINE_FLAG,
                    INIT_AGENT_FIREWALL_HTTP_POST_DOWN_ENABLED_HELP,
                    "\t\t",
                );

                HttpScripts { pre_up: agent_firewall_http_pre_up, post_down: agent_firewall_http_post_down }
            }
        } else {
            HttpScripts::default()
        };

        // HTTPS firewall
        let agent_firewall_configure_https = get_bool(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_firewall_configure_https,
            INIT_AGENT_FIREWALL_CONFIGURE_HTTPS_FLAG,
            format!("\t{}", INIT_AGENT_FIREWALL_CONFIGURE_HTTPS_HELP).as_str(),
            true,
        );

        let https_scripts = if agent_firewall_configure_https {
            let agent_firewall_https_automated = get_bool(
                init_opts.no_prompt,
                step_str(step_counter),
                init_opts.agent_firewall_https_automated,
                INIT_AGENT_FIREWALL_HTTPS_AUTOMATED_FLAG,
                format!("\t\t{}", INIT_AGENT_FIREWALL_HTTPS_AUTOMATED_HELP).as_str(),
                true,
            );

            if agent_firewall_https_automated {
                let agent_firewall_utility = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_utility.clone().map(|o| o.display().to_string()),
                    INIT_AGENT_FIREWALL_UTILITY_FLAG,
                    format!("\t\t\t{}", INIT_AGENT_FIREWALL_UTILITY_HELP).as_str(),
                    firewall_utility_options().into_iter().next().map(|o| o.display().to_string()),
                    parse_and_validate_fw_utility,
                );
                let scripts = generate_http_firewall_scripts(&agent_firewall_utility.display().to_string());
                println!("\t\t\t✓ HTTPS firewall: {} pre-up, {} post-down script(s)",
                         scripts.pre_up.len(), scripts.post_down.len());
                scripts
            } else {
                // Manual setup using get_scripts helper
                let agent_firewall_https_pre_up = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_https_pre_up_enabled,
                    init_opts.agent_firewall_https_pre_up_line.clone(),
                    INIT_AGENT_FIREWALL_HTTPS_PRE_UP_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_HTTPS_PRE_UP_LINE_FLAG,
                    INIT_AGENT_FIREWALL_HTTPS_PRE_UP_ENABLED_HELP,
                    "\t\t",
                );

                let agent_firewall_https_post_down = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_https_post_down_enabled,
                    init_opts.agent_firewall_https_post_down_line.clone(),
                    INIT_AGENT_FIREWALL_HTTPS_POST_DOWN_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_HTTPS_POST_DOWN_LINE_FLAG,
                    INIT_AGENT_FIREWALL_HTTPS_POST_DOWN_ENABLED_HELP,
                    "\t\t",
                );

                HttpScripts { pre_up: agent_firewall_https_pre_up, post_down: agent_firewall_https_post_down }
            }
        } else {
            HttpScripts::default()
        };

        // VPN firewall
        let agent_firewall_configure_vpn = get_bool(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_firewall_configure_vpn,
            INIT_AGENT_FIREWALL_CONFIGURE_VPN_FLAG,
            format!("\t{}", INIT_AGENT_FIREWALL_CONFIGURE_VPN_HELP).as_str(),
            true,
        );

        let vpn_scripts = if agent_firewall_configure_vpn {
            let agent_firewall_vpn_automated = get_bool(
                init_opts.no_prompt,
                step_str(step_counter),
                init_opts.agent_firewall_vpn_automated,
                INIT_AGENT_FIREWALL_VPN_AUTOMATED_FLAG,
                format!("\t\t{}", INIT_AGENT_FIREWALL_VPN_AUTOMATED_HELP).as_str(),
                true,
            );

            if agent_firewall_vpn_automated {
                let agent_firewall_utility = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_utility.clone().map(|o| o.display().to_string()),
                    INIT_AGENT_FIREWALL_UTILITY_FLAG,
                    format!("\t\t\t{}", INIT_AGENT_FIREWALL_UTILITY_HELP).as_str(),
                    firewall_utility_options().into_iter().next().map(|o| o.display().to_string()),
                    parse_and_validate_fw_utility,
                );
                let agent_firewall_gateway = get_value(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_gateway.clone(),
                    INIT_AGENT_FIREWALL_GATEWAY_FLAG,
                    format!("\t\t\t{}", INIT_AGENT_FIREWALL_GATEWAY_HELP).as_str(),
                    iface_name.clone(),
                    parse_and_validate_fw_gateway,
                );
                let scripts = generate_firewall_scripts(&agent_firewall_utility.display().to_string(), &agent_firewall_gateway);
                println!("\t\t\t✓ VPN firewall: {} pre-up, {} post-up, {} pre-down, {} post-down script(s)",
                    scripts.pre_up.len(), scripts.post_up.len(),
                    scripts.pre_down.len(), scripts.post_down.len());
                scripts
            } else {
                // Manual setup using get_scripts helper
                let agent_firewall_vpn_pre_up = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_vpn_pre_up_enabled,
                    init_opts.agent_firewall_vpn_pre_up_line.clone(),
                    INIT_AGENT_FIREWALL_VPN_PRE_UP_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_VPN_PRE_UP_LINE_FLAG,
                    INIT_AGENT_FIREWALL_VPN_PRE_UP_ENABLED_HELP,
                    "\t\t",
                );

                let agent_firewall_vpn_post_up = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_vpn_post_up_enabled,
                    init_opts.agent_firewall_vpn_post_up_line.clone(),
                    INIT_AGENT_FIREWALL_VPN_POST_UP_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_VPN_POST_UP_LINE_FLAG,
                    INIT_AGENT_FIREWALL_VPN_POST_UP_ENABLED_HELP,
                    "\t\t",
                );

                let agent_firewall_vpn_pre_down = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_vpn_pre_down_enabled,
                    init_opts.agent_firewall_vpn_pre_down_line.clone(),
                    INIT_AGENT_FIREWALL_VPN_PRE_DOWN_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_VPN_PRE_DOWN_LINE_FLAG,
                    INIT_AGENT_FIREWALL_VPN_PRE_DOWN_ENABLED_HELP,
                    "\t\t",
                );

                let agent_firewall_vpn_post_down = get_scripts(
                    init_opts.no_prompt,
                    step_str(step_counter),
                    init_opts.agent_firewall_vpn_post_down_enabled,
                    init_opts.agent_firewall_vpn_post_down_line.clone(),
                    INIT_AGENT_FIREWALL_VPN_POST_DOWN_ENABLED_FLAG,
                    INIT_AGENT_FIREWALL_VPN_POST_DOWN_LINE_FLAG,
                    INIT_AGENT_FIREWALL_VPN_POST_DOWN_ENABLED_HELP,
                    "\t\t",
                );

                Scripts {
                    pre_up: agent_firewall_vpn_pre_up,
                    post_up: agent_firewall_vpn_post_up,
                    pre_down: agent_firewall_vpn_pre_down,
                    post_down: agent_firewall_vpn_post_down,
                }
            }
        } else {
            Scripts::default()
        };

        (http_scripts, https_scripts, vpn_scripts)
    } else {
        (HttpScripts::default(), HttpScripts::default(), Scripts::default())
    };
    step_counter += 1;

    println!("[agent settings complete]");
    println!("[peer settings 10-21/31]");

    // [10/31] --agent-peer-name
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

    // [11/31] --agent-peer-vpn-internal-address
    let temp_network = Network {
        name: "".to_string(),
        subnet: network_subnet,
        this_peer: Default::default(),
        peers: Default::default(),
        connections: Default::default(),
        defaults: Default::default(),
        reservations: Default::default(),
        amnezia_parameters: Default::default(),
        updated_at: Utc::now(),
    };
    let agent_peer_vpn_internal_address = get_value(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_vpn_internal_address.map(|o| o.to_string()),
        INIT_AGENT_PEER_VPN_INTERNAL_ADDRESS_FLAG,
        INIT_AGENT_PEER_VPN_INTERNAL_ADDRESS_HELP,
        network_subnet.hosts().next().map(|o| o.to_string()),
        move |s: &str| parse_and_validate_peer_address(s, &temp_network),
    );
    step_counter += 1;

    // update the address in the recommended endpoint
    for iface in get_interfaces() {
        if iface_name == Some(iface.name.clone()) {
            iface_ip = match iface.ip() { IpAddr::V4(v4) => Some(v4), _ => None };
        }
    }

    // TODO: allow roaming init
    // [12/31] --agent-peer-vpn-endpoint
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

    // [13/31] --agent-peer-kind
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

    // [14/31] --agent-peer-icon-enabled & --agent-peer-icon-src
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

    // [15/31] --agent-peer-dns-enabled & --agent-peer-dns-addresses
    let agent_peer_dns_addresses = get_dns_addresses(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_dns_enabled,
        init_opts.agent_peer_dns_addresses.clone(),
        INIT_AGENT_PEER_DNS_ENABLED_FLAG,
        INIT_AGENT_PEER_DNS_ADDRESSES_FLAG,
        INIT_AGENT_PEER_DNS_ENABLED_HELP,
        INIT_AGENT_PEER_DNS_ADDRESSES_HELP,
    );
    let agent_peer_dns_enabled = !agent_peer_dns_addresses.is_empty();
    step_counter += 1;

    // [16/31] --agent-peer-mtu-enabled & --agent-peer-mtu-value
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
            init_opts.agent_peer_mtu_value.map(|o| o.to_string()),
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

    // [17/31] --agent-peer-script-pre-up-enabled & --agent-peer-script-pre-up-line
    let agent_peer_script_pre_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_pre_up_enabled,
        init_opts.agent_peer_script_pre_up_line.clone(),
        INIT_AGENT_PEER_SCRIPT_PRE_UP_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_UP_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_UP_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [18/31] --agent-peer-script-post-up-enabled & --agent-peer-script-post-up-line
    let agent_peer_script_post_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_post_up_enabled,
        init_opts.agent_peer_script_post_up_line.clone(),
        INIT_AGENT_PEER_SCRIPT_POST_UP_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_UP_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_UP_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [19/31] --agent-peer-script-pre-down-enabled & --agent-peer-script-pre-down-line
    let agent_peer_script_pre_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_pre_down_enabled,
        init_opts.agent_peer_script_pre_down_line.clone(),
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_PRE_DOWN_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [20/31] --agent-peer-script-post-down-enabled & --agent-peer-script-post-down-line
    let agent_peer_script_post_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.agent_peer_script_post_down_enabled,
        init_opts.agent_peer_script_post_down_line.clone(),
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_ENABLED_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_LINE_FLAG,
        INIT_AGENT_PEER_SCRIPT_POST_DOWN_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // Agent peer AmneziaWG obfuscation parameters (only if using amneziawg)
    // [21/31] --agent-peer-amnezia-jc & --agent-peer-amnezia-jmin & --agent-peer-amnezia-jmax
    let agent_peer_amnezia_parameters = if network_amnezia_enabled {
        let agent_peer_amnezia_jc = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_amnezia_jc.map(|o| o.to_string()),
            INIT_AGENT_PEER_AMNEZIA_JC_FLAG,
            INIT_AGENT_PEER_AMNEZIA_JC_HELP,
            Some("30".into()),
            parse_and_validate_amnezia_jc,
        );

        let agent_peer_amnezia_jmin = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_amnezia_jmin.map(|o| o.to_string()),
            INIT_AGENT_PEER_AMNEZIA_JMIN_FLAG,
            INIT_AGENT_PEER_AMNEZIA_JMIN_HELP,
            Some("60".into()),
            parse_and_validate_amnezia_jmin,
        );

        let agent_peer_amnezia_jmax = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.agent_peer_amnezia_jmax.map(|o| o.to_string()),
            INIT_AGENT_PEER_AMNEZIA_JMAX_FLAG,
            INIT_AGENT_PEER_AMNEZIA_JMAX_HELP,
            Some("120".into()),
            move |s: &str| {
                let jmax_value = parse_and_validate_amnezia_jmax(s)?;
                validate_amnezia_jmin_jmax(agent_peer_amnezia_jmin, jmax_value)?;
                Ok(jmax_value)
            },
        );

        AmneziaPeerParameters { jc: agent_peer_amnezia_jc, jmin: agent_peer_amnezia_jmin, jmax: agent_peer_amnezia_jmax }
    } else {
        println!("{} Not using amnezia, skipping...", step_str(step_counter));
        AmneziaPeerParameters { jc: 30, jmin: 60, jmax: 120 }
    };
    step_counter += 1;

    println!("[peer settings complete]");
    println!("[new peer/connection default settings 22-31/31]");

    // [22/31] --default-peer-kind
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

    // [23/31] --default-peer-icon-enabled & --default-peer-icon-src
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

    // [24/31] --default-peer-dns-enabled & --default-peer-dns-addresses
    let default_peer_dns_addresses = get_dns_addresses(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_dns_enabled,
        init_opts.default_peer_dns_addresses.clone(),
        INIT_DEFAULT_PEER_DNS_ENABLED_FLAG,
        INIT_DEFAULT_PEER_DNS_ADDRESSES_FLAG,
        INIT_DEFAULT_PEER_DNS_ENABLED_HELP,
        INIT_DEFAULT_PEER_DNS_ADDRESSES_HELP,
    );
    let default_peer_dns_enabled = !default_peer_dns_addresses.is_empty();
    step_counter += 1;

    // [25/31] --default-peer-mtu-enabled & --default-peer-mtu-value
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
            init_opts.default_peer_mtu_value.map(|o| o.to_string()),
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

    // [26/31] --default-peer-script-pre-up-enabled & --default-peer-script-pre-up-line
    let default_peer_script_pre_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_pre_up_enabled,
        init_opts.default_peer_script_pre_up_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_UP_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [27/31] --default-peer-script-post-up-enabled & --default-peer-script-post-up-line
    let default_peer_script_post_up = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_post_up_enabled,
        init_opts.default_peer_script_post_up_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_UP_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [28/31] --default-peer-script-pre-down-enabled & --default-peer-script-pre-down-line
    let default_peer_script_pre_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_pre_down_enabled,
        init_opts.default_peer_script_pre_down_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_PRE_DOWN_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // [29/31] --default-peer-script-post-down-enabled & --default-peer-script-post-down-line
    let default_peer_script_post_down = get_scripts(
        init_opts.no_prompt,
        step_str(step_counter),
        init_opts.default_peer_script_post_down_enabled,
        init_opts.default_peer_script_post_down_line.clone(),
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_ENABLED_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_LINE_FLAG,
        INIT_DEFAULT_PEER_SCRIPT_POST_DOWN_ENABLED_HELP,
        "",
    );
    step_counter += 1;

    // Default peer AmneziaWG obfuscation parameters (only if using amneziawg)
    // [30/31] --default-peer-amnezia-jc & --default-peer-amnezia-jmin & --default-peer-amnezia-jmax
    let default_peer_amnezia_parameters = if network_amnezia_enabled {
        let default_peer_amnezia_jc = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_amnezia_jc.map(|o| o.to_string()),
            INIT_DEFAULT_PEER_AMNEZIA_JC_FLAG,
            INIT_DEFAULT_PEER_AMNEZIA_JC_HELP,
            Some("30".into()),
            parse_and_validate_amnezia_jc,
        );

        let default_peer_amnezia_jmin = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_amnezia_jmin.map(|o| o.to_string()),
            INIT_DEFAULT_PEER_AMNEZIA_JMIN_FLAG,
            INIT_DEFAULT_PEER_AMNEZIA_JMIN_HELP,
            Some("60".into()),
            parse_and_validate_amnezia_jmin,
        );

        let default_peer_amnezia_jmax = get_value(
            init_opts.no_prompt,
            step_str(step_counter),
            init_opts.default_peer_amnezia_jmax.map(|o| o.to_string()),
            INIT_DEFAULT_PEER_AMNEZIA_JMAX_FLAG,
            INIT_DEFAULT_PEER_AMNEZIA_JMAX_HELP,
            Some("120".into()),
            move |s: &str| {
                let jmax_value = parse_and_validate_amnezia_jmax(s)?;
                validate_amnezia_jmin_jmax(default_peer_amnezia_jmin, jmax_value)?;
                Ok(jmax_value)
            },
        );

        AmneziaPeerParameters { jc: default_peer_amnezia_jc, jmin: default_peer_amnezia_jmin, jmax: default_peer_amnezia_jmax }
    } else {
        println!("{} Not using amnezia, skipping...", step_str(step_counter));
        AmneziaPeerParameters { jc: 30, jmin: 60, jmax: 120 }
    };
    step_counter += 1;

    // [31/31] --default-connection-persistent-keepalive-enabled & --default-connection-persistent-keepalive-period
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
            init_opts.default_connection_persistent_keepalive_period.map(|o| o.to_string()),
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
                wg: agent_vpn_wg,
                wg_userspace: WireGuardUserspace {
                    enabled: agent_vpn_wg_userspace_enabled,
                    binary: agent_vpn_wg_userspace_binary,
                },
            },
            firewall: AgentFirewall {
                http: http_firewall_scripts,
                https: https_firewall_scripts,
                vpn: vpn_firewall_scripts,
            },
        },
        network: Network {
            name: network_name.to_string(),
            subnet: network_subnet,
            this_peer: peer_id,
            peers: {
                let mut map = BTreeMap::new();
                map.insert(peer_id, Peer {
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
                    private_key: wg_generate_key(),
                    amnezia_parameters: agent_peer_amnezia_parameters,
                    created_at: now,
                    updated_at: now,
                });
                map
            },
            connections: BTreeMap::new(),
            defaults: Defaults {
                peer: DefaultPeer {
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
                    amnezia_parameters: default_peer_amnezia_parameters
                },
                connection: DefaultConnection {
                    persistent_keepalive: PersistentKeepalive {
                        enabled: default_connection_persistent_keepalive_enabled,
                        period: default_connection_persistent_keepalive_period,
                    },
                },
            },
            reservations: BTreeMap::new(),
            amnezia_parameters: amnezia_network_parameters,
            updated_at: now,
        },
    };

    conf::util::set_config(&mut config)?;
    println!(
        "✅ Configuration saved to {}",
        WG_QUICKRS_CONFIG_FILE.get().unwrap().display()
    );

    Ok(())
}
