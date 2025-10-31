use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use chrono::{Duration, Utc};
use uuid::Uuid;
use wg_quickrs_wasm::validation::network::*;
use wg_quickrs_wasm::validation::agent::*;
use wg_quickrs_wasm::validation::error::*;
use wg_quickrs_wasm::types::network::*;


/// Helper macro for passing tests
macro_rules! ok {
    ($res:expr) => {
        assert!($res.is_ok(), "Expected OK but got {:?}", $res)
    };
}

/// Helper macro for error message checks
macro_rules! is_err {
    ($res:expr, $exp_err:expr) => {
        assert!(
            $res.err() == Some($exp_err),
            "Expected error '{}', got {:?}",
            $exp_err,
            $res
        )
    };
}

// Agent Fields

#[test]
fn test_validate_ipv4_address() {
    ok!(validate_ipv4_address("10.0.0.1"));
    is_err!(
        validate_ipv4_address("not-an-ip"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        validate_ipv4_address("999.999.999.999"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        validate_ipv4_address("10.0.0"),
        ValidationError::NotIPv4Address()
    );
}

#[test]
fn test_validate_port() {
    ok!(validate_port("80"));
    is_err!(
        validate_port("not-a-port"),
        ValidationError::NotPortNumber()
    );
    is_err!(
        validate_port("70000"),
        ValidationError::NotPortNumber()
    );
    is_err!(
        validate_port("-1"),
        ValidationError::NotPortNumber()
    );
}

// Network Fields

#[test]
fn test_validate_network_name() {
    ok!(validate_network_name("test-name"));
    is_err!(validate_network_name(""), ValidationError::EmptyNetworkName());
}

#[test]
fn test_validate_ipv4_subnet() {
    ok!(validate_ipv4_subnet("10.0.0.0/24"));
    is_err!(
        validate_ipv4_subnet("not-a-cidr"),
        ValidationError::NotCIDR()
    );
    is_err!(
        validate_ipv4_subnet("999.999.999.999/24"),
        ValidationError::NotCIDR()
    );
    is_err!(
        validate_ipv4_subnet("10.0.0/24"),
        ValidationError::NotCIDR()
    );
    is_err!(
        validate_ipv4_subnet("10.0.0.0/-1"),
        ValidationError::NotCIDR()
    );
    is_err!(
        validate_ipv4_subnet("10.0.0.0/33"),
        ValidationError::NotCIDR()
    );
}

#[test]
fn test_validate_peer_id() {
    ok!(validate_peer_id(&Uuid::new_v4().to_string()));
    is_err!(validate_peer_id("not-a-uuid"), ValidationError::InvalidUuid());
}

// Network.Peer Fields

#[test]
fn test_validate_name() {
    ok!(validate_peer_name("test-name"));
    is_err!(validate_peer_name(""), ValidationError::EmptyPeerName());
}

fn generate_peer(name: &str, address: &str) -> Peer {
    Peer {
        name: name.to_string(),
        address: address.parse().unwrap(),
        endpoint: Default::default(),
        kind: Default::default(),
        icon: Default::default(),
        dns: Default::default(),
        mtu: Default::default(),
        scripts: Default::default(),
        private_key: Default::default(),
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

fn generate_network(peers: BTreeMap<Uuid, Peer>, subnet: &str, reservations: BTreeMap<String, ReservationData>) -> Network {
    Network {
        name: Default::default(),
        subnet: subnet.parse().unwrap(),
        this_peer: Default::default(),
        peers,
        connections: Default::default(),
        defaults: Default::default(),
        reservations: reservations.into_iter()
            .map(|(k, v)| (k.parse::<Ipv4Addr>().unwrap(), v))
            .collect(),
        updated_at: Default::default(),
    }
}

#[test]
fn test_validate_peer_address() {
    // Invalid IPv4 address
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    is_err!(
        validate_peer_address("not-an-ip", &network),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        validate_peer_address("999.999.999.999", &network),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        validate_peer_address("10.0.0", &network),
        ValidationError::NotIPv4Address()
    );

    // Address not in subnet
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    is_err!(
        validate_peer_address("192.168.1.10", &network),
        ValidationError::AddressNotInSubnet()
    );
    is_err!(
        validate_peer_address("10.0.1.5", &network),
        ValidationError::AddressNotInSubnet()
    );

    // Address taken by a peer
    let alice_peer_id = Uuid::new_v4();
    let network = generate_network(
        BTreeMap::from([(alice_peer_id, generate_peer("Alice", "10.0.0.5"))]),
        "10.0.0.0/24",
        BTreeMap::new());
    is_err!(
        validate_peer_address("10.0.0.5", &network),
        ValidationError::AddressIsTaken(alice_peer_id, "Alice".to_string())
    );

    // Address reserved with a valid reservation (future timestamp)
    let network = generate_network(
        BTreeMap::new(),
        "10.0.0.0/24",
        BTreeMap::from([("10.0.0.10".into(), ReservationData {
            peer_id: Uuid::new_v4(),
            valid_until: Utc::now() + Duration::minutes(10),
        })]));
    is_err!(
        validate_peer_address("10.0.0.10", &network),
        ValidationError::AddressIsReserved()
    );

    // Address with expired reservation (past timestamp) - should succeed
    let network = generate_network(
        BTreeMap::new(),
        "10.0.0.0/24",
        BTreeMap::from([("10.0.0.10".into(), ReservationData {
            peer_id: Uuid::new_v4(),
            valid_until: Utc::now() - Duration::minutes(10),
        })]));
    ok!(validate_peer_address("10.0.0.10", &network));

    // Valid address (success case)
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    ok!(validate_peer_address("10.0.0.20", &network));

    // Edge case: Valid address with peers and reservations, but not conflicting
    let network = generate_network(
        BTreeMap::from([(alice_peer_id, generate_peer("Alice", "10.0.0.5"))]),
        "10.0.0.0/24",
        BTreeMap::from([("10.0.0.10".into(), ReservationData {
            peer_id: Uuid::new_v4(),
            valid_until: Utc::now() - Duration::minutes(10),
        })]));
    ok!(validate_peer_address("10.0.0.30", &network));

    // Edge case: Network boundaries
    is_err!(
        validate_peer_address("10.0.0.0", &network),
        ValidationError::AddressIsSubnetNetwork()
    );
    is_err!(
        validate_peer_address("10.0.0.255", &network),
        ValidationError::AddressIsSubnetBroadcast()
    );
}

#[test]
fn test_validate_peer_endpoint() {
    ok!(validate_peer_endpoint(false, "")); // disabled = ok
    ok!(validate_peer_endpoint(true, "10.0.0.1:51820"));
    ok!(validate_peer_endpoint(true, "YOUR-SERVER:51820"));
    ok!(validate_peer_endpoint(true, "example.com:51820"));
    is_err!(
        validate_peer_endpoint(true, "notvalid"),
        ValidationError::InvalidEndpoint()
    );
}

#[test]
fn test_validate_peer_kind() {
    ok!(validate_peer_kind("anything"));
}

#[test]
fn test_validate_peer_icon() {
    ok!(validate_peer_icon(false, ""));
    ok!(validate_peer_icon(false, "anything"));
    ok!(validate_peer_icon(true, "anything"));
    is_err!(
        validate_peer_icon(true, ""),
        ValidationError::EmptyIcon()
    );
}


#[test]
fn test_validate_peer_dns() {
    ok!(validate_peer_dns(false, "")); // disabled = ok
    ok!(validate_peer_dns(true, "8.8.8.8"));
    ok!(validate_peer_dns(true, "8.8.8.8, 1.1.1.1"));
    is_err!(
        validate_peer_dns(true, "8.8.8.8, not-an-ip"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        validate_peer_dns(true, "not-an-ip"),
        ValidationError::NotIPv4Address()
    );
}

#[test]
fn test_validate_peer_mtu() {
    ok!(validate_peer_mtu(false, "")); // disabled = ok
    ok!(validate_peer_mtu(true, "1500"));
    ok!(validate_peer_mtu(true, "10000"));
    is_err!(
        validate_peer_mtu(true, "-1"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        validate_peer_mtu(true, "10001"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        validate_peer_mtu(true, "70000"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        validate_peer_mtu(true, "notnum"),
        ValidationError::InvalidMtu()
    );
}

#[test]
fn test_validate_peer_script() {
    ok!(validate_peer_script(false, ""));
    ok!(validate_peer_script(true, "echo ok;"));
    is_err!(
        validate_peer_script(true, "echo missing_semicolon"),
        ValidationError::ScriptMissingSemicolon()
    );
}

#[test]
fn test_validate_wg_key() {
    ok!(validate_wg_key("qBZArZg+2vEvD5tS8T7m0H0/xvd1PKdoBHXWIrQ1DEE="));
    is_err!(
        validate_wg_key("def"),
        ValidationError::NotWireGuardKey()
    );
    is_err!(
        validate_wg_key(""),
        ValidationError::NotWireGuardKey()
    );

    ok!(validate_wg_key("ySMLaPaHVrxg/rdmZlGUemyt2JKxwSdeYUa3l34RwbE="));
    is_err!(
        validate_wg_key("ghi"),
        ValidationError::NotWireGuardKey()
    );
    is_err!(
        validate_wg_key(""),
        ValidationError::NotWireGuardKey()
    );
}

// Network.Connection Fields

#[test]
fn test_validate_conn_persistent_keepalive() {
    ok!(validate_conn_persistent_keepalive(false, "")); // disabled = ok
    ok!(validate_conn_persistent_keepalive(true, "25"));
    is_err!(
        validate_conn_persistent_keepalive(true, "-1"),
        ValidationError::InvalidPersistentKeepalive()
    );
    is_err!(
        validate_conn_persistent_keepalive(true, "70000"),
        ValidationError::InvalidPersistentKeepalive()
    );
    is_err!(
        validate_conn_persistent_keepalive(true, "notnum"),
        ValidationError::InvalidPersistentKeepalive()
    );
}

#[test]
fn test_validate_conn_allowed_ips() {
    ok!(validate_conn_allowed_ips("10.0.0.0/24"));
    ok!(validate_conn_allowed_ips("10.0.0.0/24, 192.168.1.0/24"));
    is_err!(
        validate_conn_allowed_ips("invalid"),
        ValidationError::InvalidAllowedIPs()
    );
}
