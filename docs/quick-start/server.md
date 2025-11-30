# wg-quickrs quick-start guide for servers


## 1. Use the installer script with pre-built binaries (recommended)

The `installer.sh` script is the easiest way to install wg-quickrs on your server.
- It automatically detects your OS and architecture,
- installs dependencies (`wg`, os-specific commands etc.),
- downloads and installs the appropriate binary with shell completions (`bash` or `zsh`),
- (optionally) generates TLS certificates,
- (optionally) configures `systemd`/`OpenRC` services.

```bash
wget -q https://github.com/GodOfKebab/wg-quickrs/releases/latest/download/installer.sh
sh installer.sh
# OR specify a release like so
# wget -q https://github.com/GodOfKebab/wg-quickrs/releases/download/v2.0.0/installer.sh
# sh installer.sh
# Note: you still might be able to download earlier releases with the latest installer script
#       but it's not guaranteed to work. If you want to be sure, use the installer script from that old release.
```

### Installation options

```bash
# View all available options
sh installer.sh --help

# List available releases
sh installer.sh list-releases

# Install a specific release version
sh installer.sh --release v2.0.0

# Install to user directory instead of system-wide
sh installer.sh --install-to user

# Skip automatic dependency installation
sh installer.sh --skip-deps

# Use a local tarball instead of downloading (Air-gapped installation)
wget -q https://github.com/GodOfKebab/wg-quickrs/releases/latest/download/wg-quickrs-x86_64-unknown-linux-musl.tar.gz
sh installer.sh --dist-tarball ./wg-quickrs-x86_64-unknown-linux-musl.tar.gz
```

### Installation locations

| Install Type     | Binary Location             | Config Location   |
|------------------|-----------------------------|-------------------|
| System (default) | `/usr/local/bin/wg-quickrs` | `/etc/wg-quickrs` |
| User             | `~/.local/bin/wg-quickrs`   | `~/.wg-quickrs`   |

### After installation

Once the installer completes, you'll be ready to initialize your agent and test that everything works:

```bash
# System installation
sudo wg-quickrs agent init
sudo wg-quickrs agent run

# User installation
wg-quickrs --wg-quickrs-config-folder ~/.wg-quickrs agent init
wg-quickrs --wg-quickrs-config-folder ~/.wg-quickrs agent run
```

If you set up `systemd`/`OpenRC` service, you can manage it with:

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

