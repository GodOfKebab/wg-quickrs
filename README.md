# wg-quickrs

[![License](https://img.shields.io/github/license/godofkebab/wg-quickrs?logo=GitHub&color=brightgreen)](https://github.com/GodOfKebab/wg-quickrs)
![Static Badge](https://img.shields.io/badge/amd64%20%7C%20arm64%20%7C%20arm%2Fv7%20%20-%20grey?label=arch)
![Static Badge](https://img.shields.io/badge/Linux%20%7C%20macOS%20%20-%20black?label=platform)

[![Release](https://img.shields.io/github/v/tag/godofkebab/wg-quickrs?logo=github&label=latest%20tag&color=blue)](https://github.com/godofkebab/wg-quickrs/releases/latest)
[![Docker](https://img.shields.io/docker/image-size/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs)
[![Docker](https://img.shields.io/docker/pulls/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs/tags)
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs%2FCargo.toml&query=package.rust-version&logo=rust&label=rust&color=%23000000)
![Dynamic JSON Badge](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs-web%2Fpackage.json&query=dependencies.vue&logo=vue.js&label=vue&color=%234FC08D)

✨ An intuitive multi-peer `wg` / `wg-quick` wrapper written in 🦀 Rust.

⚡ Rust + Vue + WASM + WireGuard = 🧪 one static binary + 📝 one YAML file to rule them all 🪄

Run it on your server, router, or computer and manage your WireGuard VPN from a terminal or a web interface.

<p align="center">
<img src="https://yasar.idikut.cc/project-assets/wg-quickrs-homepage.gif" height="700" alt="demo">
</p>

Features:
- Interactive graph to configure your P2P network
- Sliding traffic graph to visualize your host's network traffic
- Secure API access via HTTPS with support for password login and JWT authentication
- Automatic firewall/NAT setup (`iptables` for Debian/Linux or `pf` for macOS, both come preinstalled with the OS)
- If you are not feeling like dealing with VPN/networking on your machine, you can also just use the web console to create `.conf` files/QR codes for your network peers.

---

## Quick Start Guide

### Requirements

- `wireguard-tools` (`wg(8)` and `wg-quick(8)` utilities)
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

Then add your cert to `~/.wg-quickrs/certs/YOUR-SERVER/cert.pem` and key to `~/.wg-quickrs/certs/YOUR-SERVER/key.pem`.

---

### 2. Use the pre-built Docker image

(Optionally) generate your TLS certs/keys to `$HOME/.wg-quickrs/certs/YOUR-SERVER/cert.pem`/
`$HOME/.wg-quickrs/certs/YOUR-SERVER/key.pem`.

Replace `YOUR-SERVER` with your IP address, FQDN, or a domain name.
The following command will create a rootCA cert/key (at `$HOME/.wg-quickrs/certs/root/rootCA.crt`) and use that to sign
`$HOME/.wg-quickrs/certs/YOUR-SERVER/cert.pem`.

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
  YOUR-SERVER
```

<details>
<summary>Example Output</summary>

```text
✨  Welcome to tls-cert-generator!
📋 Current configuration:
   FORCE               (-f)         = 0
   CERTS_DIR           (-o)         = certs
   COUNTRY             (--country)  = XX
   STATE               (--state)    = XX
   LOCALITY            (--locality) = XX
   ORGANIZATION        (--org)      = XX
   ORGANIZATIONAL_UNIT (--ou)       = XX
   ROOT_CN             (--cn)       = tls-cert-generator@XX

🔎 Detected key for rootCA at certs/root/rootCA.key. Use -f option to override. Skipping...
🔎 Detected cert for rootCA at certs/root/rootCA.crt. Use -f option to override. Skipping...
⏳ Generating cert/key for YOUR-SERVER ...
    ✅ Success: certs/servers/YOUR-SERVER/key.pem
    ✅ Success: certs/servers/YOUR-SERVER.csr
    ✅ Success: certs/servers/YOUR-SERVER/cert.pem
# tree "$HOME/.wg-quickrs/certs"
# └── certs
#     ├── root
#     │   ├── rootCA.crt
#     │   └── rootCA.key
#     └── servers
#         └── YOUR-SERVER
#             ├── cert.pem
#             └── key.pem
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
    --agent-web-address          0.0.0.0                            \
    --agent-web-http-enabled     true                               \
    --agent-web-http-port        80                                 \
    --agent-web-https-enabled    true                               \
    --agent-web-https-port       443                                \
    --agent-web-https-tls-cert   certs/servers/YOUR-SERVER/cert.pem \
    --agent-web-https-tls-key    certs/servers/YOUR-SERVER/key.pem  \
    --agent-web-password-enabled true                               \
    --agent-web-password         YOUR_PASSWORD                      \
    --agent-vpn-enabled          true                               \
    --agent-vpn-port             51820                              \
    --agent-firewall-enabled     true                               \
    --agent-firewall-utility     /usr/sbin/iptables                 \
    --agent-firewall-gateway     eth0                               \
    --agent-peer-name                     wg-quickrs-host                \
    --agent-peer-vpn-internal-address     10.0.34.1                      \
    --agent-peer-vpn-endpoint             YOUR-SERVER:51820              \
    --agent-peer-kind                     server                         \
    --agent-peer-icon-enabled             false                          \
    --agent-peer-dns-enabled              true                           \
    --agent-peer-dns-server               1.1.1.1                        \
    --agent-peer-mtu-enabled              false                          \
    --agent-peer-script-pre-up-enabled    false                          \
    --agent-peer-script-post-up-enabled   false                          \
    --agent-peer-script-pre-down-enabled  false                          \
    --agent-peer-script-post-down-enabled false                          \
    --default-peer-kind                               laptop  \
    --default-peer-icon-enabled                       false   \
    --default-peer-dns-enabled                        true    \
    --default-peer-dns-server                         1.1.1.1 \
    --default-peer-mtu-enabled                        false   \
    --default-peer-script-pre-up-enabled              false   \
    --default-peer-script-post-up-enabled             false   \
    --default-peer-script-pre-down-enabled            false   \
    --default-peer-script-post-down-enabled           false   \
    --default-connection-persistent-keepalive-enabled true    \
    --default-connection-persistent-keepalive-period  25
```

<details>
<summary>Example Output</summary>

```text
version: 1.0.0-rc | build: v1.0.0-rc#6f7351d@2025-10-20T04:37:28Z
2025-10-20T20:13:17.116Z INFO  [wg_quickrs] using the wg-quickrs config file at "/app/.wg-quickrs/conf.yml"
2025-10-20T20:13:17.116Z INFO  [wg_quickrs::commands::init] Initializing wg-quickrs...
[general network settings 1-2/28]
	[ 1/28] Using Set VPN network identifier from CLI option '--network-identifier': wg-quickrs-home
	[ 2/28] Using Set VPN network CIDR subnet from CLI option '--network-subnet': 10.0.34.0/24
[general network settings complete]
[agent settings 3-8/28]
	[ 3/28] Using Set agent web server bind IPv4 address from CLI option '--agent-web-address': 0.0.0.0
	[ 4/28] Enable HTTP on web server is enabled from CLI option '--agent-web-http-enabled'
	[ 4/28] Using 	Set web server HTTP port from CLI option '--agent-web-http-port': 80
	[ 5/28] Enable HTTPS on web server is enabled from CLI option '--agent-web-https-enabled'
	[ 5/28] Using 	Set web server HTTPS port from CLI option '--agent-web-https-port': 443
	[ 5/28] Using 	Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS from CLI option '--agent-web-https-tls-cert': certs/servers/YOUR-SERVER/cert.pem
	[ 5/28] Using 	Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS from CLI option '--agent-web-https-tls-key': certs/servers/YOUR-SERVER/key.pem
	[ 6/28] Enable password authentication for web server is enabled from CLI option '--agent-web-password-enabled'
	[ 6/28]  Using password for the web server from CLI argument: ***hidden***
	[ 7/28] Enable VPN server is enabled from CLI option '--agent-vpn-enabled'
	[ 7/28] Using 	Set VPN server listening port from CLI option '--agent-vpn-port': 51820
	[ 8/28] Enable running firewall commands for setting up NAT and input rules is enabled from CLI option '--agent-firewall-enabled'
	[ 8/28] Using 	Set the utility used to configure firewall NAT and input rules from CLI option '--agent-firewall-utility': /usr/sbin/iptables
	[ 8/28] Using 	Set gateway (outbound interface) for VPN packet forwarding from CLI option '--agent-firewall-gateway': eth0
[agent settings complete]
[peer settings 9-19/28]
	[ 9/28] Using Set agent peer name from CLI option '--agent-peer-name': wg-quickrs-host
	[10/28] Using Set internal IPv4 address for agent in VPN network from CLI option '--agent-peer-vpn-internal-address': 10.0.34.1
	[11/28] Using Set publicly accessible endpoint(IP/FQDN:PORT) for VPN endpoint from CLI option '--agent-peer-vpn-endpoint': YOUR-SERVER:51820
	[12/28] Using Set peer kind for agent from CLI option '--agent-peer-kind': server
	[13/28] Enable peer icon for agent is disabled from CLI option '--agent-peer-icon-enabled'
	[14/28] Enable DNS configuration for agent is enabled from CLI option '--agent-peer-dns-enabled'
	[14/28] Using 	Set DNS server for agent from CLI option '--agent-peer-dns-server': 1.1.1.1
	[15/28] Enable MTU configuration for agent is disabled from CLI option '--agent-peer-mtu-enabled'
	[16/28] Enable PreUp script for agent is disabled from CLI option '--agent-peer-script-pre-up-enabled'
	[17/28] Enable PostUp script for agent is disabled from CLI option '--agent-peer-script-post-up-enabled'
	[18/28] Enable PreDown script for agent is disabled from CLI option '--agent-peer-script-pre-down-enabled'
	[19/28] Enable PostDown script for agent is disabled from CLI option '--agent-peer-script-post-down-enabled'
[peer settings complete]
[new peer/connection default settings 20-28/28]
	[20/28] Using Set peer kind for new peers by default from CLI option '--default-peer-kind': laptop
	[21/28] Enable peer icon for new peers by default is disabled from CLI option '--default-peer-icon-enabled'
	[22/28] Enable DNS for new peers by default is enabled from CLI option '--default-peer-dns-enabled'
	[22/28] Using 	Set default DNS server for new peers from CLI option '--default-peer-dns-server': 1.1.1.1
	[23/28] Enable MTU for new peers by default is disabled from CLI option '--default-peer-mtu-enabled'
	[24/28] Enable PreUp script for new peers by default is disabled from CLI option '--default-peer-script-pre-up-enabled'
	[25/28] Enable PostUp script for new peers by default is disabled from CLI option '--default-peer-script-post-up-enabled'
	[26/28] Enable PreDown script for new peers by default is disabled from CLI option '--default-peer-script-pre-down-enabled'
	[27/28] Enable PostDown script for new peers by default is disabled from CLI option '--default-peer-script-post-down-enabled'
	[28/28] Enable PersistentKeepalive for new connections by default is enabled from CLI option '--default-connection-persistent-keepalive-enabled'
	[28/28] Using 	Set default PersistentKeepalive period in seconds from CLI option '--default-connection-persistent-keepalive-period': 25
[new peer/connection default settings complete]
✅ This was all the information required to initialize wg-quickrs. Finalizing the configuration...
2025-10-20T20:13:17.132Z INFO  [wg_quickrs::conf::util] updated config file
✅ Configuration saved to /app/.wg-quickrs/conf.yml
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
  -p 80:80/tcp \
  -p 443:443/tcp \
  -p 51820:51820/udp \
  godofkebab/wg-quickrs \
  agent run
```

<details>
<summary>Example Output</summary>

```text
version: 1.0.0-rc | build: v1.0.0-rc#6f7351d@2025-10-20T04:36:32Z
2025-10-20T20:29:12.013Z INFO  [wg_quickrs] using the wg-quickrs config file at "/app/.wg-quickrs/conf.yml"
2025-10-20T20:29:12.014Z INFO  [wg_quickrs::conf::util] loaded config file
2025-10-20T20:29:12.015Z INFO  [wg_quickrs::web::server] $ /usr/sbin/iptables -A INPUT -p tcp --dport 443 -j ACCEPT
2025-10-20T20:29:12.018Z INFO  [wg_quickrs::web::server] $ /usr/sbin/iptables -A INPUT -p tcp --dport 80 -j ACCEPT
2025-10-20T20:29:12.018Z INFO  [wg_quickrs::web::server] Starting HTTP server at http://0.0.0.0:80/
2025-10-20T20:29:12.018Z INFO  [actix_server::builder] starting 2 workers
2025-10-20T20:29:12.018Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
2025-10-20T20:29:12.018Z INFO  [actix_server::server] starting service: "actix-web-service-0.0.0.0:80", workers: 2, listening on: 0.0.0.0:80
2025-10-20T20:29:12.020Z INFO  [wg_quickrs::web::server] Starting HTTPS server at https://0.0.0.0:443/
2025-10-20T20:29:12.020Z INFO  [actix_server::builder] starting 2 workers
2025-10-20T20:29:12.020Z INFO  [actix_server::server] Actix runtime found; starting in Actix runtime
2025-10-20T20:29:12.020Z INFO  [actix_server::server] starting service: "actix-web-service-0.0.0.0:443", workers: 2, listening on: 0.0.0.0:443
2025-10-20T20:29:12.022Z INFO  [wg_quickrs::wireguard::cmd] using the wireguard config file at "/etc/wireguard/wg-quickrs-home.conf"
2025-10-20T20:29:12.024Z INFO  [wg_quickrs::wireguard::cmd] Always disable wireguard first on startup
2025-10-20T20:29:12.024Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg-quick down wg-quickrs-home
2025-10-20T20:29:12.054Z WARN  [wg_quickrs::wireguard::cmd] wg-quick: `wg-quickrs-home' is not a WireGuard interface

2025-10-20T20:29:12.054Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg-quick up wg-quickrs-home
2025-10-20T20:29:12.225Z WARN  [wg_quickrs::wireguard::cmd] [#] ip link add dev wg-quickrs-home type wireguard
[#] wg setconf wg-quickrs-home /dev/fd/63
[#] ip -4 address add 10.0.34.1/24 dev wg-quickrs-home
[#] ip link set mtu 1420 up dev wg-quickrs-home
[#] resolvconf -a wg-quickrs-home -m 0 -x
could not detect a useable init system
[#] /usr/sbin/iptables -t nat -A POSTROUTING -s 10.0.34.0/24 -o eth0 -j MASQUERADE;
[#] /usr/sbin/iptables -A INPUT -p udp -m udp --dport 51820 -j ACCEPT;
[#] /usr/sbin/iptables -A FORWARD -i wg-quickrs-home -j ACCEPT;
[#] /usr/sbin/iptables -A FORWARD -o wg-quickrs-home -j ACCEPT;

2025-10-20T20:29:12.225Z INFO  [wg_quickrs::wireguard::cmd] Started the wireguard tunnel at 0.0.0.0:51820
2025-10-20T20:30:46.225Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg show wg-quickrs-home dump
2025-10-20T20:30:47.225Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg show wg-quickrs-home dump
2025-10-20T20:30:48.080Z INFO  [wg_quickrs::conf::respond] reserved address 10.0.34.2 for da62bd89-9f2f-4980-9121-5d9d2c790a6f until 2025-10-20T20:40:48Z
2025-10-20T20:30:48.081Z INFO  [wg_quickrs::conf::util] updated config file
2025-10-20T20:30:48.081Z INFO  [wg_quickrs::conf::respond] updated config file
2025-10-20T20:30:48.226Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg show wg-quickrs-home dump
2025-10-20T20:30:49.226Z INFO  [wg_quickrs::wireguard::cmd] $ sudo wg show wg-quickrs-home dump
...
```

</details>

