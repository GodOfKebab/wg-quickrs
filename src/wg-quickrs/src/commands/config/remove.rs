// ============================================================================
// Remove Functions - Delete network entities
// ============================================================================

use std::net::Ipv4Addr;
use uuid::Uuid;
use crate::commands::config::{parse_connection_id, ConfigCommandError};
use crate::conf;

/// Remove a peer from the network by UUID
pub fn remove_network_peer(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;

    // Check if trying to remove this_peer
    if id == &config.network.this_peer {
        return Err(ConfigCommandError::CannotRemoveThisPeer(*id));
    }

    // Check if peer exists
    if !config.network.peers.contains_key(id) {
        return Err(ConfigCommandError::PeerNotFound(*id));
    }

    // Get peer for logging
    let peer = config.network.peers.get(id).ok_or_else(|| {
        log::error!("Failed to get peer {} after confirming existence", id);
        ConfigCommandError::PeerNotFound(*id)
    })?;
    let peer_name = peer.name.clone();

    // Remove the peer
    config.network.peers.remove(id);

    // Also remove any connections involving this peer
    config.network.connections.retain(|conn_id, _| {
        !conn_id.contains(id)
    });

    // Also remove any reservations for this peer
    config.network.reservations.retain(|_, reservation| {
        reservation.peer_id != *id
    });

    log::info!("Removed peer {} ({})", peer_name, id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Remove a connection from the network by connection ID
pub fn remove_network_connection(id_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;

    // Check if connection exists
    if !config.network.connections.contains_key(&conn_id) {
        return Err(ConfigCommandError::ConnectionNotFound(id_str.to_string()));
    }

    // Remove the connection
    config.network.connections.remove(&conn_id);

    log::info!("Removed connection {}", id_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Remove a reservation from the network by IPv4 address
pub fn remove_network_reservation(address: &Ipv4Addr) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;

    // Check if reservation exists
    if !config.network.reservations.contains_key(address) {
        return Err(ConfigCommandError::ReservationNotFound(*address));
    }

    // Get reservation for logging
    let reservation = config.network.reservations.get(address).ok_or_else(|| {
        log::error!("Failed to get reservation {} after confirming existence", address);
        ConfigCommandError::ReservationNotFound(*address)
    })?;
    let peer_id = reservation.peer_id;

    // Remove the reservation
    config.network.reservations.remove(address);

    log::info!("Removed reservation {} (peer: {})", address, peer_id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

