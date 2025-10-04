# wg-quickrs

[![License](https://img.shields.io/github/license/godofkebab/wg-quickrs?logo=GitHub&color=brightgreen)](https://github.com/GodOfKebab/wg-quickrs)
![Static Badge](https://img.shields.io/badge/amd64%20%7C%20arm64%20%7C%20arm%2Fv7%20%20-%20grey?label=arch)
![Static Badge](https://img.shields.io/badge/Debian%20%7C%20MacOS%20%20-%20black?label=platform)

[![Release](https://img.shields.io/github/v/tag/godofkebab/wg-quickrs?logo=github&label=latest%20tag&color=blue)](https://github.com/godofkebab/wg-quickrs/releases/latest)
[![Docker](https://img.shields.io/docker/image-size/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs)
[![Docker](https://img.shields.io/docker/pulls/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs/tags)
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs%2FCargo.toml&query=package.rust-version&logo=rust&label=rust&color=%23000000)
![Dynamic JSON Badge](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fweb%2Fpackage.json&query=dependencies.vue&logo=vue.js&label=vue&color=%234FC08D)

⚠️ This repo is a **work in progress**!

✨ An intuitive multi-peer `wg` / `wg-quick` wrapper written in 🦀 Rust.

⚡ Rust + Vue + WASM + WireGuard = 🧪 one static binary + 📝 one YAML file to rule them all 🪄 (~15MB uncompressed)

Run it on your server, router, or computer and manage your WireGuard VPN from a web interface.

<p align="center">
<img src="https://yasar.idikut.cc/project-assets/wg-quickrs-homepage.gif" height="700" alt="demo">
</p>

Features:
- Interactive graph to configure your P2P network
- Sliding traffic graph to visualize your host's network traffic
- Secure API access via HTTPS with support for password login and JWT authentication
- Automatic firewall/NAT setup (`iptables` for Debian/Linux or `pf` for macOS, both come preinstalled with the OS)
- If you are not feeling like dealing with VPN/networking on your machine, you can also just use the web console to create `.conf` files/QR codes for your clients.

---

## Quick Start Guide

### Requirements

- `wg` and `wg-quick`
- optional for setting up the firewall: `iptables`(Linux) or `pf`(macOS)

```bash
# Install on Debian/Ubuntu
sudo apt install -y wireguard wireguard-tools
```

To get started, you can either use the pre-built binaries (recommended) or use the pre-built Docker image.

### 1. Use the pre-built binaries (recommended)

Use the installer script to auto-detect OS/architecture combo to determine which binary is needed.
This script also allows you to configure the TLS certificates/keys.

```bash
wget -qO installer.sh https://raw.githubusercontent.com/GodOfKebab/wg-quickrs/refs/heads/main/installer.sh
sh installer.sh
````

<details>
<summary>Example Output</summary>

```text
Detected target: aarch64-apple-darwin
Fetching latest release version...
    Using latest release: v0.1.11
Setting up and downloading the install directory at /Users/XXX/.wg-quickrs...
Setting up TLS certs/keys at /Users/XXX/.wg-quickrs/certs...
Enter COUNTRY [XX]: 
Enter STATE [XX]: 
Enter LOCALITY [XX]: 
Enter ORGANIZATION [XX]: 
Enter ORGANIZATIONAL_UNIT [XX]: 
Enter ROOT_CN [tls-cert-generator@XX]: 
Generating key for rootCA ...
    certs/root/rootCA.key
    Done.
Generating cert for rootCA ...
    certs/root/rootCA.crt
    Done.
Generating cert/key for XXX ...
    Generated key at certs/servers/XXX/key.pem
    Generated cert at certs/servers/XXX/cert.pem
    ...
    ...
    ✅ Generated TLS certs/keys
Setting up PATH and completions...
    ✅ Added PATH and completions to /Users/XXX/.zshrc

Open a new shell or run the following to use wg-quickrs command on this shell:

    export PATH="/Users/XXX/.wg-quickrs/bin:$PATH"
    source "/Users/XXX/.wg-quickrs/completions/_wg-quickrs"

Then, you are ready to initialize your service with:

    wg-quickrs init

After a successful initialization, you can start up your service with:

    wg-quickrs agent run
```

</details>

If you are not comfortable using the installer script, you can also manually set up your system.
Note: remember to update the `VERSION` in the url.

```bash
wget -qO wg-quickrs.tar.gz "https://github.com/GodOfKebab/wg-quickrs/releases/download/VERSION/wg-quickrs-aarch64-apple-darwin.tar.gz"
mkdir -p ~/.wg-quickrs && tar -xzf wg-quickrs.tar.gz -C ~/.wg-quickrs
```

Then add your cert to `~/.wg-quickrs/certs/YOUR_SERVER/cert.pem` and key to `~/.wg-quickrs/certs/YOUR_SERVER/key.pem`.

---

### 2. Use the pre-built Docker image

(Optionally) generate your TLS certs/keys to `$HOME/.wg-quickrs/certs/YOUR_SERVER/cert.pem`/
`$HOME/.wg-quickrs/certs/YOUR_SERVER/key.pem`.

Replace `YOUR_SERVER` with your IP address, FQDN, or a domain name.
The following command will create a rootCA cert/key (at `$HOME/.wg-quickrs/certs/root/rootCA.crt`) and use that to sign
`$HOME/.wg-quickrs/certs/YOUR_SERVER/cert.pem`.

```bash
docker run --rm \
  -v "$HOME/.wg-quickrs/certs:/app/certs" \
  -e COUNTRY="XX" \
  -e STATE="XX" \
  -e LOCALITY="XX" \
  -e ORGANIZATION="XX" \
  -e ORGANIZATIONAL_UNIT="XX" \
  -e ROOT_CN="tls-cert-generator@XX" \
  godofkebab/tls-cert-generator \
  YOUR_SERVER
```

<details>
<summary>Example Output</summary>

```text
Generating key for rootCA ...
    certs/root/rootCA.key
    Done.
Generating cert for rootCA ...
    certs/root/rootCA.crt
    Done.
Generating cert/key for YOUR_SERVER ...
    Generated key at certs/servers/YOUR_SERVER/key.pem
    Generated cert at certs/servers/YOUR_SERVER/cert.pem
tree "$HOME/.wg-quickrs/certs"
└── certs
    ├── root
    │   ├── rootCA.crt
    │   └── rootCA.key
    └── servers
        └── YOUR_SERVER
            ├── cert.pem
            └── key.pem
```

</details>

Initialize your agent using the init command:

```bash
docker run --rm \
  --name wg-quickrs-init-cnt \
  -v "$HOME/.wg-quickrs:/app/.wg-quickrs" \
  godofkebab/wg-quickrs \
  init --no-prompt true \
    --network-identifier wg-quickrs-home \
    --network-subnet     10.0.34.0/24    \
    --agent-web-address          0.0.0.0                          \
    --agent-web-http-enabled     true                             \
    --agent-web-http-port        80                               \
    --agent-web-https-enabled    true                             \
    --agent-web-https-port       443                              \
    --agent-web-https-tls-cert   certs/servers/localhost/cert.pem \
    --agent-web-https-tls-key    certs/servers/localhost/key.pem  \
    --agent-web-password-enabled true                             \
    --agent-web-password         TODO_YOUR_PASSWORD_TODO          \
    --agent-vpn-enabled          true                             \
    --agent-vpn-port             51820                            \
    --agent-vpn-gateway          eth0                             \
    --agent-firewall-enabled     true                             \
    --agent-firewall-utility     /usr/sbin/iptables               \
    --agent-peer-name                     wg-quickrs-host                \
    --agent-peer-vpn-endpoint             YOUR_SERVER:51820 \
    --agent-peer-vpn-internal-address     10.0.34.1                      \
    --agent-peer-dns-enabled              true                           \
    --agent-peer-dns-server               1.1.1.1                        \
    --agent-peer-mtu-enabled              false                          \
    --agent-peer-mtu-value                ""                             \
    --agent-peer-script-pre-up-enabled    false                          \
    --agent-peer-script-pre-up-line       ""                             \
    --agent-peer-script-post-up-enabled   false                          \
    --agent-peer-script-post-up-line      ""                             \
    --agent-peer-script-pre-down-enabled  false                          \
    --agent-peer-script-pre-down-line     ""                             \
    --agent-peer-script-post-down-enabled false                          \
    --agent-peer-script-post-down-line    ""                             \
    --default-peer-dns-enabled                        true    \
    --default-peer-dns-server                         1.1.1.1 \
    --default-peer-mtu-enabled                        false   \
    --default-peer-mtu-value                          ""      \
    --default-peer-script-pre-up-enabled              false   \
    --default-peer-script-pre-up-line                 ""      \
    --default-peer-script-post-up-enabled             false   \
    --default-peer-script-post-up-line                ""      \
    --default-peer-script-pre-down-enabled            false   \
    --default-peer-script-pre-down-line               ""      \
    --default-peer-script-post-down-enabled           false   \
    --default-peer-script-post-down-line              ""      \
    --default-connection-persistent-keepalive-enabled true    \
    --default-connection-persistent-keepalive-period  25
```

<details>
<summary>Example Output</summary>

```text
backend: v0.1.0, frontend: v0.0.0, build: unknown#unknown@2025-09-10T03:54:51Z
2025-09-10T04:34:04.818Z INFO  [wg_quickrs] using the wg-quickrs config file at ".wg-quickrs/conf.yml"
2025-09-10T04:34:04.818Z INFO  [wg_quickrs::commands::init] Initializing wg-quickrs...
[general network settings 1-2/24]
	[ 1/24] Using Set VPN network identifier from CLI option '--network-identifier': wg-quickrs-home
	[ 2/24] Using Set VPN network CIDR subnet from CLI option '--network-subnet': 10.0.34.0/24
[general network settings complete]
[agent settings 3-17/24]
	[ 3/24] Using Set agent web server bind IPv4 address from CLI option '--agent-web-address': 0.0.0.0
	[ 4/24] Enable HTTP on web server is enabled from CLI option '--agent-web-http-enabled'
	[ 4/24] Using 	Set web server HTTP port from CLI option '--agent-web-http-port': 80
	[ 5/24] Enable HTTPS on web server is enabled from CLI option '--agent-web-https-enabled'
	[ 5/24] Using 	Set web server HTTPS port from CLI option '--agent-web-https-port': 443
	[ 5/24] Using 	Set path (relative to the wg-quickrs home directory) to TLS certificate file for HTTPS from CLI option '--agent-web-https-tls-cert': certs/servers/localhost/cert.pem
	[ 5/24] Using 	Set path (relative to the wg-quickrs home directory) to TLS private key file for HTTPS from CLI option '--agent-web-https-tls-key': certs/servers/localhost/key.pem
	[ 6/24] Enable password authentication for web server is enabled from CLI option '--agent-web-password-enabled'
	[ 6/24]  Using password for the web server from CLI argument: ***hidden***
	[ 7/24] Enable VPN server is enabled from CLI option '--agent-vpn-enabled'
	[ 7/24] Using 	Set VPN server listening port from CLI option '--agent-vpn-port': 51820
	[ 7/24] Using 	Set gateway (outbound interface) for VPN packet forwarding from CLI option '--agent-vpn-gateway': eth0
	[ 8/24] Enable running firewall commands for setting up NAT and input rules is enabled from CLI option '--agent-firewall-enabled'
	[ 8/24] Using 	Set the utility used to configure firewall NAT and input rules from CLI option '--agent-firewall-utility': /usr/sbin/iptables
	[ 9/24] Using Set agent peer name from CLI option '--agent-peer-name': wg-quickrs-host
	[10/24] Using Set publicly accessible endpoint(IP/FQDN:PORT) for VPN endpoint from CLI option '--agent-peer-vpn-endpoint': YOUR_SERVER:51820
	[11/24] Using Set internal IPv4 address for agent in VPN network from CLI option '--agent-peer-vpn-internal-address': 10.0.34.1
	[12/24] Enable DNS configuration for agent is enabled from CLI option '--agent-peer-dns-enabled'
	[12/24] Using 	Set DNS server for agent from CLI option '--agent-peer-dns-server': 1.1.1.1
	[13/24] Enable MTU configuration for agent is disabled from CLI option '--agent-peer-mtu-enabled'
	[13/24] Using 	Set MTU value for agent from CLI option '--agent-peer-mtu-value':
	[14/24] Enable PreUp script for agent is disabled from CLI option '--agent-peer-script-pre-up-enabled'
	[14/24] Using 	Set PreUp script line for agent from CLI option '--agent-peer-script-pre-up-line':
	[15/24] Enable PostUp script for agent is disabled from CLI option '--agent-peer-script-post-up-enabled'
	[15/24] Using 	Set PostUp script line for agent from CLI option '--agent-peer-script-post-up-line':
	[16/24] Enable PreDown script for agent is disabled from CLI option '--agent-peer-script-pre-down-enabled'
	[16/24] Using 	Set PreDown script line for agent from CLI option '--agent-peer-script-pre-down-line':
	[17/24] Enable PostDown script for agent is disabled from CLI option '--agent-peer-script-post-down-enabled'
	[17/24] Using 	Set PostDown script line for agent from CLI option '--agent-peer-script-post-down-line':
[agent settings complete]
[new peer/connection default settings 18-24/24]
	[18/24] Enable DNS for new peers by default is enabled from CLI option '--default-peer-dns-enabled'
	[18/24] Using 	Set default DNS server for new peers from CLI option '--default-peer-dns-server': 1.1.1.1
	[19/24] Enable MTU for new peers by default is disabled from CLI option '--default-peer-mtu-enabled'
	[19/24] Using 	Set default MTU value for new peers from CLI option '--default-peer-mtu-value':
	[20/24] Enable PreUp script for new peers by default is disabled from CLI option '--default-peer-script-pre-up-enabled'
	[20/24] Using 	Set default PreUp script line for new peers from CLI option '--default-peer-script-pre-up-line':
	[21/24] Enable PostUp script for new peers by default is disabled from CLI option '--default-peer-script-post-up-enabled'
	[21/24] Using 	Set default PostUp script line for new peers from CLI option '--default-peer-script-post-up-line':
	[22/24] Enable PreDown script for new peers by default is disabled from CLI option '--default-peer-script-pre-down-enabled'
	[22/24] Using 	Set default PreDown script line for new peers from CLI option '--default-peer-script-pre-down-line':
	[23/24] Enable PostDown script for new peers by default is disabled from CLI option '--default-peer-script-post-down-enabled'
	[23/24] Using 	Set default PostDown script line for new peers from CLI option '--default-peer-script-post-down-line':
	[24/24] Enable PersistentKeepalive for new connections by default is enabled from CLI option '--default-connection-persistent-keepalive-enabled'
	[24/24] Using 	Set default PersistentKeepalive period in seconds from CLI option '--default-connection-persistent-keepalive-period': 25
[new peer/connection default settings complete]
✅ This was all the information required to initialize wg-quickrs. Finalizing the configuration...
2025-09-10T04:34:04.837Z INFO  [wg_quickrs::wireguard::cmd] $ wg genkey
2025-09-10T04:34:04.837Z INFO  [wg_quickrs::wireguard::cmd] $ wg genkey | wg pubkey
2025-09-10T04:34:04.839Z INFO  [wg_quickrs::conf::util] updated config file
```

</details>

---

Then start the agent like so:

```bash
docker run \
  --name wg-quickrs-agent-run-cnt \
  -v "$HOME/.wg-quickrs:/app/.wg-quickrs" \
  --cap-add NET_ADMIN \
  --cap-add SYS_MODULE \
  --sysctl net.ipv4.ip_forward=1 \
  --sysctl net.ipv4.conf.all.src_valid_mark=1 \
  -p 80:8080/tcp \
  -p 443:8443/tcp \
  -p 51820:51820/udp \
  godofkebab/wg-quickrs \
  agent run
```

<details>
<summary>Example Output</summary>

```text
backend: v0.1.0, frontend: v0.1.0, build: v0.1.11@2025-09-10T17:03:53Z
2025-09-11T21:25:33.440Z INFO  [wg_quickrs] using the wg-quickrs config file at ".wg-quickrs/conf.yml"
2025-09-11T21:25:33.442Z INFO  [wg_quickrs::commands::agent] using the wireguard config file at "/etc/wireguard/wg-quickrs.conf"
2025-09-11T21:25:33.454Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg-quick down wg-quickrs
2025-09-11T21:25:33.454Z WARN  [wg_quickrs::wireguard::cmd] wg-quick: `wg-quickrs' is not a WireGuard interface

2025-09-11T21:25:33.627Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg-quick up wg-quickrs
2025-09-11T21:25:33.627Z WARN  [wg_quickrs::wireguard::cmd] [#] ip link add dev wg-quickrs type wireguard
[#] wg setconf wg-quickrs /dev/fd/63
[#] ip -4 address add 10.0.34.1/24 dev wg-quickrs
[#] ip link set mtu 65455 up dev wg-quickrs
[#] resolvconf -a wg-quickrs -m 0 -x
could not detect a useable init system
[#] sudo sysctl -w net.ipv4.ip_forward=1
sysctl: error setting key 'net.ipv4.ip_forward': Read-only file system
[#] resolvconf -d wg-quickrs -f
could not detect a useable init system
[#] ip link delete dev wg-quickrs

2025-09-11T21:25:33.627Z ERROR [wg_quickrs::commands::agent] wireguard::cmd::error::command_exec_not_successful -> command for $ sudo wg-quick up wg-quickrs completed unsuccessfully
2025-09-11T21:25:33.628Z INFO  [wg_quickrs::web::server] Started HTTP frontend/API at http://0.0.0.0:80/
2025-09-11T21:25:33.628Z INFO  [actix_server::builder] starting 14 workers
2025-09-11T21:25:33.635Z INFO  [wg_quickrs::web::server] Started HTTPS frontend/API at https://0.0.0.0:443/
2025-09-11T21:25:33.635Z INFO  [actix_server::builder] starting 14 workers
2025-09-11T21:25:33.635Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
2025-09-11T21:25:33.635Z INFO  [actix_server::server] starting service: "actix-web-service-0.0.0.0:80", workers: 14, listening on: 0.0.0.0:80
2025-09-11T21:25:33.643Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
2025-09-11T21:25:33.643Z INFO  [actix_server::server] starting service: "actix-web-service-0.0.0.0:443", workers: 14, listening on: 0.0.0.0:443
```

</details>

