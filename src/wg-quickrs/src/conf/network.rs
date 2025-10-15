/// Convert IPv4 string to u32
pub fn ipv4_str_to_u32(ip: String) -> Option<u32> {
    let ipv4 = ip.parse::<std::net::Ipv4Addr>().ok()?;
    Some(u32::from(ipv4))
}

/// Convert CIDR (e.g. "24") to subnet mask as u32
pub fn cidr_to_u32(cidr: u8) -> u32 {
    if cidr == 0 {
        0
    } else {
        (!0u32) << (32 - cidr)
    }
}

/// Convert u32 to IPv4 string
pub fn u32_to_ipv4_str(ip: u32) -> String {
    std::net::Ipv4Addr::from(ip).to_string()
}

/// Get the next available IPv4 address in subnet not ending with .0 or .255 and not in `taken`
pub fn get_next_available_address(subnet: &str, taken: &[String]) -> Option<String> {
    let (base_ip_str, cidr_str) = subnet.split_once('/')?;
    let cidr: u8 = cidr_str.parse().ok()?;
    let base_ip: u32 = ipv4_str_to_u32(base_ip_str.into())?;
    let subnet_mask = cidr_to_u32(cidr);

    // Calculate network start and broadcast addresses
    let network = base_ip & subnet_mask;
    let broadcast = network | (!subnet_mask);

    // Iterate through usable hosts (network+1 … broadcast-1)
    for candidate in (network + 1)..broadcast {
        let ip_str = u32_to_ipv4_str(candidate);
        if !taken.contains(&ip_str) {
            return Some(ip_str);
        }
    }

    None
}
