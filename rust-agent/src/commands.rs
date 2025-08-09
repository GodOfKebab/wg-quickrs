use crate::cli::AgentCommands;
use crate::conf::util::ConfUtilError;
use crate::web::server;
use crate::wireguard::cmd::get_public_private_keys;
use crate::{WG_RUSTEZE_CONFIG_FILE, WIREGUARD_CONFIG_FILE, conf, wireguard};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use dialoguer::{Confirm, Input};
use get_if_addrs::get_if_addrs;
use ipnetwork::IpNetwork;
use rand::{RngCore, rng};
use rust_wasm::types::{
    Agent, AgentVpn, AgentWeb, Config, DefaultConnection, DefaultPeer, Defaults, EnabledValue,
    Network, Password, Peer, Scripts,
};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use uuid::Uuid;

// Helper to prompt a value with optional default
fn prompt<T: std::str::FromStr>(msg: &str, default: Option<&str>) -> T {
    let input = if let Some(d) = default {
        Input::new()
            .with_prompt(format!("{msg} ({d})"))
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

pub(crate) fn initialize_agent(
    cli_network_identifier: Option<String>,
    cli_peer_name: Option<String>,
    cli_public_address: Option<String>,
    cli_web_port: Option<u16>,
    cli_vpn_port: Option<u16>,
    cli_subnet: Option<String>,
    cli_vpn_address: Option<String>,
    cli_use_tls: Option<bool>,
    cli_password: Option<String>,
    cli_dns_server: Option<String>,
    cli_mtu_value: Option<String>,
    cli_persistent_keepalive_seconds: Option<String>,
) -> ExitCode {
    if let Err(ConfUtilError::Read(..)) = conf::util::get_config() {
    } else {
        log::error!("wg-rusteze rust-agent is already initialized.");
        return ExitCode::FAILURE;
    }
    log::info!("Initializing wg-rusteze rust-agent...");

    let identifier = match cli_network_identifier {
        Some(v) => {
            println!("[1/12] Using network identifier from CLI argument: {}", v);
            v.clone()
        }
        _ => prompt("[1/12] Enter network identifier", Some("wg-rusteze")),
    };
    let peer_name = match cli_peer_name {
        Some(v) => {
            println!("[2/12] Using peer name from CLI argument: {}", v);
            v.clone()
        }
        _ => prompt("[2/12] Enter peer name", Some("wg-rusteze-host")),
    };
    let public_address = match cli_public_address {
        Some(v) => {
            println!("[3/12] Using public IPv4 address from CLI argument: {}", v);
            v.clone()
        }
        _ => prompt(
            "[3/12] Enter public IPv4 address for rust-agent",
            primary_ip().as_deref(),
        ),
    };
    let web_port = match cli_web_port {
        Some(v) => {
            println!("[4/12] Using web port from CLI argument: {}", v);
            v.clone()
        }
        _ => prompt("[4/12] Enter web port", Some("8080")),
    };
    let vpn_port = match cli_vpn_port {
        Some(v) => {
            println!("[5/12] Using VPN port from CLI argument: {}", v);
            v.clone()
        }
        _ => prompt("[5/12] Enter VPN port", Some("51820")),
    };
    let subnet = match cli_subnet {
        Some(v) => {
            println!(
                "[6/12] Using network CIDR subnet mask from CLI argument: {}",
                v
            );
            v.clone()
        }
        _ => prompt(
            "[6/12] Enter network CIDR subnet mask",
            Some("10.0.34.0/24"),
        ),
    };
    let vpn_address = match cli_vpn_address {
        Some(v) => {
            println!(
                "[7/12] Using VPN address for rust-agent from CLI argument: {}",
                v
            );
            v.clone()
        }
        _ => prompt(
            "[7/12] Enter VPN address for rust-agent",
            Some(&*first_ip(&subnet)),
        ),
    };
    let use_tls = match cli_use_tls {
        Some(v) => {
            println!(
                "[8/12] TLS is {} from CLI argument",
                if v { "enabled" } else { "disabled" }
            );
            v.clone()
        }
        _ => Confirm::new()
            .with_prompt("[8/12] Use TLS?")
            .default(true)
            .interact()
            .unwrap(),
    };
    // TODO: add --no-password flag
    let pwd_enabled: bool;
    let pwd = match cli_password {
        Some(v) => {
            println!("[9/12] Using password from CLI argument: ***hidden***");
            pwd_enabled = true;
            v.clone()
        }
        _ => {
            pwd_enabled = Confirm::new()
                .with_prompt("[9/12] Enable password?")
                .default(true)
                .interact()
                .unwrap();
            if pwd_enabled {
                dialoguer::Password::new()
                    .with_prompt("[9/12]\t Enter password")
                    .interact()
                    .unwrap()
            } else {
                "".to_string()
            }
        }
    };

    let pwd_hash = match calculate_password_hash(pwd.trim()) {
        Ok(p) => {
            if pwd_enabled {
                p
            } else {
                "".into()
            }
        }
        Err(e) => {
            return e;
        }
    };

    // TODO: add --no-dns flag
    let dns_enabled: bool;
    let dns_value = match cli_dns_server {
        Some(v) => {
            println!("[10/12] Using DNS server from CLI argument: {}", v);
            dns_enabled = true;
            v.clone()
        }
        _ => {
            dns_enabled = Confirm::new()
                .with_prompt("[10/12] Enable DNS?")
                .default(true)
                .interact()
                .unwrap();
            if dns_enabled {
                prompt("[10/12]\t DNS value", Some("1.1.1.1"))
            } else {
                "".to_string()
            }
        }
    };

    // TODO: add --no-mtu flag
    let mtu_enabled: bool;
    let mtu_value = match cli_mtu_value {
        Some(v) => {
            println!("[11/12] Using MTU value from CLI argument: {}", v);
            mtu_enabled = true;
            v.clone()
        }
        _ => {
            mtu_enabled = Confirm::new()
                .with_prompt("[11/12] Enable MTU?")
                .default(false)
                .interact()
                .unwrap();
            if mtu_enabled {
                prompt("[11/12]\t MTU value", Some("1420"))
            } else {
                "".to_string()
            }
        }
    };

    // TODO: add --no-persistent-keepalive flag
    let pk_enabled: bool;
    let pk_value = match cli_persistent_keepalive_seconds {
        Some(v) => {
            pk_enabled = true;
            v.clone()
        }
        _ => {
            pk_enabled = Confirm::new()
                .with_prompt("[12/12] Enable PersistentKeepalive in connections?")
                .default(true)
                .interact()
                .unwrap();
            if pk_enabled {
                prompt("[12/12]\t PersistentKeepalive value (seconds)", Some("25"))
            } else {
                "".to_string()
            }
        }
    };
    println!(
        "✅ This was all the information required to initialize the rust-agent. Finalizing the configuration..."
    );

    let peer_id = Uuid::new_v4().to_string();
    let pub_priv_key = get_public_private_keys().unwrap();
    let now = conf::timestamp::get_now_timestamp_formatted();

    let peer = Peer {
        name: peer_name,
        address: vpn_address,
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
            value: format!("{public_address}:{vpn_port}"),
        },
        dns: EnabledValue {
            enabled: dns_enabled,
            value: dns_value.clone(),
        },
        mtu: EnabledValue {
            enabled: mtu_enabled,
            value: mtu_value,
        },
        scripts: Scripts {
            pre_up: EnabledValue {
                enabled: false,
                value: "".into(),
            },
            post_up: EnabledValue {
                enabled: false,
                value: "".into(),
            },
            pre_down: EnabledValue {
                enabled: false,
                value: "".into(),
            },
            post_down: EnabledValue {
                enabled: false,
                value: "".into(),
            },
        },
    };

    let config = Config {
        agent: Agent {
            address: public_address,
            web: AgentWeb {
                port: web_port,
                use_tls,
                password: Password {
                    enabled: pwd_enabled,
                    hash: pwd_hash,
                },
            },
            vpn: AgentVpn { port: vpn_port },
        },
        network: Network {
            identifier,
            subnet: subnet.clone(),
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
                        enabled: true,
                        value: dns_value,
                    },
                    mtu: EnabledValue {
                        enabled: false,
                        value: "".to_string(),
                    },
                    scripts: Scripts {
                        pre_up: EnabledValue {
                            enabled: false,
                            value: "".into(),
                        },
                        post_up: EnabledValue {
                            enabled: false,
                            value: "".into(),
                        },
                        pre_down: EnabledValue {
                            enabled: false,
                            value: "".into(),
                        },
                        post_down: EnabledValue {
                            enabled: false,
                            value: "".into(),
                        },
                    },
                },
                connection: DefaultConnection {
                    persistent_keepalive: EnabledValue {
                        enabled: pk_enabled,
                        value: pk_value,
                    },
                },
            },
        },
    };

    let yaml = serde_yml::to_string(&config).unwrap();
    let file_path = WG_RUSTEZE_CONFIG_FILE.get().unwrap();
    std::fs::write(file_path, yaml).expect("Failed to write config.yml");
    println!("✅ Configuration saved to `config.yml`.");

    ExitCode::SUCCESS
}

