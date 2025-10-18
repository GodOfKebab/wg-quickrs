use wg_quickrs::commands::validation::*;


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

// ------------------------
// check_field_str
// ------------------------
#[test]
fn test_check_peer_id() {
    ok!(check_field_str_agent("peer_id", "550e8400-e29b-41d4-a716-446655440000"));
    err_contains!(
        check_field_str_agent("peer_id", "not-a-uuid"),
        "peer_id needs to follow uuid4 standards"
    );
}

#[test]
fn test_check_identifier() {
    ok!(check_field_str_agent("identifier", "wg-quickrs-test"));
    err_contains!(check_field_str_agent("identifier", ""), "identifier cannot be empty");
}

#[test]
fn test_check_subnet() {
    ok!(check_field_str_agent("subnet", "0.0.0.0/0"));
    ok!(check_field_str_agent("subnet", "10.0.0.0/24"));
    ok!(check_field_str_agent("subnet", "10.0.0.1/32"));
    err_contains!(check_field_str_agent("subnet", ""), "subnet is not in CIDR format");
}

#[test]
fn test_port() {
    ok!(check_field_str_agent("port", "1500"));
    err_contains!(check_field_str_agent("port", "-1"), "port is invalid");
    err_contains!(check_field_str_agent("port", "70000"), "port is invalid");
    err_contains!(check_field_str_agent("port", "notnum"), "port is invalid");
}
