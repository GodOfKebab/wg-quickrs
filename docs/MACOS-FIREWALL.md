## PF Firewall (macOS/BSD) - Rule Ordering Requirement

### The Issue
Unlike other firewalls, PF (Packet Filter) requires rules to be declared in a specific order in `/etc/pf.conf`. NAT rules **must** appear before filter rules, or PF will fail to load the configuration.

### What `wg-quickrs` Does
When adding NAT rules for WireGuard, the tool:
1. Searches for existing `nat` rules in `pf.conf`
2. Inserts the new NAT rule immediately after the last `nat` rule
3. This ensures proper rule ordering is maintained

### Example pf.conf
```
#
# Default PF configuration file.
#
# ...
# 
scrub-anchor "com.apple/*"
nat-anchor "com.apple/*"  # <-- (1) This is the existing rule the tool searches for
nat on en0 from 10.0.34.0/24 to any -> en0  # <-- (2) This is the new rule the tool inserts on tunnel startup and removes on exit
rdr-anchor "com.apple/*"
dummynet-anchor "com.apple/*"
anchor "com.apple/*"
load anchor "com.apple" from "/etc/pf.anchors/com.apple"
```

This rule translates traffic from your WireGuard subnet (`10.0.34.0/24`) through your gateway interface (`en0`).

### Manual Configuration
If the tool fails and you need to add the rule manually:

1. Edit `/etc/pf.conf` (requires sudo)
2. Find the section with other `nat` rules (or create one after any `include` statements)
3. Add your NAT rule:
```
   nat on <gateway_interface> from <wireguard_subnet> to any -> <gateway_interface>
```
4. Load the new configuration:
```bash
   sudo pfctl -f /etc/pf.conf
```

### References
- [PF User's Guide - Rule Ordering](https://www.openbsd.org/faq/pf/filter.html)
- [PF NAT Configuration](https://www.openbsd.org/faq/pf/nat.html)