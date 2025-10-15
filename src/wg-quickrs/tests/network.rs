use wg_quickrs::conf::network::*;


#[test]
fn test_ipv4_to_u32_basic() {
    assert_eq!(ipv4_str_to_u32("0.0.0.0".into()), Some(0));
    assert_eq!(ipv4_str_to_u32("0.0.0.1".into()), Some(1));
    assert_eq!(ipv4_str_to_u32("255.255.255.255".into()), Some(0xFF_FF_FF_FF));
    assert_eq!(ipv4_str_to_u32("192.168.1.1".into()), Some(0xC0_A8_01_01));
    assert_eq!(ipv4_str_to_u32("not-ip".into()), None);
}

#[test]
fn test_cidr_to_u32_basic() {
    assert_eq!(cidr_to_u32(0), 0);
    assert_eq!(cidr_to_u32(8), 0xFF_00_00_00);
    assert_eq!(cidr_to_u32(16), 0xFF_FF_00_00);
    assert_eq!(cidr_to_u32(24), 0xFF_FF_FF_00);
    assert_eq!(cidr_to_u32(32), 0xFF_FF_FF_FF);
}

#[test]
fn test_u32_to_ipv4_roundtrip() {
    assert_eq!(u32_to_ipv4_str(ipv4_str_to_u32("10.1.2.3".into()).unwrap()), "10.1.2.3");
}

#[test]
fn test_get_next_available_address_basic() {
    // /30 = 4 hosts total: .0 (net), .1, .2, .3 (broadcast)
    let subnet = "192.168.10.0/30";
    let taken = vec![];
    let addr = get_next_available_address(subnet, &taken).unwrap();
    assert_eq!(addr, "192.168.10.1");

    // If .1 is taken, next available should be .2
    let taken = vec!["192.168.10.1".to_string()];
    let addr = get_next_available_address(subnet, &taken).unwrap();
    assert_eq!(addr, "192.168.10.2");

    // If both .1 and .2 are taken, none left
    let taken = vec!["192.168.10.1".to_string(), "192.168.10.2".to_string()];
    assert_eq!(get_next_available_address(subnet, &taken), None);
}

#[test]
fn test_get_next_available_address_ignores_dot0_and_dot255() {
    // /24 = .0 network, .255 broadcast
    let subnet = "10.0.0.0/24";
    let mut taken = vec![];
    for _ in 0..254 {
        let taken_ip = get_next_available_address(subnet, &taken).unwrap();
        taken.push(taken_ip);
    }
    assert!(get_next_available_address(subnet, &taken).is_none());
}

#[test]
fn test_get_next_available_address_invalid_input() {
    // Missing CIDR
    assert_eq!(get_next_available_address("10.0.0.0", &[]), None);
    // Invalid CIDR value
    assert_eq!(get_next_available_address("10.0.0.0/abc", &[]), None);
    // Invalid base IP
    assert_eq!(get_next_available_address("not_an_ip/24", &[]), None);
}

#[test]
fn test_get_next_available_address_full_subnet() {
    // All usable IPs are taken
    let subnet = "10.10.10.0/30";
    let taken = vec!["10.10.10.1".into(), "10.10.10.2".into()];
    assert_eq!(get_next_available_address(subnet, &taken), None);
}

