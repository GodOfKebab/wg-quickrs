use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use chrono::{Duration, Utc};
use uuid::Uuid;
use wg_quickrs_lib::validation::network::*;
use wg_quickrs_lib::validation::agent::*;
use wg_quickrs_lib::validation::error::*;
use wg_quickrs_lib::types::network::*;


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
    ok!(parse_and_validate_ipv4_address("10.0.0.1"));
    is_err!(
        parse_and_validate_ipv4_address("not-an-ip"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_ipv4_address("999.999.999.999"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_ipv4_address("10.0.0"),
        ValidationError::NotIPv4Address()
    );
}

#[test]
fn test_validate_port() {
    ok!(parse_and_validate_port("80"));
    is_err!(
        parse_and_validate_port("not-a-port"),
        ValidationError::NotPortNumber()
    );
    is_err!(
        parse_and_validate_port("70000"),
        ValidationError::NotPortNumber()
    );
    is_err!(
        parse_and_validate_port("-1"),
        ValidationError::NotPortNumber()
    );
}

// Network Fields

#[test]
fn test_validate_network_name() {
    ok!(parse_and_validate_network_name("test-name"));
    is_err!(parse_and_validate_network_name(""), ValidationError::EmptyNetworkName());
}

#[test]
fn test_validate_ipv4_subnet() {
    ok!(parse_and_validate_ipv4_subnet("10.0.0.0/24"));
    is_err!(
        parse_and_validate_ipv4_subnet("not-a-cidr"),
        ValidationError::NotCIDR()
    );
    is_err!(
        parse_and_validate_ipv4_subnet("999.999.999.999/24"),
        ValidationError::NotCIDR()
    );
    is_err!(
        parse_and_validate_ipv4_subnet("10.0.0/24"),
        ValidationError::NotCIDR()
    );
    is_err!(
        parse_and_validate_ipv4_subnet("10.0.0.0/-1"),
        ValidationError::NotCIDR()
    );
    is_err!(
        parse_and_validate_ipv4_subnet("10.0.0.0/33"),
        ValidationError::NotCIDR()
    );
}

#[test]
fn test_validate_peer_id() {
    ok!(parse_and_validate_peer_id(&Uuid::new_v4().to_string()));
    is_err!(parse_and_validate_peer_id("not-a-uuid"), ValidationError::InvalidUuid());
}

// Network.Peer Fields

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
fn test_validate_peer_name() {
    ok!(parse_and_validate_peer_name("test-name"));
    is_err!(parse_and_validate_peer_name(""), ValidationError::EmptyPeerName());
}

#[test]
fn test_validate_peer_address() {
    // Invalid IPv4 address
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    is_err!(
        parse_and_validate_peer_address("not-an-ip", &network),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_peer_address("999.999.999.999", &network),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_peer_address("10.0.0", &network),
        ValidationError::NotIPv4Address()
    );

    // Address not in subnet
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    is_err!(
        parse_and_validate_peer_address("192.168.1.10", &network),
        ValidationError::AddressNotInSubnet()
    );
    is_err!(
        parse_and_validate_peer_address("10.0.1.5", &network),
        ValidationError::AddressNotInSubnet()
    );

    // Address taken by a peer
    let alice_peer_id = Uuid::new_v4();
    let network = generate_network(
        BTreeMap::from([(alice_peer_id, generate_peer("Alice", "10.0.0.5"))]),
        "10.0.0.0/24",
        BTreeMap::new());
    is_err!(
        parse_and_validate_peer_address("10.0.0.5", &network),
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
        parse_and_validate_peer_address("10.0.0.10", &network),
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
    ok!(parse_and_validate_peer_address("10.0.0.10", &network));

    // Valid address (success case)
    let network = generate_network(BTreeMap::new(), "10.0.0.0/24", BTreeMap::new());
    ok!(parse_and_validate_peer_address("10.0.0.20", &network));

    // Edge case: Valid address with peers and reservations, but not conflicting
    let network = generate_network(
        BTreeMap::from([(alice_peer_id, generate_peer("Alice", "10.0.0.5"))]),
        "10.0.0.0/24",
        BTreeMap::from([("10.0.0.10".into(), ReservationData {
            peer_id: Uuid::new_v4(),
            valid_until: Utc::now() - Duration::minutes(10),
        })]));
    ok!(parse_and_validate_peer_address("10.0.0.30", &network));

    // Edge case: Network boundaries
    is_err!(
        parse_and_validate_peer_address("10.0.0.0", &network),
        ValidationError::AddressIsSubnetNetwork()
    );
    is_err!(
        parse_and_validate_peer_address("10.0.0.255", &network),
        ValidationError::AddressIsSubnetBroadcast()
    );
}

#[test]
fn test_validate_peer_endpoint() {
    ok!(parse_and_validate_peer_endpoint(""));
    ok!(parse_and_validate_peer_endpoint("10.0.0.1:51820"));
    ok!(parse_and_validate_peer_endpoint("YOUR-SERVER:51820"));
    ok!(parse_and_validate_peer_endpoint("example.com:51820"));
    is_err!(
        parse_and_validate_peer_endpoint("notvalid"),
        ValidationError::InvalidEndpoint()
    );
}

#[test]
fn test_validate_endpoint_struct() {
    // Disabled endpoint should always be valid (even with None address)
    let endpoint = Endpoint {
        enabled: false,
        address: EndpointAddress::None,
    };
    ok!(validate_peer_endpoint(&endpoint));

    // Enabled endpoint with None address should fail
    let endpoint = Endpoint {
        enabled: true,
        address: EndpointAddress::None,
    };
    is_err!(
        validate_peer_endpoint(&endpoint),
        ValidationError::EmptyEndpoint()
    );

    // Enabled endpoint with a valid IPv4 address should succeed
    let endpoint = Endpoint {
        enabled: true,
        address: EndpointAddress::Ipv4AndPort(Ipv4AndPort {
            ipv4: "10.0.0.1".parse().unwrap(),
            port: 51820,
        }),
    };
    ok!(validate_peer_endpoint(&endpoint));

    // Enabled endpoint with a valid hostname should succeed
    let endpoint = Endpoint {
        enabled: true,
        address: EndpointAddress::HostnameAndPort(HostnameAndPort {
            hostname: "example.com".to_string(),
            port: 51820,
        }),
    };
    ok!(validate_peer_endpoint(&endpoint));

    // Enabled endpoint with an invalid hostname should fail
    let endpoint = Endpoint {
        enabled: true,
        address: EndpointAddress::HostnameAndPort(HostnameAndPort {
            hostname: "".to_string(),
            port: 51820,
        }),
    };
    is_err!(
        validate_peer_endpoint(&endpoint),
        ValidationError::InvalidEndpoint()
    );
}

#[test]
fn test_validate_peer_kind() {
    ok!(parse_and_validate_peer_kind("anything"));
}

#[test]
fn test_validate_peer_icon() {
    ok!(parse_and_validate_peer_icon_src("anything"));
    is_err!(
        parse_and_validate_peer_icon_src(""),
        ValidationError::EmptyIcon()
    );
}

#[test]
fn test_validate_icon_struct() {
    // Disabled icon should always be valid (even with empty src)
    let icon = Icon {
        enabled: false,
        src: "".to_string(),
    };
    ok!(validate_peer_icon(&icon));

    // Enabled icon with empty src should fail
    let icon = Icon {
        enabled: true,
        src: "".to_string(),
    };
    is_err!(
        validate_peer_icon(&icon),
        ValidationError::EmptyIcon()
    );

    // Enabled icon with valid src should succeed
    let icon = Icon {
        enabled: true,
        src: "laptop".to_string(),
    };
    ok!(validate_peer_icon(&icon));
}

#[test]
fn test_validate_peer_dns() {
    ok!(parse_and_validate_peer_dns_addresses("8.8.8.8"));
    ok!(parse_and_validate_peer_dns_addresses("8.8.8.8, 1.1.1.1"));
    is_err!(
        parse_and_validate_peer_dns_addresses(""),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_peer_dns_addresses("8.8.8.8, not-an-ip"),
        ValidationError::NotIPv4Address()
    );
    is_err!(
        parse_and_validate_peer_dns_addresses("not-an-ip"),
        ValidationError::NotIPv4Address()
    );
}

#[test]
fn test_validate_dns_struct() {
    // Disabled dns should always be valid (even with empty addresses)
    let dns = Dns {
        enabled: false,
        addresses: vec![],
    };
    ok!(validate_peer_dns(&dns));

    // Enabled dns with empty addresses should fail
    let dns = Dns {
        enabled: true,
        addresses: vec![],
    };
    is_err!(
        validate_peer_dns(&dns),
        ValidationError::EmptyDns()
    );

    // Enabled dns with valid addresses should succeed
    let dns = Dns {
        enabled: true,
        addresses: vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()],
    };
    ok!(validate_peer_dns(&dns));
}

#[test]
fn test_validate_peer_mtu() {
    ok!(parse_and_validate_peer_mtu_value("1500"));
    ok!(parse_and_validate_peer_mtu_value("10000"));
    is_err!(
        parse_and_validate_peer_mtu_value(""),
        ValidationError::InvalidMtu()
    );
    is_err!(
        parse_and_validate_peer_mtu_value("0"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        parse_and_validate_peer_mtu_value("-1"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        parse_and_validate_peer_mtu_value("10001"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        parse_and_validate_peer_mtu_value("70000"),
        ValidationError::InvalidMtu()
    );
    is_err!(
        parse_and_validate_peer_mtu_value("notnum"),
        ValidationError::InvalidMtu()
    );
}

#[test]
fn test_validate_mtu_struct() {
    // Disabled mtu should always be valid (even with invalid value)
    let mtu = Mtu {
        enabled: false,
        value: 65535, // Max u16 but > 10000 MTU limit
    };
    ok!(validate_peer_mtu(&mtu));

    // Enabled mtu with invalid value (> 10000) should fail
    let mtu = Mtu {
        enabled: true,
        value: 10001,
    };
    is_err!(
        validate_peer_mtu(&mtu),
        ValidationError::InvalidMtu()
    );

    // Enabled mtu with a valid value should succeed
    let mtu = Mtu {
        enabled: true,
        value: 1500,
    };
    ok!(validate_peer_mtu(&mtu));

    // Enabled mtu with max valid value should succeed
    let mtu = Mtu {
        enabled: true,
        value: 10000,
    };
    ok!(validate_peer_mtu(&mtu));
}

#[test]
fn test_validate_peer_script() {
    ok!(parse_and_validate_peer_script("echo ok;"));
    is_err!(
        parse_and_validate_peer_script(""),
        ValidationError::ScriptMissingSemicolon()
    );
    is_err!(
        parse_and_validate_peer_script("echo missing_semicolon"),
        ValidationError::ScriptMissingSemicolon()
    );
}

#[test]
fn test_validate_script_struct() {
    // Disabled script should always be valid (even with invalid script)
    let script = Script {
        enabled: false,
        script: "echo missing_semicolon".to_string(),
    };
    ok!(validate_peer_script(&script));

    // Enabled script without semicolon should fail
    let script = Script {
        enabled: true,
        script: "echo missing_semicolon".to_string(),
    };
    is_err!(
        validate_peer_script(&script),
        ValidationError::ScriptMissingSemicolon()
    );

    // Enabled script with semicolon should succeed
    let script = Script {
        enabled: true,
        script: "echo ok;".to_string(),
    };
    ok!(validate_peer_script(&script));

    // Enabled script with trailing whitespace and semicolon should succeed
    let script = Script {
        enabled: true,
        script: "echo ok;  ".to_string(),
    };
    ok!(validate_peer_script(&script));
}

#[test]
fn test_validate_scripts_vec() {
    // Empty scripts vector should succeed
    let scripts = vec![];
    ok!(validate_peer_scripts(&scripts));

    // All valid scripts should succeed
    let scripts = vec![
        Script {
            enabled: true,
            script: "echo first;".to_string(),
        },
        Script {
            enabled: true,
            script: "echo second;".to_string(),
        },
    ];
    ok!(validate_peer_scripts(&scripts));

    // Scripts with disabled invalid script should succeed
    let scripts = vec![
        Script {
            enabled: true,
            script: "echo valid;".to_string(),
        },
        Script {
            enabled: false,
            script: "echo invalid".to_string(),
        },
    ];
    ok!(validate_peer_scripts(&scripts));

    // Scripts with one invalid enabled script should fail with index
    let scripts = vec![
        Script {
            enabled: true,
            script: "echo first;".to_string(),
        },
        Script {
            enabled: true,
            script: "echo missing".to_string(),
        },
        Script {
            enabled: true,
            script: "echo third;".to_string(),
        },
    ];
    is_err!(
        validate_peer_scripts(&scripts),
        ValidationError::ScriptMissingSemicolonAt(1)
    );
}

#[test]
fn test_validate_wg_key() {
    ok!(parse_and_validate_wg_key("qBZArZg+2vEvD5tS8T7m0H0/xvd1PKdoBHXWIrQ1DEE="));
    is_err!(
        parse_and_validate_wg_key("def"),
        ValidationError::NotWireGuardKey()
    );
    is_err!(
        parse_and_validate_wg_key(""),
        ValidationError::NotWireGuardKey()
    );

    ok!(parse_and_validate_wg_key("ySMLaPaHVrxg/rdmZlGUemyt2JKxwSdeYUa3l34RwbE="));
    is_err!(
        parse_and_validate_wg_key("ghi"),
        ValidationError::NotWireGuardKey()
    );
    is_err!(
        parse_and_validate_wg_key(""),
        ValidationError::NotWireGuardKey()
    );
}

// Network.Connection Fields

#[test]
fn test_validate_conn_persistent_keepalive() {
    ok!(parse_and_validate_conn_persistent_keepalive_period("25"));
    is_err!(
        parse_and_validate_conn_persistent_keepalive_period(""),
        ValidationError::InvalidPersistentKeepalivePeriod()
    );
    is_err!(
        parse_and_validate_conn_persistent_keepalive_period("-1"),
        ValidationError::InvalidPersistentKeepalivePeriod()
    );
    is_err!(
        parse_and_validate_conn_persistent_keepalive_period("70000"),
        ValidationError::InvalidPersistentKeepalivePeriod()
    );
    is_err!(
        parse_and_validate_conn_persistent_keepalive_period("notnum"),
        ValidationError::InvalidPersistentKeepalivePeriod()
    );
}

#[test]
fn test_validate_persistent_keepalive_struct() {
    // Disabled persistent_keepalive should always be valid (even with 0 period)
    let pk = PersistentKeepalive {
        enabled: false,
        period: 0,
    };
    ok!(validate_conn_persistent_keepalive_period(&pk));

    // Enabled persistent_keepalive with 0 period should fail
    let pk = PersistentKeepalive {
        enabled: true,
        period: 0,
    };
    is_err!(
        validate_conn_persistent_keepalive_period(&pk),
        ValidationError::InvalidPersistentKeepalivePeriod()
    );

    // Enabled persistent_keepalive with valid period should succeed
    let pk = PersistentKeepalive {
        enabled: true,
        period: 25,
    };
    ok!(validate_conn_persistent_keepalive_period(&pk));

    // Enabled persistent_keepalive with max valid period should succeed
    let pk = PersistentKeepalive {
        enabled: true,
        period: 65535,
    };
    ok!(validate_conn_persistent_keepalive_period(&pk));
}

#[test]
fn test_validate_conn_allowed_ips() {
    ok!(parse_and_validate_conn_allowed_ips("10.0.0.0/24"));
    ok!(parse_and_validate_conn_allowed_ips("10.0.0.0/24, 192.168.1.0/24"));
    is_err!(
        parse_and_validate_conn_allowed_ips("invalid"),
        ValidationError::InvalidAllowedIPs()
    );
}
