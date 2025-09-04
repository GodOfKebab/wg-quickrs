# wg-rusteze

⚠️ This repo is a **work in progress**!

An intuitive and feature-rich WireGuard configuration management tool written mainly in Rust.

## 1. Installation

You can build from scratch or Docker.

Install the repo:

```aiignore
sudo apt install git
git clone https://github.com/GodOfKebab/wg-rusteze.git
cd wg-rusteze
```

### 1.1 Build from scratch

#### Requirements

- `iptables`

Building from scratch involves three main steps.
`rust-wasm` includes the rust component for the web frontend.
`web` folder includes all the _rest_ of the files for the web frontend.
`rust-agent` embeds the web server backend, web frontend, and scripting tools in a single binary (`wg-rusteze`).

Install Rust/Cargo:

```aiignore
# Install rust/cargo
curl https://sh.rustup.rs -sSf | sh
# Source rust/cargo
. "$HOME/.cargo/env"
```

### 1.1.1 Build rust-wasm

Install wasm-pack and build `rust-wasm`:

```aiignore
# Install wasm-pack
cargo install wasm-pack
# Build rust-wasm
cd rust-wasm && wasm-pack build --target web --out-dir ../web/pkg -- --features wasm --color=always && cd ..
```

### 1.1.2 Build the web folder

Install npm and 'build' (create a `dist` folder) `web` frontend:

```aiignore
# Install wasm-pack
sudo apt install npm
# Build rust-wasm
cd web && npm install && npm run build && cd ..
```

### 1.1.3 Build wg-rusteze

Build and install `wg-rusteze`:

```aiignore
# Build wg-rusteze (might take some time on slower machines)
cargo build --profile release --package wg-rusteze --bin wg-rusteze
# Install the binary to the folder
mkdir -p ~/.wg-rusteze/bin && cp target/release/wg-rusteze ~/.wg-rusteze/bin/wg-rusteze
echo 'export PATH="$HOME/.wg-rusteze/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
# $ wg-rusteze
# A tool to manage the peer and network configuration of the WireGuard-based overlay network over the web console
# 
# Usage: wg-rusteze [OPTIONS] <COMMAND>
# 
# Commands:
#   init   Initialize the wg-rusteze rust-agent.
#          Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command
#   agent  Configure and run the wg-rusteze rust-agent
#   help   Print this message or the help of the given subcommand(s)
# 
# Options:
#   -v, --verbose
#           Increase verbosity level from Info to Debug
#       --wg-rusteze-config-folder <WG_RUSTEZE_CONFIG_FOLDER>
#           [default: .wg-rusteze]
#   -h, --help
#           Print help
#   -V, --version
#           Print version
```

### 1.1.4 Configure

Configure TLS/HTTPS certificates

```aiignore
chmod +x cert/make-tls-certs.sh
COUNTRY="XX" \
STATE="XXX" \
LOCALITY="XXX" \
ORGANIZATION="XXX" \
ORGANIZATIONAL_UNIT="XXX" \
ROOT_CN="certificate-manager@XXX" \
./cert/make-tls-certs.sh $(hostname -I | awk '{print $1}')
# If successful, you should see the certificates at
ls -1d certs/servers/$(hostname -I | awk '{print $1}')/*
```

Create the `~/.wg-rusteze` folder and move certs/keys there

```aiignore
cp certs/servers/$(hostname -I | awk '{print $1}')/* ~/.wg-rusteze
/root/.wg-rusteze
├── bin
│   └── wg-rusteze
├── cert.pem
└── key.pem

2 directories, 3 files
```

Install wireguard

```aiignore
sudo apt install -y wireguard wireguard-tools
```

Initialize and configure the agent using prompts

