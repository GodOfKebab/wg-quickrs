use std::net::Ipv4Addr;
use ipnet::Ipv4Net;

/// Get the next available IPv4 address in subnet not ending with .0 or .255 and not in `taken`
pub fn get_next_available_address(subnet: &Ipv4Net, taken: &[Ipv4Addr]) -> Option<Ipv4Addr> {
    // Iterate through all hosts in the subnet
    for ip in subnet.hosts() {
        // Skip if already taken
        if !taken.contains(&ip) {
            return Some(ip);
        }
    }

    None
}
