# wg-rusteze

⚠️ This repo is a **work in progress**!

An intuitive and feature-rich WireGuard configuration management tool written mainly in Rust.

---

## 1. Installation

You can either build from scratch or use Docker.

Clone the repository:

```bash
sudo apt install git
git clone https://github.com/GodOfKebab/wg-rusteze.git
cd wg-rusteze
```

---

### 1.1 Build from Scratch

#### Requirements

* `iptables`
* Rust and Cargo
* Node.js/npm (for the web frontend)

The project is split into:

* **`rust-wasm`** – Rust code for the web frontend
* **`web`** – frontend assets
* **`rust-agent`** – backend, frontend server, and scripting tools bundled in `wg-rusteze` binary

---

#### 1.1.1 Install Rust/Cargo

```bash
curl https://sh.rustup.rs -sSf | sh
. "$HOME/.cargo/env"
```

---

#### 1.1.2 Build `rust-wasm`

```bash
cargo install wasm-pack
cd rust-wasm
wasm-pack build --target web --out-dir ../web/pkg -- --features wasm --color=always
cd ..
```

---

#### 1.1.3 Build the web frontend

```bash
sudo apt install npm
cd web
npm install
npm run build
cd ..
```

---

#### 1.1.4 Build and Install `wg-rusteze`

This might take some time on slower machines.

```bash
cargo build --profile release --package wg-rusteze --bin wg-rusteze

mkdir -p ~/.wg-rusteze/bin
cp target/release/wg-rusteze ~/.wg-rusteze/bin/wg-rusteze
cp -r target/release/completions ~/.wg-rusteze/completions

# Bash
echo 'export PATH="$HOME/.wg-rusteze/bin:$PATH"' >> ~/.bashrc
echo 'source $HOME/.wg-rusteze/completions/wg-rusteze.bash' >> ~/.bashrc
source ~/.bashrc

# ZSH
echo 'export PATH="$HOME/.wg-rusteze/bin:$PATH"' >> ~/.zshrc
echo 'source $HOME/.wg-rusteze/completions/_wg-rusteze' >> ~/.zshrc
source ~/.zshrc

wg-rusteze --help
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

wg-rusteze <TAB>           # Shows available commands (init, agent)
wg-rusteze agent <TAB>     # Shows available agent subcommands
wg-rusteze init --<TAB>    # Shows available options for the init command
```

---

#### 1.1.5 Configure TLS/HTTPS Certificates

```bash
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

Move the certificates to `~/.wg-rusteze`:

```bash
cp certs/servers/$(hostname -I | awk '{print $1}')/* ~/.wg-rusteze/
```

---

#### 1.1.6 Install WireGuard

```bash
sudo apt install -y wireguard wireguard-tools
```

---

#### 1.1.7 Initialize and Configure the Agent

```bash
wg-rusteze init
# $ wg-rusteze init
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

Follow the prompts to configure network, agent, and default peer settings. This generates `~/.wg-rusteze/conf.yml`.

Folder structure after initialization:

```bash
tree ~/.wg-rusteze
# ~/.wg-rusteze
# ├── bin
# │   └── wg-rusteze
# ├── cert.pem
# ├── completions
# │   ├── _wg-rusteze
# │   └── wg-rusteze.bash
# ├── conf.yml
# └── key.pem
# 
# 3 directories, 6 files
```

---

#### 1.1.8 Set Firewall Rules

```bash
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
```

---

#### 1.1.9 Run the Agent

```bash
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

* HTTP frontend/API: `http://<your-ip>:80/`
* HTTPS frontend/API: `https://<your-ip>:443/`
* WireGuard tunnel: `<public-ip>:51820`

---

### 1.2 Using Docker

Install Docker on Debian 12:

```bash
for pkg in docker.io docker-doc docker-compose podman-docker containerd runc; do sudo apt-get remove $pkg; done
sudo apt-get update
sudo apt-get install ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc

echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

Edit the TLS certificate settings and enter the FQDN/Domain names for the certificates from `certificate-manager`
service in `docker-compose.init.yml`.

```bash
docker compose -f docker-compose.init.yml up certificate-manager
tree .wg-rusteze-docker
# .wg-rusteze-docker
# └── certs
#     ├── root
#     │   ├── rootCA.crt
#     │   └── rootCA.key
#     └── servers
#         ├── 127.0.0.1
#         │   ├── cert.pem
#         │   └── key.pem
#         └── localhost
#             ├── cert.pem
#             └── key.pem
```

Edit the wg-rusteze settings from `wg-rusteze-init` service in `docker-compose.init.yml`.
Especially make sure that the IP addresses are updated and correct TLS cert/key paths are entered.

```bash
docker compose -f docker-compose.init.yml up wg-rusteze-init
tree .wg-rusteze-docker
# .wg-rusteze-docker
# └── certs
#     ├── root
#     │   ├── rootCA.crt
#     │   └── rootCA.key
#     └── servers
#         ├── 127.0.0.1
#         │   ├── cert.pem
#         │   └── key.pem
#         └── localhost
#             ├── cert.pem
#             └── key.pem
```

After initialization, you can run the `wg-rusteze-agent-run` service in `docker-compose.agent.yml`.

```bash
docker compose -f docker-compose.agent.yml up wg-rusteze-agent-run
```