pub(crate) fn calculate_password_hash(password: &str) -> Result<String, ExitCode> {
    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = match SaltString::encode_b64(&sbytes) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return Err(ExitCode::FAILURE);
        }
    };

    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_ref(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => {
            log::error!("Password hashing failed: {e}");
            Err(ExitCode::FAILURE)
        }
    }
}

pub(crate) fn reset_web_password() -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    log::info!("Resetting the web password...");
    print!("Enter your new password: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

    let mut password = String::new();
    match io::stdin().read_line(&mut password) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    }
    let password_hash = match calculate_password_hash(password.trim()) {
        // Remove newline character
        Ok(p) => p,
        Err(e) => {
            return e;
        }
    };

    config.agent.web.password.enabled = true;
    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config).expect("Failed to set config");

    ExitCode::SUCCESS
}

pub(crate) async fn run_agent(
    wireguard_config_folder: &Path,
    tls_cert: &PathBuf,
    tls_key: &PathBuf,
    commands: &AgentCommands,
) -> ExitCode {
    // get the wireguard config file path
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let mut run_wireguard = true;
    let mut run_web = true;
    match commands {
        AgentCommands::Run(opts) => {
            if opts.only_wireguard {
                run_web = false;
                log::info!("--only-wireguard flag detected. Starting only the wireguard server...")
            } else if opts.only_web {
                run_wireguard = false;
                log::info!("--only-web flag detected. Running only the web configuration portal...")
            } else if opts.all {
                log::info!(
                    "--all flag detected. Starting the wireguard server and running the web configuration portal..."
                )
            } else {
                log::info!(
                    "No run mode selected. Defaulting to --all (Starting the wireguard server and running the web configuration portal...)"
                );
            }
        }
    }

    if run_wireguard {
        WIREGUARD_CONFIG_FILE
            .set(wireguard_config_folder.join(format!("{}.conf", config.network.identifier)))
            .expect("Failed to set WIREGUARD_CONFIG_FILE");
        log::info!(
            "using the wireguard config file at \"{}\"",
            WIREGUARD_CONFIG_FILE.get().unwrap().display()
        );

        // start the tunnel
        wireguard::cmd::start_tunnel(&config).unwrap_or_else(|e| {
            log::error!("{e}");
        });
    }

    if run_web {
        // start the HTTP server with TLS for frontend and API control
        server::run_http_server(&config, tls_cert, tls_key)
            .await
            .expect("HTTP server failed to start");
    }
    ExitCode::SUCCESS
}