```aiignore
wg-rusteze init
# root@vultr:~/wg-rusteze# wg-rusteze init
# backend: v0.1.0, frontend: v0.0.0, built: 2025-09-04T02:19:14Z
# 2025-09-04T02:25:13.293Z INFO  [wg_rusteze] using the wg-rusteze config file at "/root/.wg-rusteze/conf.yml"
# 2025-09-04T02:25:13.293Z INFO  [wg_rusteze::commands::init] Initializing wg-rusteze rust-agent...
# [general network settings 1-2/25]
# 	[ 1/25] Enter VPN network's identifier (CLI option '--network-identifier') (e.g. wg-rusteze): wg-rusteze
# 	[ 2/25] Enter VPN network's CIDR subnet mask (CLI option '--network-subnet') (e.g. 10.0.34.0/24): 10.0.34.0/24
# [general network settings complete]
# [agent settings 3-18/25]
# 	[ 3/25] Enter agent's peer name (CLI option '--agent-peer-name') (e.g. wg-rusteze-host): wg-rusteze-host
# 	[ 4/25] Enter agent's local IPv4 address for the web server to bind and vpn server to listen (CLI option '--agent-local-address') (e.g. XXX.XXX.XXX.XXX): XXX.XXX.XXX.XXX
# 	[ 5/25] Enable Enable/Disable HTTP for the web server (CLI option '--agent-local-enable-web-http')? yes
# 	[ 5/25] 	Enter agent's local HTTP port for the web server to bind (CLI option '--agent-local-web-http-port') (e.g. 80): 80
# 	[ 6/25] Enable Enable/Disable HTTPS for the web server (CLI option '--agent-local-enable-web-https')? yes
# 	[ 6/25] 	Enter agent's local HTTPS port for the web server to bind (CLI option '--agent-local-web-https-port') (e.g. 443): 443
# 	[ 6/25] 	Enter TLS certificate file path for HTTPS (CLI option '--agent-local-web-https-tls-cert') (e.g. cert.pem): cert.pem
# 	[ 6/25] 	Enter TLS signing key file path for HTTPS (CLI option '--agent-local-web-https-tls-key') (e.g. key.pem): key.pem
# 	[ 7/25] Enable Enable/Disable VPN server (CLI option '--agent-local-enable-vpn')? yes
# 	[ 7/25] 	Enter agent's local VPN port for the vpn server to bind (CLI option '--agent-local-vpn-port') (e.g. 51820): 51820
# 	[ 7/25] 	Enter interface for the VPN server's packet forwarding setup (CLI option '--agent-local-vpn-interface') (e.g. enp1s0): enp1s0
# 	[ 8/25] Enter agent's publicly accessible IPv4 address to be used in the VPN endpoint advertisement (CLI option '--agent-public-address') (e.g. XXX.XXX.XXX.XXX): XXX.XXX.XXX.XXX
# 	[ 9/25] Enter agent's publicly accessible port to be used in the VPN endpoint advertisement (CLI option '--agent-public-vpn-port') (e.g. 51820): 51820
# 	[10/25] Enter agent's internal IPv4 address for VPN network (CLI option '--agent-internal-vpn-address') (e.g. 10.0.34.1): 10.0.34.1
# 	[11/25] Enable password for this agent's web server (CLI option '--agent-enable-web-password')? yes
# 	[12/25] 	Enter password for this agent's web server: [hidden]
# 	[13/25] Enable DNS server field for this agent (CLI option '--agent-enable-dns')? yes
# 	[13/25] 	Enter DNS server for this agent (CLI option '--agent-dns-server') (e.g. 1.1.1.1): 1.1.1.1
# 	[14/25] Enable MTU value field for this agent (CLI option '--agent-enable-mtu')? no
# 	[15/25] Enable PreUp scripting field for this agent (CLI option '--agent-enable-script-pre-up')? no
# 	[16/25] Enable PostUp scripting field for this agent (CLI option '--agent-enable-script-post-up')? no
# 	[17/25] Enable PreDown scripting field for this agent (CLI option '--agent-enable-script-pre-down')? no
# 	[18/25] Enable PostDown scripting field for this agent (CLI option '--agent-enable-script-post-down')? no
# [agent settings complete]
# [new peer/connection default settings 19-25/25]
# 	[19/25] Enable DNS field for new peers by default (CLI option '--default-enable-dns')? yes
# 	[19/25] 	Enter DNS server for new peers by default (CLI option '--default-dns-server') (e.g. 1.1.1.1): 1.1.1.1
# 	[20/25] Enable MTU field for new peers by default (CLI option '--default-enable-mtu')? no
# 	[21/25] Enable PreUp scripting field for new peers by default (CLI option '--default-enable-script-pre-up')? no
# 	[22/25] Enable PostUp scripting field for this default (CLI option '--default-enable-script-post-up')? no
# 	[23/25] Enable PreDown scripting field for this default (CLI option '--default-enable-script-pre-down')? no
# 	[24/25] Enable PostDown scripting field for this default (CLI option '--default-enable-script-post-down')? no
# 	[25/25] Enable PersistentKeepalive field for new connections by default (CLI option '--default-enable-persistent-keepalive')? yes
# 	[25/25] 	Enter PersistentKeepalive period (seconds) for new connections by default (CLI option '--default-persistent-keepalive-period') (e.g. 25): 25
# [new peer/connection default settings complete]
# ✅ This was all the information required to initialize the rust-agent. Finalizing the configuration...
# 2025-09-04T02:25:20.211Z INFO  [wg_rusteze::wireguard::cmd] $ wg genkey
# 2025-09-04T02:25:20.212Z INFO  [wg_rusteze::wireguard::cmd] $ wg genkey | wg pubkey
# 2025-09-04T02:25:20.212Z INFO  [wg_rusteze::conf::util] updated config file
# ✅ Configuration saved to `config.yml`.
```

