# wg-rusteze

⚠️ This repo is a **work in progress**!

An intuitive and feature-rich WireGuard configuration management tool written mainly in Rust.

---

## 1. Installation

You can either build from scratch or use Docker.

Clone the repository:

```bash
sudo apt install -y git
git clone --recursive https://github.com/GodOfKebab/wg-rusteze.git
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
sudo apt install -y npm
cd web
npm install
npm run build
cd ..
```

---

#### 1.1.4 Build and Install `wg-rusteze`

This might take some time on slower machines.

```bash
cargo build --release --package wg-rusteze --bin wg-rusteze

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

# Fish
echo 'export PATH="$HOME/.wg-rusteze/bin:$PATH"' >> ~/.config/fish/config.fish
echo 'source $HOME/.wg-rusteze/completions/wg-rusteze.fish' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish

# Elvish
echo 'export PATH="$HOME/.wg-rusteze/bin:$PATH"' >> ~/.elvish/rc.elv
echo 'source $HOME/.wg-rusteze/completions/wg-rusteze.elv' >> ~/.elvish/rc.elv
source ~/.elvish/rc.elv

# PowerShell Core / Windows PowerShell
profile_path=$(pwsh -NoProfile -Command '$PROFILE')
echo '$env:PATH = "$HOME/.wg-rusteze/bin;" + $env:PATH' >> "$profile_path"
echo '. $HOME/.wg-rusteze/completions/_wg-rusteze.ps1' >> "$profile_path"
pwsh -NoProfile -Command ". \$PROFILE"

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
export COUNTRY="TR"
export STATE="Istanbul"
export LOCALITY="Fatih"
export ORGANIZATION="God Of Kebab Labs"
export ORGANIZATIONAL_UNIT="God Of Kebab's Guide to the WWW"
export ROOT_CN="certificate-manager@kebabnet"
sh certificate-manager/make-tls-certs.sh all

# If successful, you should see the certificates under
ls -1 certs/servers/

# Move the certs directory to `~/.wg-rusteze`
mkdir -p ~/.wg-rusteze && cp -r certs ~/.wg-rusteze/certs
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
# backend: v0.1.0, frontend: v0.0.0, built: 2025-09-08T00:33:15Z
# 2025-09-08T00:52:14.344Z INFO  [wg_rusteze] using the wg-rusteze config file at "/root/.wg-rusteze/conf.yml"
# 2025-09-08T00:52:14.345Z INFO  [wg_rusteze::commands::init] Initializing wg-rusteze rust-agent...
# [general network settings 1-2/24]
# 	[ 1/24] Set VPN network identifier (CLI option '--network-identifier'): wg-rusteze
# 	[ 2/24] Set VPN network CIDR subnet (CLI option '--network-subnet'): 10.0.34.0/24
# [general network settings complete]
# [agent settings 3-17/24]
# 	[ 3/24] Set agent web server bind IPv4 address (CLI option '--agent-web-address'): XX.XX.XX.XX
# 	[ 4/24] Enable HTTP on web server (CLI option '--agent-web-http-enabled')? yes
# 	[ 4/24] 	Set web server HTTP port (CLI option '--agent-web-http-port'): 80
# 	[ 5/24] Enable HTTPS on web server (CLI option '--agent-web-https-enabled')? yes
# 	[ 5/24] 	Set web server HTTPS port (CLI option '--agent-web-https-port'): 443
# 	[ 5/24] 	Set path (relative to the wg-rusteze home directory) to TLS certificate file for HTTPS (CLI option '--agent-web-https-tls-cert'): certs/servers/XX.XX.XX.XX/cert.pem
# 	[ 5/24] 	Set path (relative to the wg-rusteze home directory) to TLS private key file for HTTPS (CLI option '--agent-web-https-tls-key'): certs/servers/XX.XX.XX.XX/key.pem
# 	[ 6/24] Enable password authentication for web server (CLI option '--agent-web-password-enabled')? yes
# 	[ 6/24] 	Set password for web server access: [hidden]
# 	[ 7/24] Enable VPN server (CLI option '--agent-vpn-enabled')? yes
# 	[ 7/24] 	Set VPN server listening port (CLI option '--agent-vpn-port'): 51820
# 	[ 7/24] 	Set gateway (outbound interface) for VPN packet forwarding (CLI option '--agent-vpn-gateway'): enp1s0
# 	[ 8/24] Enable running firewall commands for setting up NAT and input rules (CLI option '--agent-firewall-enabled')? yes
# 	[ 8/24] 	Set the utility used to configure firewall NAT and input rules (CLI option '--agent-firewall-utility'): /usr/sbin/iptables
# 	[ 9/24] Set agent peer name (CLI option '--agent-peer-name'): wg-rusteze-host
# 	[10/24] Set publicly accessible endpoint(IP/FQDN:PORT) for VPN endpoint (CLI option '--agent-peer-vpn-endpoint'): XX.XX.XX.XX:51820
# 	[11/24] Set internal IPv4 address for agent in VPN network (CLI option '--agent-peer-vpn-internal-address'): 10.0.34.1
# 	[12/24] Enable DNS configuration for agent (CLI option '--agent-peer-dns-enabled')? yes
# 	[12/24] 	Set DNS server for agent (CLI option '--agent-peer-dns-server'): 1.1.1.1
# 	[13/24] Enable MTU configuration for agent (CLI option '--agent-peer-mtu-enabled')? no
# 	[14/24] Enable PreUp script for agent (CLI option '--agent-peer-script-pre-up-enabled')? no
# 	[15/24] Enable PostUp script for agent (CLI option '--agent-peer-script-post-up-enabled')? no
# 	[16/24] Enable PreDown script for agent (CLI option '--agent-peer-script-pre-down-enabled')? no
# 	[17/24] Enable PostDown script for agent (CLI option '--agent-peer-script-post-down-enabled')? no
# [agent settings complete]
# [new peer/connection default settings 18-24/24]
# 	[18/24] Enable DNS for new peers by default (CLI option '--default-peer-dns-enabled')? yes
# 	[18/24] 	Set default DNS server for new peers (CLI option '--default-peer-dns-server'): 1.1.1.1
# 	[19/24] Enable MTU for new peers by default (CLI option '--default-peer-mtu-enabled')? no
# 	[20/24] Enable PreUp script for new peers by default (CLI option '--default-peer-script-pre-up-enabled')? no
# 	[21/24] Enable PostUp script for new peers by default (CLI option '--default-peer-script-post-up-enabled')? no
# 	[22/24] Enable PreDown script for new peers by default (CLI option '--default-peer-script-pre-down-enabled')? no
# 	[23/24] Enable PostDown script for new peers by default (CLI option '--default-peer-script-post-down-enabled')? no
# 	[24/24] Enable PersistentKeepalive for new connections by default (CLI option '--default-connection-persistent-keepalive-enabled')? yes
# 	[24/24] 	Set default PersistentKeepalive period in seconds (CLI option '--default-connection-persistent-keepalive-period'): 25
# [new peer/connection default settings complete]
# ✅ This was all the information required to initialize the rust-agent. Finalizing the configuration...
# 2025-09-08T00:53:26.909Z INFO  [wg_rusteze::wireguard::cmd] $ wg genkey
# 2025-09-08T00:53:26.911Z INFO  [wg_rusteze::wireguard::cmd] $ wg genkey | wg pubkey
# 2025-09-08T00:53:26.912Z INFO  [wg_rusteze::conf::util] updated config file
# ✅ Configuration saved to `config.yml`.
```

