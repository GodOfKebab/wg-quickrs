use std::collections::HashMap;
use chrono::Duration;
use wg_quickrs_wasm::validation::*;
use wg_quickrs_wasm::types::*;
use wg_quickrs_wasm::timestamp;


/// Helper macro for passing tests
macro_rules! ok {
    ($res:expr) => {
        assert!($res.status, "Expected OK but got {:?}", $res)
    };
}

/// Helper macro for error message checks
macro_rules! err_contains {
    ($res:expr, $msg:expr) => {
        assert!(
            !$res.status && $res.msg.contains($msg),
            "Expected error containing '{}', got {:?}",
            $msg,
            $res
        )
    };
}

#[test]
fn test_is_fqdn_with_port() {
    assert!(is_fqdn_with_port("hostname:443"));
    assert!(is_fqdn_with_port("example.com:443"));
    assert!(!is_fqdn_with_port(":443")); // no hostname
    assert!(!is_fqdn_with_port("example.com")); // no port
    assert!(!is_fqdn_with_port("example.com:99999")); // invalid port
    assert!(!is_fqdn_with_port("example.com:abc")); // invalid port
    assert!(!is_fqdn_with_port("256.0.0.1:80")); // invalid hostname
}

// ------------------------
// check_field_str
// ------------------------
#[test]
fn test_check_name() {
    ok!(check_field_str("name", "test-name"));
    err_contains!(check_field_str("name", ""), "name cannot be empty");
}

fn generate_peer(name: &str, address: &str) -> Peer {
    Peer {
        name: name.to_string(),
        address: address.to_string(),
        endpoint: Default::default(),
        kind: "".to_string(),
        icon: Default::default(),
        dns: Default::default(),
        mtu: Default::default(),
        scripts: Default::default(),
        private_key: "".to_string(),
        created_at: "".to_string(),
        updated_at: "".to_string(),
    }
}

fn generate_network(peers: HashMap<String, Peer>, subnet: &str, reservations: HashMap<String, ReservationData>) -> Network {
    Network {
        identifier: "".to_string(),
        subnet: subnet.to_string(),
        this_peer: "".to_string(),
        peers,
        connections: Default::default(),
        defaults: Default::default(),
        reservations,
        updated_at: "".to_string(),
    }
}

#[test]
fn test_check_internal_address() {
    // Invalid IPv4 address
    let network = generate_network(HashMap::new(), "10.0.0.0/24", HashMap::new());
    err_contains!(
        check_internal_address("not-an-ip", &network),
        "address is not IPv4"
    );
    err_contains!(
        check_internal_address("999.999.999.999", &network),
        "address is not IPv4"
    );
    err_contains!(
        check_internal_address("10.0.0", &network),
        "address is not IPv4"
    );

    // Invalid CIDR subnet format in network
    let network = generate_network(HashMap::new(), "not-cidr", HashMap::new());
    err_contains!(
        check_internal_address("10.0.0.5", &network),
        "network subnet is not in CIDR format"
    );
    let network = generate_network(HashMap::new(), "10.0.0.0", HashMap::new());
    err_contains!(
        check_internal_address("10.0.0.5", &network),
        "network subnet is not in CIDR format"
    );
    let network = generate_network(HashMap::new(), "10.0.0.0/33", HashMap::new());
    err_contains!(
        check_internal_address("10.0.0.5", &network),
        "network subnet is not in CIDR format"
    );

    // Address not in subnet
    let network = generate_network(HashMap::new(), "10.0.0.0/24", HashMap::new());
    err_contains!(
        check_internal_address("192.168.1.10", &network),
        "address is not in the network subnet"
    );
    err_contains!(
        check_internal_address("10.0.1.5", &network),
        "address is not in the network subnet"
    );

    // Address taken by a peer
    let peers = vec![generate_peer("Alice", "10.0.0.5")]
        .into_iter()
        .enumerate()
        .map(|(i, peer)| (i.to_string(), peer))
        .collect();
    let network = generate_network(peers, "10.0.0.0/24", HashMap::new());
    err_contains!(
        check_internal_address("10.0.0.5", &network),
        "address is already taken by Alice(0)"
    );

    // Address reserved with a valid reservation (future timestamp)
    let mut reservations = HashMap::new();
    let future_time = timestamp::get_future_timestamp_formatted(Duration::minutes(10));
    reservations.insert("10.0.0.10".into(), ReservationData {
        peer_id: "".to_string(),
        valid_until: future_time,
    });
    let network = generate_network(HashMap::new(), "10.0.0.0/24", reservations);
    err_contains!(
        check_internal_address("10.0.0.10", &network),
        "address is reserved for another peer"
    );

    // Address with expired reservation (past timestamp) - should succeed
    let mut reservations = HashMap::new();
    let past_time = timestamp::get_future_timestamp_formatted(Duration::minutes(-10));
    reservations.insert("10.0.0.10".into(), ReservationData {
        peer_id: "".to_string(),
        valid_until: past_time,
    });
    let network = generate_network(HashMap::new(), "10.0.0.0/24", reservations);
    ok!(check_internal_address("10.0.0.10", &network));

    // Address with invalid timestamp format in the reservation
    let mut reservations = HashMap::new();
    reservations.insert("10.0.0.10".into(), ReservationData {
        peer_id: "".to_string(),
        valid_until: "invalid-timestamp".to_string(),
    });
    let network = generate_network(HashMap::new(), "10.0.0.0/24", reservations);
    err_contains!(
        check_internal_address("10.0.0.10", &network),
        "failed to parse reservation validity period"
    );

    // Valid address (success case)
    let network = generate_network(HashMap::new(), "10.0.0.0/24", HashMap::new());
    ok!(check_internal_address("10.0.0.20", &network));

    // Edge case: Valid address with peers and expired reservations, but not conflicting
    let peers = vec![generate_peer("Bob", "10.0.0.5")]
        .into_iter()
        .enumerate()
        .map(|(i, peer)| (i.to_string(), peer))
        .collect();
    let mut reservations = HashMap::new();
    let past_time = timestamp::get_future_timestamp_formatted(Duration::minutes(-10));
    reservations.insert("10.0.0.10".into(), ReservationData {
        peer_id: "".to_string(),
        valid_until: past_time,
    });
    let network = generate_network(peers, "10.0.0.0/24", reservations);
    ok!(check_internal_address("10.0.0.30", &network));

    // Edge case: Network boundaries
    // Edge case: Network and broadcast addresses should fail
    err_contains!(
        check_internal_address("10.0.0.0", &network),
        "address is the network address and cannot be assigned"
    );
    err_contains!(
        check_internal_address("10.0.0.255", &network),
        "address is the broadcast address and cannot be assigned"
    );
}