Setup the firewall to accept HTTP and HTTPS ports

```aiignore
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
```

Finally, run the agent

```aiignore
wg-rusteze agent run
# $ wg-rusteze agent run
# backend: v0.1.0, frontend: v0.0.0, built: 2025-09-04T03:23:38Z
# 2025-09-04T03:27:33.100Z INFO  [wg_rusteze] using the wg-rusteze config file at "/root/.wg-rusteze/conf.yml"
# 2025-09-04T03:27:33.100Z INFO  [wg_rusteze::commands::agent] using the wireguard config file at "/etc/wireguard/wg-rusteze.conf"
# 2025-09-04T03:27:33.364Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg-quick down wg-rusteze
# 2025-09-04T03:27:33.364Z WARN  [wg_rusteze::wireguard::cmd] [#] ip link delete dev wg-rusteze
# [#] resolvconf -d tun.wg-rusteze -f
# [#] iptables -t nat -D POSTROUTING -s 10.0.34.0/24 -o enp1s0 -j MASQUERADE;
# [#] iptables -D INPUT -p udp -m udp --dport 51820 -j ACCEPT;
# [#] iptables -D FORWARD -i wg-rusteze -j ACCEPT;
# [#] iptables -D FORWARD -o wg-rusteze -j ACCEPT;
# 
# 2025-09-04T03:27:33.512Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg-quick up wg-rusteze
# 2025-09-04T03:27:33.512Z WARN  [wg_rusteze::wireguard::cmd] [#] ip link add wg-rusteze type wireguard
# [#] wg setconf wg-rusteze /dev/fd/63
# [#] ip -4 address add 10.0.34.1/24 dev wg-rusteze
# [#] ip link set mtu 1420 up dev wg-rusteze
# [#] resolvconf -a tun.wg-rusteze -m 0 -x
# [#] iptables -t nat -A POSTROUTING -s 10.0.34.0/24 -o enp1s0 -j MASQUERADE;
# [#] iptables -A INPUT -p udp -m udp --dport 51820 -j ACCEPT;
# [#] iptables -A FORWARD -i wg-rusteze -j ACCEPT;
# [#] iptables -A FORWARD -o wg-rusteze -j ACCEPT;
# 
# 2025-09-04T03:27:33.512Z INFO  [wg_rusteze::wireguard::cmd] wireguard tunnel accessible at XXX.XXX.XXX.XXX:51820
# 2025-09-04T03:27:33.513Z INFO  [wg_rusteze::web::server] Started HTTP frontend/API at http://XXX.XXX.XXX.XXX:80/
# 2025-09-04T03:27:33.513Z INFO  [actix_server::builder] starting 1 workers
# 2025-09-04T03:27:33.514Z INFO  [wg_rusteze::web::server] Started HTTPS frontend/API at https://XXX.XXX.XXX.XXX:443/
# 2025-09-04T03:27:33.515Z INFO  [actix_server::builder] starting 1 workers
# 2025-09-04T03:27:33.515Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
# 2025-09-04T03:27:33.515Z INFO  [actix_server::server] starting service: "actix-web-service-XXX.XXX.XXX.XXX:80", workers: 1, listening on: XXX.XXX.XXX.XXX:80
# 2025-09-04T03:27:33.517Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
# 2025-09-04T03:27:33.517Z INFO  [actix_server::server] starting service: "actix-web-service-XXX.XXX.XXX.XXX:443", workers: 1, listening on: XXX.XXX.XXX.XXX:443
# 2025-09-04T03:27:39.892Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg show wg-rusteze dump
# 2025-09-04T03:27:40.900Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg show wg-rusteze dump
```

### 1.2 Run using Docker

Install Docker:

```aiignore
# Remove old versions
for pkg in docker.io docker-doc docker-compose podman-docker containerd runc; do sudo apt-get remove $pkg; done
# Install Docker using the apt repository
# Add Docker's official GPG key:
sudo apt-get update
sudo apt-get install ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc

# Add the repository to Apt sources:
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```
