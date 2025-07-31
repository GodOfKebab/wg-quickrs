use std::net::Ipv4Addr;

/// Convert IPv4 string to u32
fn ipv4_to_u32(ip: &str) -> u32 {
    ip.parse::<Ipv4Addr>()
        .unwrap()
        .octets()
        .iter()
        .fold(0, |acc, &b| (acc << 8) + b as u32)
}

/// Convert CIDR (e.g. "24") to subnet mask as u32
fn cidr_to_u32(cidr: u8) -> u32 {
    if cidr == 0 { 0 } else { (!0u32) << (32 - cidr) }
}

/// Convert u32 to IPv4 string
fn u32_to_ipv4(ip: u32) -> String {
    Ipv4Addr::from(ip).to_string()
}

/// Get next available IPv4 address in subnet not ending with .0 or .255 and not in `taken`
pub(crate) fn get_next_available_address(subnet: &str, taken: &[String]) -> Option<String> {
    let (base_ip_str, cidr_str) = subnet.split_once('/')?;
    let cidr: u8 = cidr_str.parse().ok()?;
    let base_ip = ipv4_to_u32(base_ip_str);
    let subnet_mask = cidr_to_u32(cidr);
    let start_ip = base_ip & subnet_mask;
    let num_hosts = 1u32 << (32 - cidr);

    for i in 0..num_hosts {
        let candidate_ip = start_ip + i;
        let ip_str = u32_to_ipv4(candidate_ip);
        if !ip_str.ends_with(".0") && !ip_str.ends_with(".255") && !taken.contains(&ip_str) {
            return Some(ip_str);
        }
    }

    None
}
