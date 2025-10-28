use chrono::{Duration, Utc};
use wg_quickrs_wasm::types::Network;

pub(crate) fn remove_expired_reservations(network: &mut Network) {
    network.reservations.retain(|_, lease_data| {
        Utc::now().signed_duration_since(&lease_data.valid_until) < Duration::zero()
    });
}

pub(crate) fn get_allocated_addresses(network: &Network) -> Vec<String> {
    network.peers.values()
        .map(|peer| peer.address.clone())
        .chain(network.reservations.keys().cloned())
        .collect()
}