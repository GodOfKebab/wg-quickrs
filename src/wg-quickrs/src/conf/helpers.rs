use std::net::Ipv4Addr;
use wg_quickrs_lib::types::network::Network;


pub(crate) fn get_allocated_addresses(network: &Network) -> Vec<Ipv4Addr> {
    network.peers.values()
        .map(|peer| peer.address.clone())
        .chain(network.reservations.keys().cloned())
        .collect()
}