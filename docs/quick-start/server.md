# wg-quickrs quick-start guide for servers

## Requirements

`installer.sh` script will install the following dependencies if not already installed:

- `wireguard-tools` (only the `wg(8)` utility)
- Linux (same dependencies as `wg-easy`)
    - `openresolv` / `resolvconf` one or the other required for DNS resolution
    - `iproute2` required for setting up interfaces
    - `iptables` / `nftables` (optional one or the other for setting up firewall)
- macOS
    - None (brew install `wireguard-tools` sets up all the required dependencies)
- Windows
    - Not supported

---

## 1. Use the pre-built binaries (recommended)

Use the installer script to auto-detect OS/architecture combo to determine which binary is needed.

TODO: Add more details about the installer script (options, what it does, etc.)

```bash
wget -qO installer.sh https://raw.githubusercontent.com/GodOfKebab/wg-quickrs/refs/heads/main/installer.sh
sh installer.sh
````

---

## 2. Build from source

See [BUILDING.md](../BUILDING.md)

