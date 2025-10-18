use wg_quickrs_wasm::validation::*;
use wg_quickrs_wasm::types::EnabledValue;


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

#[test]
fn test_check_address() {
    ok!(check_field_str("address", "10.0.0.1"));
    err_contains!(check_field_str("address", "not-ip"), "address is not IPv4");
}

#[test]
fn test_check_public_private_pre_shared_keys() {
    ok!(check_field_str("public_key", "Cex1GKPMNCdt1sE+FSy09rxzubKJsqnu/i2odlSsHSc="));
    err_contains!(
        check_field_str("public_key", "abc"),
        "public_key is not base64 encoded with 32 bytes"
    );
    err_contains!(
        check_field_str("public_key", ""),
        "public_key is not base64 encoded with 32 bytes"
    );
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