#[test]
fn test_check_public_private_pre_shared_keys() {
    ok!(check_field_str("private_key", "qBZArZg+2vEvD5tS8T7m0H0/xvd1PKdoBHXWIrQ1DEE="));
    err_contains!(
        check_field_str("private_key", "def"),
        "private_key is not base64 encoded with 32 bytes"
    );
    err_contains!(
        check_field_str("private_key", ""),
        "private_key is not base64 encoded with 32 bytes"
    );

    ok!(check_field_str("pre_shared_key", "ySMLaPaHVrxg/rdmZlGUemyt2JKxwSdeYUa3l34RwbE="));
    err_contains!(
        check_field_str("pre_shared_key", "ghi"),
        "pre_shared_key is not base64 encoded with 32 bytes"
    );
    err_contains!(
        check_field_str("pre_shared_key", ""),
        "pre_shared_key is not base64 encoded with 32 bytes"
    );
}

#[test]
fn test_check_allowed_ips() {
    ok!(check_field_str("allowed_ips_a_to_b", "10.0.0.0/24"));
    ok!(check_field_str("allowed_ips_b_to_a", "10.0.0.0/24, 192.168.1.0/24"));
    err_contains!(
        check_field_str("allowed_ips_a_to_b", "invalid"),
        "AllowedIPs is not in CIDR format"
    );
}

#[test]
fn test_unknown_field() {
    err_contains!(
        check_field_str("does_not_exist", "anything"),
        "field doesn't exist"
    );
}

// ------------------------
// check_field_enabled_value
// ------------------------
fn ev(enabled: bool, val: &str) -> EnabledValue {
    EnabledValue {
        enabled,
        value: val.into(),
    }
}

#[test]
fn test_endpoint_ipv4_or_fqdn() {
    ok!(check_field_enabled_value("endpoint", &ev(false, ""))); // disabled = ok
    ok!(check_field_enabled_value("endpoint", &ev(true, "10.0.0.1:51820")));
    ok!(check_field_enabled_value("endpoint", &ev(true, "example.com:51820")));
    err_contains!(
        check_field_enabled_value("endpoint", &ev(true, "notvalid")),
        "endpoint is not IPv4 nor an FQDN"
    );
}

#[test]
fn test_dns() {
    ok!(check_field_enabled_value("dns", &ev(false, ""))); // disabled = ok
    ok!(check_field_enabled_value("dns", &ev(true, "8.8.8.8")));
    ok!(check_field_enabled_value("dns", &ev(true, "8.8.8.8, 1.1.1.1")));
    err_contains!(
        check_field_enabled_value("dns", &ev(true, "8.8.8.8, not-an-ip")),
        "DNS is invalid"
    );
    err_contains!(
        check_field_enabled_value("dns", &ev(true, "not-an-ip")),
        "DNS is invalid"
    );
}

#[test]
fn test_mtu() {
    ok!(check_field_enabled_value("mtu", &ev(false, ""))); // disabled = ok
    ok!(check_field_enabled_value("mtu", &ev(true, "1500")));
    err_contains!(
        check_field_enabled_value("mtu", &ev(true, "-1")),
        "MTU is invalid"
    );
    err_contains!(
        check_field_enabled_value("mtu", &ev(true, "10000")),
        "MTU is invalid"
    );
    err_contains!(
        check_field_enabled_value("mtu", &ev(true, "70000")),
        "MTU is invalid"
    );
    err_contains!(
        check_field_enabled_value("mtu", &ev(true, "notnum")),
        "MTU is invalid"
    );
}

#[test]
fn test_scripts() {
    ok!(check_field_enabled_value("pre_up", &ev(false, "")));
    ok!(check_field_enabled_value("post_up", &ev(true, "echo ok;")));
    err_contains!(
        check_field_enabled_value("pre_up", &ev(true, "echo missing_semicolon")),
        "script needs to end with a semicolon"
    );
}

#[test]
fn test_persistent_keepalive() {
    ok!(check_field_enabled_value("persistent_keepalive", &ev(false, ""))); // disabled = ok
    ok!(check_field_enabled_value("persistent_keepalive", &ev(true, "25")));
    err_contains!(
        check_field_enabled_value("persistent_keepalive", &ev(true, "-1")),
        "Persistent Keepalive is invalid"
    );
    err_contains!(
        check_field_enabled_value("persistent_keepalive", &ev(true, "70000")),
        "Persistent Keepalive is invalid"
    );
    err_contains!(
        check_field_enabled_value("persistent_keepalive", &ev(true, "notnum")),
        "Persistent Keepalive is invalid"
    );
}

#[test]
fn test_unknown_enabled_field() {
    err_contains!(
        check_field_enabled_value("does_not_exist", &ev(true, "")),
        "field doesn't exist"
    );
}
