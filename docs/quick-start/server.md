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

The `installer.sh` script is the easiest way to install wg-quickrs on your server. It automatically detects your OS and architecture, downloads the appropriate binary, installs dependencies, and optionally configures systemd/OpenRC services.

### What the installer does

The installer script performs the following actions:

1. **Dependency Management**: Checks for and optionally installs required dependencies:
   - WireGuard tools (`wg`)
   - Linux: `iproute2`, `openresolv`/`resolvconf`, `iptables`
   - macOS: `wireguard-tools` (via Homebrew)

2. **Binary Installation**:
   - Downloads the correct release tarball for your OS/architecture from GitHub
   - Extracts and installs the binary to `/usr/local/bin` (system) or `~/.local/bin` (user)
   - Installs configuration files to `/etc/wg-quickrs` (system) or `~/.wg-quickrs` (user)

3. **Shell Completions**: Sets up auto-completion for `bash` or `zsh`

4. **TLS Certificate Setup**: Optionally generates TLS certificates/keys for HTTPS support

5. **Service Setup**: Optionally configures `systemd` (Linux) or `openrc` (Alpine) services with proper user/group permissions. So the agent can be managed easier and survive system reboots.

### Basic installation

```bash
wget -qO installer.sh https://raw.githubusercontent.com/GodOfKebab/wg-quickrs/refs/heads/main/installer.sh
sh installer.sh
```

### Installation options

```bash
# View all available options
sh installer.sh --help

# List available releases
sh installer.sh list-releases

# Install a specific release version
sh installer.sh --release v1.0.0

# Install to user directory instead of system-wide
sh installer.sh --install-to user

# Skip automatic dependency installation
sh installer.sh --skip-deps

# Use a local tarball instead of downloading (Air-gapped installation)
wget https://github.com/GodOfKebab/wg-quickrs/releases/download/v1.0.0/wg-quickrs-x86_64-unknown-linux-musl.tar.gz
sh installer.sh --dist-tarball ./wg-quickrs-x86_64-unknown-linux-musl.tar.gz
```

### Installation locations

| Install Type     | Binary Location             | Config Location   |
|------------------|-----------------------------|-------------------|
| System (default) | `/usr/local/bin/wg-quickrs` | `/etc/wg-quickrs` |
| User             | `~/.local/bin/wg-quickrs`   | `~/.wg-quickrs`   |

### After installation

Once the installer completes, you'll be ready to initialize your agent:

```bash
# System installation
sudo wg-quickrs agent init
sudo wg-quickrs agent run

# User installation
wg-quickrs --wg-quickrs-config-folder ~/.wg-quickrs agent init
wg-quickrs --wg-quickrs-config-folder ~/.wg-quickrs agent run
```

If you set up systemd/OpenRC service, you can manage it with:

```bash
# Systemd
sudo systemctl enable wg-quickrs
sudo systemctl start wg-quickrs
sudo systemctl status wg-quickrs

# OpenRC (Alpine Linux)
sudo rc-update add wg-quickrs default
sudo rc-service wg-quickrs start
sudo rc-service wg-quickrs status
```

---

## 2. Build from source

See [BUILDING.md](../BUILDING.md)

