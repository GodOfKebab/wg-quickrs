use crate::conf;
use crate::commands::config::{ConfigCommandError};
use crate::commands::helpers::*;
use chrono::Utc;
use std::net::Ipv4Addr;
use uuid::Uuid;
use wg_quickrs_lib::helpers::{get_connection_id, wg_generate_key};
use wg_quickrs_lib::types::network::*;
use wg_quickrs_lib::validation::network::*;
use wg_quickrs_cli::config::add::{AddPeerOptions, AddConnectionOptions};
use crate::conf::network::get_next_available_address;

include!(concat!(env!("OUT_DIR"), "/add_peer_options_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/add_connection_options_generated.rs"));

/// Add a new peer to the network
pub fn add_peer(opts: &AddPeerOptions) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;

    // Generate new peer ID
    let peer_id = Uuid::new_v4();
    let mut step_counter = 1;
    let step_str = make_step_formatter(11);

    // Get peer name
    let peer_name = get_value(
        opts.no_prompt,
        step_str(step_counter),
        opts.name.clone(),
        ADD_PEER_NAME_FLAG,
        ADD_PEER_NAME_HELP,
        Some(format!("peer-{}", &peer_id.to_string()[..8])),
        parse_and_validate_peer_name,
    );
    step_counter += 1;

    // Get peer address
    let taken_addresses: Vec<Ipv4Addr> = config.network.peers.values()
        .map(|p| p.address)
        .chain(config.network.reservations.keys().copied())
        .collect();
    let network_copy = config.network.clone();
    let peer_address = get_value(
        opts.no_prompt,
        step_str(step_counter),
        opts.address.clone().map(|o| o.to_string()),
        ADD_PEER_ADDRESS_FLAG,
        ADD_PEER_ADDRESS_HELP,
        get_next_available_address(&config.network.subnet, &taken_addresses).map(|o| o.to_string()),
        move |s: &str| parse_and_validate_peer_address(s, &network_copy),
    );

    // Get endpoint
    let endpoint_enabled = get_bool(
        opts.no_prompt,
        step_str(step_counter),
        opts.endpoint_enabled,
        ADD_PEER_ENDPOINT_ENABLED_FLAG,
        ADD_PEER_ENDPOINT_ENABLED_HELP,
        config.network.defaults.peer.endpoint.enabled,
    );

    let endpoint_address = if endpoint_enabled {
        get_value(
            opts.no_prompt,
            step_str(step_counter),
            opts.endpoint_address.clone(),
            ADD_PEER_ENDPOINT_ADDRESS_FLAG,
            ADD_PEER_ENDPOINT_ADDRESS_HELP,
            None,
            parse_and_validate_peer_endpoint,
        )
    } else {
        EndpointAddress::None
    };
    step_counter += 1;

    // Get peer kind
    let peer_kind = get_value(
        opts.no_prompt,
        step_str(step_counter),
        opts.kind.clone(),
        ADD_PEER_KIND_FLAG,
        ADD_PEER_KIND_HELP,
        Some(config.network.defaults.peer.kind.clone()),
        parse_and_validate_peer_kind,
    );
    step_counter += 1;

    // Get icon
    let icon_enabled = get_bool(
        opts.no_prompt,
        step_str(step_counter),
        opts.icon_enabled,
        ADD_PEER_ICON_ENABLED_FLAG,
        ADD_PEER_ICON_ENABLED_HELP,
        config.network.defaults.peer.icon.enabled,
    );

    let icon_src = if icon_enabled {
        get_value(
            opts.no_prompt,
            step_str(step_counter),
            opts.icon_src.clone(),
            ADD_PEER_ICON_SRC_FLAG,
            format!("\t{}", ADD_PEER_ICON_SRC_HELP).as_str(),
            config.network.defaults.peer.icon.enabled.then(|| config.network.defaults.peer.icon.src.clone()),
            parse_and_validate_peer_icon_src,
        )
    } else {
        config.network.defaults.peer.icon.src.clone()
    };
    step_counter += 1;

    // Get DNS
    let dns_enabled = get_bool(
        opts.no_prompt,
        step_str(step_counter),
        opts.dns_enabled,
        ADD_PEER_DNS_ENABLED_FLAG,
        ADD_PEER_DNS_ENABLED_HELP,
        config.network.defaults.peer.dns.enabled,
    );

    let dns_addresses = if dns_enabled {
        get_value(
            opts.no_prompt,
            step_str(step_counter),
            opts.dns_addresses.clone(),
            ADD_PEER_DNS_ADDRESSES_FLAG,
            format!("\t{}", ADD_PEER_DNS_ADDRESSES_HELP).as_str(),
            (!config.network.defaults.peer.dns.addresses.is_empty())
                .then_some(config.network.defaults.peer.dns.addresses.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",")),
            parse_and_validate_peer_dns_addresses,
        )
    } else {
        config.network.defaults.peer.dns.addresses.clone()
    };
    step_counter += 1;

    // Get MTU
    let mtu_enabled = get_bool(
        opts.no_prompt,
        step_str(step_counter),
        opts.mtu_enabled,
        ADD_PEER_MTU_ENABLED_FLAG,
        ADD_PEER_MTU_ENABLED_HELP,
        config.network.defaults.peer.mtu.enabled,
    );

    let mtu_value = if mtu_enabled {
        get_value(
            opts.no_prompt,
            step_str(step_counter),
            opts.mtu_value.clone().map(|o| o.to_string()),
            ADD_PEER_MTU_VALUE_FLAG,
            format!("\t{}", ADD_PEER_MTU_VALUE_HELP).as_str(),
            config.network.defaults.peer.mtu.enabled.then(|| config.network.defaults.peer.mtu.value.to_string()),
            parse_and_validate_peer_mtu_value,
        )
    } else {
        // if disabled, default to an mtu of 1420
        config.network.defaults.peer.mtu.value.clone()
    };
    step_counter += 1;

    // Get scripts
    let script_pre_up = get_scripts(
        opts.no_prompt,
        step_str(step_counter),
        opts.script_pre_up_enabled,
        opts.script_pre_up_line.clone(),
        ADD_PEER_SCRIPT_PRE_UP_ENABLED_FLAG,
        ADD_PEER_SCRIPT_PRE_UP_LINE_FLAG,
        ADD_PEER_SCRIPT_PRE_UP_ENABLED_HELP,
        ADD_PEER_SCRIPT_PRE_UP_LINE_HELP,
    );
    step_counter += 1;

    let script_post_up = get_scripts(
        opts.no_prompt,
        step_str(step_counter),
        opts.script_post_up_enabled,
        opts.script_post_up_line.clone(),
        ADD_PEER_SCRIPT_POST_UP_ENABLED_FLAG,
        ADD_PEER_SCRIPT_POST_UP_LINE_FLAG,
        ADD_PEER_SCRIPT_POST_UP_ENABLED_HELP,
        ADD_PEER_SCRIPT_POST_UP_LINE_HELP,
    );
    step_counter += 1;

    let script_pre_down = get_scripts(
        opts.no_prompt,
        step_str(step_counter),
        opts.script_pre_down_enabled,
        opts.script_pre_down_line.clone(),
        ADD_PEER_SCRIPT_PRE_DOWN_ENABLED_FLAG,
        ADD_PEER_SCRIPT_PRE_DOWN_LINE_FLAG,
        ADD_PEER_SCRIPT_PRE_DOWN_ENABLED_HELP,
        ADD_PEER_SCRIPT_PRE_DOWN_LINE_HELP,
    );
    step_counter += 1;

    let script_post_down = get_scripts(
        opts.no_prompt,
        step_str(step_counter),
        opts.script_post_down_enabled,
        opts.script_post_down_line.clone(),
        ADD_PEER_SCRIPT_POST_DOWN_ENABLED_FLAG,
        ADD_PEER_SCRIPT_POST_DOWN_LINE_FLAG,
        ADD_PEER_SCRIPT_POST_DOWN_ENABLED_HELP,
        ADD_PEER_SCRIPT_POST_DOWN_LINE_HELP,
    );

    // Create the peer
    let peer = Peer {
        name: peer_name.clone(),
        address: peer_address,
        endpoint: Endpoint {
            enabled: endpoint_enabled,
            address: endpoint_address,
        },
        kind: peer_kind,
        icon: Icon {
            enabled: icon_enabled,
            src: icon_src,
        },
        dns: Dns {
            enabled: dns_enabled,
            addresses: dns_addresses,
        },
        mtu: Mtu {
            enabled: mtu_enabled,
            value: mtu_value,
        },
        scripts: Scripts {
            pre_up: script_pre_up,
            post_up: script_post_up,
            pre_down: script_pre_down,
            post_down: script_post_down,
        },
        private_key: wg_generate_key(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Add peer to the network
    config.network.peers.insert(peer_id, peer);
    config.network.updated_at = Utc::now();
    conf::util::set_config(&mut config)?;
    log::info!("Successfully added peer {} ({})", peer_name, peer_id);

    // Get peers to connect to
    let peers_to_connect = if opts.no_prompt == Some(true) {
        vec![]
    } else {
        let peer_items: Vec<String> = config.network.peers.iter()
            .filter(|(id, _)| **id != peer_id)
            .map(|(id, p)| format!("{} ({})", p.name, id))
            .collect();

        if peer_items.is_empty() {
            vec![]
        } else {
            // Ask if user wants to connect to any peers
            let want_connections = dialoguer::Confirm::new()
                .with_prompt("Do you want to connect this peer to any existing peers?")
                .default(true)
                .interact()
                .map_err(|e| ConfigCommandError::ReadFailed(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

            if !want_connections {
                vec![]
            } else {
                let defaults: Vec<bool> = config.network.peers.iter()
                    .map(|(id, _)| *id == config.network.this_peer)
                    .collect();

                // Ask which peers to connect to
                let selections = dialoguer::MultiSelect::new()
                    .with_prompt("Select peers to connect to (space to select, enter to confirm)")
                    .items(&peer_items)
                    .defaults(&defaults)
                    .report(false)
                    .interact()
                    .map_err(|e| ConfigCommandError::ReadFailed(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                // Print the selected items
                println!("Selected peers:");
                for &index in &selections {
                    println!(" ✓ {}", peer_items[index]);
                }

                selections.iter()
                    .filter_map(|&idx| {
                        config.network.peers.keys()
                            .filter(|id| **id != peer_id)
                            .nth(idx)
                            .copied()
                    })
                    .collect()
            }
        }
    };

    // Create connections
    let step_str = make_step_formatter(peers_to_connect.len());
    for (i, other_peer_id) in peers_to_connect.iter().enumerate() {
        println!("{} Connecting {} ({}) to {} ({})", step_str(i+1).trim(), peer_name, peer_id, config.network.peers.get(other_peer_id).unwrap().name, other_peer_id);
        add_connection(&AddConnectionOptions{
            no_prompt: opts.no_prompt,
            first_peer: Some(peer_id),
            second_peer: Some(other_peer_id.clone()),
            persistent_keepalive_enabled: None,
            persistent_keepalive_period: None,
            allowed_ips_first_to_second: None,
            allowed_ips_second_to_first: None,
        })?;
    }

    Ok(())
}

/// Add a new connection between two peers
pub fn add_connection(opts: &AddConnectionOptions) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let mut step_counter = 1;
    let step_str = make_step_formatter(3);

    // Get first peer
    let first_peer_id = if let Some(peer_id) = opts.first_peer {
        peer_id
    } else if opts.no_prompt == Some(true) {
        return Err(ConfigCommandError::MissingArgument("first_peer".to_string()));
    } else {
        let peer_items: Vec<String> = config.network.peers.iter()
            .map(|(id, p)| format!("{} ({})", p.name, id))
            .collect();

        if peer_items.is_empty() {
            return Err(ConfigCommandError::MissingArgument("No peers available".to_string()));
        }

        let selection = dialoguer::Select::new()
            .with_prompt("Select first peer")
            .items(&peer_items)
            .interact()
            .map_err(|e| ConfigCommandError::ReadFailed(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        // TODO: If more than one peer is in the network, skip ones that are already connected to all other peers

        *config.network.peers.keys().nth(selection).unwrap()
    };
    let first_peer_name = &config.network.peers.get(&first_peer_id).ok_or(ConfigCommandError::PeerNotFound(first_peer_id.clone()))?.name;

    // Get peer B
    let second_peer_id = if let Some(peer_id) = opts.second_peer {
        if peer_id == first_peer_id {
            return Err(ConfigCommandError::MissingArgument("peer_a and peer_b cannot be the same".to_string()));
        }
        peer_id.clone()
    } else if opts.no_prompt == Some(true) {
        return Err(ConfigCommandError::MissingArgument("peer_b".to_string()));
    } else {
        let peer_items: Vec<String> = config.network.peers.iter()
            .filter(|(id, _)| **id != first_peer_id)
            .map(|(id, p)| format!("{} ({})", p.name, id))
            .collect();
        // TODO: also filter out if this connection already exists

        if peer_items.is_empty() {
            return Err(ConfigCommandError::MissingArgument("No other peers available".to_string()));
        }

        let selection = dialoguer::Select::new()
            .with_prompt("Select second peer")
            .items(&peer_items)
            .interact()
            .map_err(|e| ConfigCommandError::ReadFailed(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        *config.network.peers.keys()
            .filter(|id| **id != first_peer_id)
            .nth(selection)
            .unwrap()
    };
    let second_peer_name = &config.network.peers.get(&second_peer_id).ok_or(ConfigCommandError::PeerNotFound(second_peer_id.clone()))?.name;

    // Create connection ID (always store with smaller UUID first)
    let conn_id = get_connection_id(first_peer_id, second_peer_id);

    // Check if the connection already exists
    if config.network.connections.contains_key(&conn_id) {
        return Err(ConfigCommandError::MissingArgument(format!("Connection already exists: {}", conn_id)));
    }

    let first_peer_disp = format!("{} ({})", first_peer_name, first_peer_id);
    let second_peer_disp = format!("{} ({})", second_peer_name, second_peer_id);

    // Get allowed IPs A to B
    let ips_first_to_second = get_value(
        opts.no_prompt,
        step_str(step_counter),
        opts.allowed_ips_first_to_second.clone(),
        ADD_CONNECTION_ALLOWED_IPS_FIRST_TO_SECOND_FLAG,
        &ADD_CONNECTION_ALLOWED_IPS_FIRST_TO_SECOND_HELP.replace("the first peer", &first_peer_disp).replace("the second peer", &second_peer_disp),
        Some(format!("{}/32", config.network.peers.get(&second_peer_id).unwrap().address)),
        parse_and_validate_conn_allowed_ips,
    );
    step_counter += 1;

    // Get allowed IPs B to A
    let ips_second_to_first = get_value(
        opts.no_prompt,
        step_str(step_counter),
        opts.allowed_ips_second_to_first.clone(),
        ADD_CONNECTION_ALLOWED_IPS_SECOND_TO_FIRST_FLAG,
        &ADD_CONNECTION_ALLOWED_IPS_SECOND_TO_FIRST_HELP.replace("the first peer", &first_peer_disp).replace("the second peer", &second_peer_disp),
        Some(format!("{}/32", config.network.peers.get(&first_peer_id).unwrap().address)),
        parse_and_validate_conn_allowed_ips,
    );
    step_counter += 1;

    // Get persistent keepalive
    let keepalive_enabled = get_bool(
        opts.no_prompt,
        step_str(step_counter),
        opts.persistent_keepalive_enabled,
        ADD_CONNECTION_PERSISTENT_KEEPALIVE_ENABLED_FLAG,
        ADD_CONNECTION_PERSISTENT_KEEPALIVE_ENABLED_HELP,
        config.network.defaults.connection.persistent_keepalive.enabled,
    );

    let keepalive_period = if keepalive_enabled {
        if let Some(p) = opts.persistent_keepalive_period {
            validate_conn_persistent_keepalive_period(p)?
        } else {
            get_value(
                opts.no_prompt,
                step_str(step_counter),
                None,
                ADD_CONNECTION_PERSISTENT_KEEPALIVE_PERIOD_FLAG,
                format!("\t{}", ADD_CONNECTION_PERSISTENT_KEEPALIVE_PERIOD_HELP).as_str(),
                Some(config.network.defaults.connection.persistent_keepalive.period.to_string()),
                parse_and_validate_conn_persistent_keepalive_period,
            )
        }
    } else {
        config.network.defaults.connection.persistent_keepalive.period.clone()
    };

    // Create connection
    let (allowed_ips_a_to_b, allowed_ips_b_to_a) = if conn_id.a == first_peer_id {
        (ips_first_to_second, ips_second_to_first)
    } else {
        (ips_second_to_first, ips_first_to_second)
    };

    let connection = Connection {
        enabled: true,
        pre_shared_key: wg_generate_key(),
        persistent_keepalive: PersistentKeepalive {
            enabled: keepalive_enabled,
            period: keepalive_period,
        },
        allowed_ips_a_to_b,
        allowed_ips_b_to_a,
    };

    config.network.connections.insert(conn_id.clone(), connection);
    config.network.updated_at = Utc::now();
    conf::util::set_config(&mut config)?;
    log::info!("Successfully added connection {}", conn_id);
    Ok(())
}
