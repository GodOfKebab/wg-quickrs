# wg-quickrs quick-start guide for docker hosts

## 1. Use the pre-built Docker images (recommended)

## 1.1 Generate your TLS certs/keys (optional)

Generate your TLS certs/keys to `$HOME/.wg-quickrs-docker/certs/YOUR-SERVER/cert.pem`/`$HOME/.wg-quickrs-docker/certs/YOUR-SERVER/key.pem`.

Replace `YOUR-SERVER` with your IP address, FQDN, or a domain name.
The following command will create a rootCA cert/key (at `$HOME/.wg-quickrs-docker/certs/root/rootCA.crt`) and use that to sign
`$HOME/.wg-quickrs-docker/certs/YOUR-SERVER/cert.pem`.

```bash
docker run --rm \
  -v "$HOME/.wg-quickrs-docker/certs:/app/certs" \
  -e COUNTRY="XX" \
  -e STATE="XX" \
  -e LOCALITY="XX" \
  -e ORGANIZATION="XX" \
  -e ORGANIZATIONAL_UNIT="XX" \
  -e ROOT_CN="tls-cert-generator@XX" \
  godofkebab/tls-cert-generator \
  YOUR-SERVER
```

---

## 1.2 Initialize the agent

Initialize your agent using the init command:

```bash
docker run --rm \
  --name wg-quickrs-init-cnt \
  -v "$HOME/.wg-quickrs-docker:/app/.wg-quickrs" \
  godofkebab/wg-quickrs \
  agent init --no-prompt true \
    --network-name   wg-quickrs-home \
    --network-subnet 10.0.34.0/24    \
    --agent-web-address          0.0.0.0                            \
    --agent-web-http-enabled     true                               \
    --agent-web-http-port        80                                 \
    --agent-web-https-enabled    true                               \
    --agent-web-https-port       443                                \
    --agent-web-https-tls-cert   certs/servers/YOUR-SERVER/cert.pem \
    --agent-web-https-tls-key    certs/servers/YOUR-SERVER/key.pem  \
    --agent-web-password-enabled true                               \
    --agent-web-password         YOUR-PASSWORD                      \
    --agent-vpn-enabled          true                               \
    --agent-vpn-port             51820                              \
    --agent-firewall-enabled     true                               \
    --agent-firewall-utility     /usr/sbin/iptables                 \
    --agent-firewall-gateway     eth0                               \
    --agent-firewall-configure-http  true                           \
    --agent-firewall-http-automated  true                           \
    --agent-firewall-configure-https true                           \
    --agent-firewall-https-automated true                           \
    --agent-firewall-configure-vpn   true                           \
    --agent-firewall-vpn-automated   true                           \
    --agent-peer-name                     wg-quickrs-host   \
    --agent-peer-vpn-internal-address     10.0.34.1         \
    --agent-peer-vpn-endpoint             YOUR-SERVER:51820 \
    --agent-peer-kind                     server            \
    --agent-peer-icon-enabled             false             \
    --agent-peer-dns-enabled              true              \
    --agent-peer-dns-addresses            1.1.1.1           \
    --agent-peer-mtu-enabled              false             \
    --agent-peer-script-pre-up-enabled    false             \
    --agent-peer-script-post-up-enabled   false             \
    --agent-peer-script-pre-down-enabled  false             \
    --agent-peer-script-post-down-enabled false             \
    --default-peer-kind                               laptop  \
    --default-peer-icon-enabled                       false   \
    --default-peer-dns-enabled                        true    \
    --default-peer-dns-addresses                      1.1.1.1 \
    --default-peer-mtu-enabled                        false   \
    --default-peer-script-pre-up-enabled              false   \
    --default-peer-script-post-up-enabled             false   \
    --default-peer-script-pre-down-enabled            false   \
    --default-peer-script-post-down-enabled           false   \
    --default-connection-persistent-keepalive-enabled true    \
    --default-connection-persistent-keepalive-period  25
```

---

## 1.3 Run the agent

Then start the agent and fork it in the background like so:

```bash
docker run -d \
  --name wg-quickrs-agent-run-cnt \
  -v "$HOME/.wg-quickrs-docker:/app/.wg-quickrs" \
  --cap-add NET_ADMIN \
  --cap-add SYS_MODULE \
  --sysctl net.ipv4.ip_forward=1 \
  --sysctl net.ipv4.conf.all.src_valid_mark=1 \
  -p 8443:443/tcp \
  -p 51820:51820/udp \
  --restart unless-stopped \
  godofkebab/wg-quickrs \
  agent run
```

HTTPS server will be available at `https://YOUR-SERVER:8443`.
WireGuard endpoint will be available at `YOUR-SERVER:51820`.

## 1.4 Reset web password

If you need to reset the web password in the future, make sure the 'agent run' container is not running and run the following command:

```bash
docker run --rm \
  -v "$HOME/.wg-quickrs-docker:/app/.wg-quickrs" \
  godofkebab/wg-quickrs \
  config reset agent web password --password NEW_PASSWORD
```

⚠️ Note: Keep in mind that the plaintext password might show up in the bash/zsh history.
If you instead use the binaries instead of docker, `wg-quickrs config reset agent web password` prompts for the password interactively, which is safer.

### 2. Build the Docker images from source

See [BUILDING.md](../BUILDING.md#12-using-docker)

