use std::collections::HashSet;
use std::net::Ipv4Addr;
use wg_quickrs_lib::types::network::Network;


/// Get the next available IPv4 address in subnet not ending with .0 or .255 and not in `taken`
pub fn get_next_available_address(network: &Network) -> Option<Ipv4Addr> {
    let taken: HashSet<Ipv4Addr> = network.peers.values()
        .map(|peer| peer.address)
        .chain(network.reservations.keys().cloned())
        .collect();
    
    network.subnet.hosts().find(|&ip| !taken.contains(&ip))
}
