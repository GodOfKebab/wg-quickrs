// ============================================================================
// List Functions - Human-readable output
// ============================================================================

use crate::commands::config::ConfigCommandError;
use crate::conf;

/// Helper function to format EndpointAddress for display
fn format_endpoint_address(addr: &wg_quickrs_lib::types::network::EndpointAddress) -> String {
    use wg_quickrs_lib::types::network::EndpointAddress;
    match addr {
        EndpointAddress::None => "none".to_string(),
        EndpointAddress::Ipv4AndPort(ip_port) => format!("{}:{}", ip_port.ipv4, ip_port.port),
        EndpointAddress::HostnameAndPort(host_port) => format!("{}:{}", host_port.hostname, host_port.port),
    }
}

/// List all peers in human-readable format
/// Format: "name (peerid) @ address / {endpoint if enabled}"
pub fn list_network_peers() -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    if config.network.peers.is_empty() {
        println!("No peers found.");
        return Ok(());
    }

    for (peer_id, peer) in &config.network.peers {
        let endpoint_str = if peer.endpoint.enabled {
            format!(" / {}", format_endpoint_address(&peer.endpoint.address))
        } else {
            String::new()
        };

        println!("{} ({}) @ {}{}", peer.name, peer_id, peer.address, endpoint_str);
    }

    Ok(())
}

/// List all connections in human-readable format
/// Format: "name1<->name2 (connectionid)"
pub fn list_network_connections() -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    if config.network.connections.is_empty() {
        println!("No connections found.");
        return Ok(());
    }

    for conn_id in config.network.connections.keys() {
        // Get peer names
        let peer_a_name = config.network.peers.get(&conn_id.a)
            .map(|p| p.name.as_str())
            .unwrap_or("unknown");
        let peer_b_name = config.network.peers.get(&conn_id.b)
            .map(|p| p.name.as_str())
            .unwrap_or("unknown");

        println!("{}<->{} ({}*{})", peer_a_name, peer_b_name, conn_id.a, conn_id.b);
    }

    Ok(())
}

/// List all reservations in human-readable format
/// Format: "address (peerid) valid until: {valid_until}"
pub fn list_network_reservations() -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    if config.network.reservations.is_empty() {
        println!("No reservations found.");
        return Ok(());
    }

    for (address, reservation) in &config.network.reservations {
        println!("{} ({}) valid until: {}", address, reservation.peer_id, reservation.valid_until);
    }

    Ok(())
}
