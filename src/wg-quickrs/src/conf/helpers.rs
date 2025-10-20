use chrono::Duration;
use wg_quickrs_wasm::timestamp::get_duration_since_formatted;
use wg_quickrs_wasm::types::Network;

pub(crate) fn remove_expired_reservations(network: &mut Network) {
    network.reservations.retain(|_, lease_data| {
        get_duration_since_formatted(&lease_data.valid_until)
            .map(|duration| duration < Duration::zero())
            .unwrap_or(false) // if the timestamp is invalid, assume it's expired
    });
}

pub(crate) fn get_allocated_addresses(network: &Network) -> Vec<String> {
    network.peers.values()
        .map(|peer| peer.address.clone())
        .chain(network.reservations.keys().cloned())
        .collect()
}