Follow the prompts to configure network, agent, and default peer settings. This generates `~/.wg-rusteze/conf.yml`.

Folder structure after initialization:

```bash
# sudo apt install -y tree
tree ~/.wg-rusteze
# ├── bin
# │   └── wg-rusteze
# ├── certs
# │   ├── root
# │   │   ├── rootCA.crt
# │   │   └── rootCA.key
# │   └── servers
# │       ├── ...
# │       │   ├── cert.pem
# │       │   └── key.pem
# │       ├── XX.XX.XX.XX
# │       │   ├── cert.pem
# │       │   └── key.pem
# │       ├── ...
# │       │   ├── cert.pem
# │       │   └── key.pem
# ├── completions
# │   ├── _wg-rusteze
# │   └── wg-rusteze.bash
# └── conf.yml
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
# backend: v0.1.0, frontend: v0.0.0, built: 2025-09-08T00:33:15Z
# 2025-09-08T00:57:32.398Z INFO  [wg_rusteze] using the wg-rusteze config file at "/root/.wg-rusteze/conf.yml"
# 2025-09-08T00:57:32.399Z INFO  [wg_rusteze::commands::agent] using the wireguard config file at "/etc/wireguard/wg-rusteze.conf"
# 2025-09-08T00:57:32.449Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg-quick down wg-rusteze
# 2025-09-08T00:57:32.449Z WARN  [wg_rusteze::wireguard::cmd] wg-quick: `wg-rusteze' is not a WireGuard interface
# 
# 2025-09-08T00:57:32.744Z INFO  [wg_rusteze::wireguard::cmd] $ sudo wg-quick up wg-rusteze
# 2025-09-08T00:57:32.745Z WARN  [wg_rusteze::wireguard::cmd] [#] ip link add wg-rusteze type wireguard
# [#] wg setconf wg-rusteze /dev/fd/63
# [#] ip -4 address add 10.0.34.1/24 dev wg-rusteze
# [#] ip link set mtu 1420 up dev wg-rusteze
# [#] resolvconf -a tun.wg-rusteze -m 0 -x
# [#] iptables -t nat -A POSTROUTING -s 10.0.34.0/24 -o enp1s0 -j MASQUERADE;
# [#] iptables -A INPUT -p udp -m udp --dport 51820 -j ACCEPT;
# [#] iptables -A FORWARD -i wg-rusteze -j ACCEPT;
# [#] iptables -A FORWARD -o wg-rusteze -j ACCEPT;
# 
# 2025-09-08T00:57:32.745Z INFO  [wg_rusteze::wireguard::cmd] wireguard tunnel accessible at XX.XX.XX.XX:51820
# 2025-09-08T00:57:32.745Z INFO  [wg_rusteze::web::server] Started HTTP frontend/API at http://XX.XX.XX.XX:80/
# 2025-09-08T00:57:32.745Z INFO  [actix_server::builder] starting 2 workers
# 2025-09-08T00:57:32.746Z INFO  [wg_rusteze::web::server] Started HTTPS frontend/API at https://XX.XX.XX.XX:443/
# 2025-09-08T00:57:32.746Z INFO  [actix_server::builder] starting 2 workers
# 2025-09-08T00:57:32.746Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
# 2025-09-08T00:57:32.746Z INFO  [actix_server::server] starting service: "actix-web-service-XX.XX.XX.XX:80", workers: 2, listening on: XX.XX.XX.XX:80
# 2025-09-08T00:57:32.748Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
# 2025-09-08T00:57:32.748Z INFO  [actix_server::server] starting service: "actix-web-service-XX.XX.XX.XX:443", workers: 2, listening on: XX.XX.XX.XX:443
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

