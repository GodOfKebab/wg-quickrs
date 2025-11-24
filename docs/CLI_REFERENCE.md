# `wg-quickrs`

A tool to manage the peer and network configuration of the WireGuard-based overlay network over the web console

**Usage:** `Usage: wg-quickrs [OPTIONS] <COMMAND>`

###### **Subcommands:**
* `agent` — Run agent commands
* `config` — Edit agent configuration options

###### **Options:**

* `-v`, `--verbose` — Increase verbosity level from Info to Debug
* `--wg-quickrs-config-folder <WG_QUICKRS_CONFIG_FOLDER>`

**Command Overview:**
* [`wg-quickrs agent`↴](#wg-quickrs-agent)
* [`wg-quickrs config`↴](#wg-quickrs-config)

---

## `wg-quickrs agent`


**Subcommand Overview:**

* [`agent`↴](#agent)
* [`agent init`↴](#agent-init)
* [`agent run`↴](#agent-run)

### `agent`

Run agent commands

**Usage:** `agent <COMMAND>`

###### **Subcommands:**

* `init` — Initialize the wg-quickrs agent.
Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command
* `run` — Run the wg-quickrs agent



### `agent init`

Initialize the wg-quickrs agent.
Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command

**Usage:** `agent init [OPTIONS]`

###### **Options:**

* `--network-name <wg-quickrs-home>` — Set VPN network name
* `--network-subnet <10.0.34.0/24>` — Set VPN network CIDR subnet
* `--agent-web-address <AGENT_WEB_ADDRESS>` — Set agent web server bind IPv4 address
* `--agent-web-http-enabled <AGENT_WEB_HTTP_ENABLED>` — Enable HTTP on web server

  Possible values: `true`, `false`

* `--agent-web-http-port <80>` — Set web server HTTP port
* `--agent-web-https-enabled <AGENT_WEB_HTTPS_ENABLED>` — Enable HTTPS on web server

  Possible values: `true`, `false`

* `--agent-web-https-port <443>` — Set web server HTTPS port
* `--agent-web-https-tls-cert <certs/servers/localhost/cert.pem>` — Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS
* `--agent-web-https-tls-key <certs/servers/localhost/key.pem>` — Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS
* `--agent-web-password-enabled <AGENT_WEB_PASSWORD_ENABLED>` — Enable password authentication for web server

  Possible values: `true`, `false`

* `--agent-web-password <AGENT_WEB_PASSWORD>` — Set password for web server access
* `--agent-vpn-enabled <AGENT_VPN_ENABLED>` — Enable VPN server

  Possible values: `true`, `false`

* `--agent-vpn-port <51820>` — Set VPN server listening port
* `--agent-firewall-enabled <AGENT_FIREWALL_ENABLED>` — Enable running firewall commands for setting up NAT and input rules

  Possible values: `true`, `false`

* `--agent-firewall-utility <iptables>` — Set the utility used to configure firewall NAT and input rules
* `--agent-firewall-gateway <eth0>` — Set gateway (outbound interface) for VPN packet forwarding
* `--agent-firewall-configure-http <AGENT_FIREWALL_CONFIGURE_HTTP>` — Configure HTTP firewall

  Possible values: `true`, `false`

* `--agent-firewall-http-automated <AGENT_FIREWALL_HTTP_AUTOMATED>` — Use automated setup for HTTP firewall

  Possible values: `true`, `false`

* `--agent-firewall-configure-https <AGENT_FIREWALL_CONFIGURE_HTTPS>` — Configure HTTPS firewall

  Possible values: `true`, `false`

* `--agent-firewall-https-automated <AGENT_FIREWALL_HTTPS_AUTOMATED>` — Use automated setup for HTTPS firewall

  Possible values: `true`, `false`

* `--agent-firewall-configure-vpn <AGENT_FIREWALL_CONFIGURE_VPN>` — Configure VPN firewall

  Possible values: `true`, `false`

* `--agent-firewall-vpn-automated <AGENT_FIREWALL_VPN_AUTOMATED>` — Use automated setup for VPN firewall

  Possible values: `true`, `false`

* `--agent-firewall-http-pre-up-enabled <AGENT_FIREWALL_HTTP_PRE_UP_ENABLED>` — Enable HTTP firewall PreUp scripts

  Possible values: `true`, `false`

* `--agent-firewall-http-pre-up-line <AGENT_FIREWALL_HTTP_PRE_UP_LINE>` — Set HTTP firewall PreUp script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-http-post-down-enabled <AGENT_FIREWALL_HTTP_POST_DOWN_ENABLED>` — Enable HTTP firewall PostDown scripts

  Possible values: `true`, `false`

* `--agent-firewall-http-post-down-line <AGENT_FIREWALL_HTTP_POST_DOWN_LINE>` — Set HTTP firewall PostDown script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-https-pre-up-enabled <AGENT_FIREWALL_HTTPS_PRE_UP_ENABLED>` — Enable HTTPS firewall PreUp scripts

  Possible values: `true`, `false`

* `--agent-firewall-https-pre-up-line <AGENT_FIREWALL_HTTPS_PRE_UP_LINE>` — Set HTTPS firewall PreUp script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-https-post-down-enabled <AGENT_FIREWALL_HTTPS_POST_DOWN_ENABLED>` — Enable HTTPS firewall PostDown scripts

  Possible values: `true`, `false`

* `--agent-firewall-https-post-down-line <AGENT_FIREWALL_HTTPS_POST_DOWN_LINE>` — Set HTTPS firewall PostDown script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-vpn-pre-up-enabled <AGENT_FIREWALL_VPN_PRE_UP_ENABLED>` — Enable VPN firewall PreUp scripts

  Possible values: `true`, `false`

* `--agent-firewall-vpn-pre-up-line <AGENT_FIREWALL_VPN_PRE_UP_LINE>` — Set VPN firewall PreUp script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-vpn-post-up-enabled <AGENT_FIREWALL_VPN_POST_UP_ENABLED>` — Enable VPN firewall PostUp scripts

  Possible values: `true`, `false`

* `--agent-firewall-vpn-post-up-line <AGENT_FIREWALL_VPN_POST_UP_LINE>` — Set VPN firewall PostUp script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-vpn-pre-down-enabled <AGENT_FIREWALL_VPN_PRE_DOWN_ENABLED>` — Enable VPN firewall PreDown scripts

  Possible values: `true`, `false`

* `--agent-firewall-vpn-pre-down-line <AGENT_FIREWALL_VPN_PRE_DOWN_LINE>` — Set VPN firewall PreDown script line(s). Can be specified multiple times for multiple script lines.
* `--agent-firewall-vpn-post-down-enabled <AGENT_FIREWALL_VPN_POST_DOWN_ENABLED>` — Enable VPN firewall PostDown scripts

  Possible values: `true`, `false`

* `--agent-firewall-vpn-post-down-line <AGENT_FIREWALL_VPN_POST_DOWN_LINE>` — Set VPN firewall PostDown script line(s). Can be specified multiple times for multiple script lines.
* `--agent-peer-name <wg-quickrs-host>` — Set agent peer name
* `--agent-peer-vpn-internal-address <10.0.34.1>` — Set internal IPv4 address for agent in VPN network
* `--agent-peer-vpn-endpoint <AGENT_PEER_VPN_ENDPOINT>` — Set publicly accessible endpoint(IP/FQDN:PORT) for VPN endpoint
* `--agent-peer-kind <AGENT_PEER_KIND>` — Set peer kind for agent
* `--agent-peer-icon-enabled <AGENT_PEER_ICON_ENABLED>` — Enable peer icon for agent

  Possible values: `true`, `false`

* `--agent-peer-icon-src <AGENT_PEER_ICON_SRC>` — Set peer icon for agent
* `--agent-peer-dns-enabled <AGENT_PEER_DNS_ENABLED>` — Enable DNS configuration for agent

  Possible values: `true`, `false`

* `--agent-peer-dns-addresses <1.1.1.1>` — Set DNS address for agent
* `--agent-peer-mtu-enabled <AGENT_PEER_MTU_ENABLED>` — Enable MTU configuration for agent

  Possible values: `true`, `false`

* `--agent-peer-mtu-value <1420>` — Set MTU value for agent
* `--agent-peer-script-pre-up-enabled <AGENT_PEER_SCRIPT_PRE_UP_ENABLED>` — Enable PreUp script for agent

  Possible values: `true`, `false`

* `--agent-peer-script-pre-up-line <AGENT_PEER_SCRIPT_PRE_UP_LINE>` — Set PreUp script line(s) for agent. Can be specified multiple times for multiple script lines.
* `--agent-peer-script-post-up-enabled <AGENT_PEER_SCRIPT_POST_UP_ENABLED>` — Enable PostUp script for agent

  Possible values: `true`, `false`

* `--agent-peer-script-post-up-line <AGENT_PEER_SCRIPT_POST_UP_LINE>` — Set PostUp script line(s) for agent. Can be specified multiple times for multiple script lines.
* `--agent-peer-script-pre-down-enabled <AGENT_PEER_SCRIPT_PRE_DOWN_ENABLED>` — Enable PreDown script for agent

  Possible values: `true`, `false`

* `--agent-peer-script-pre-down-line <AGENT_PEER_SCRIPT_PRE_DOWN_LINE>` — Set PreDown script line(s) for agent. Can be specified multiple times for multiple script lines.
* `--agent-peer-script-post-down-enabled <AGENT_PEER_SCRIPT_POST_DOWN_ENABLED>` — Enable PostDown script for agent

  Possible values: `true`, `false`

* `--agent-peer-script-post-down-line <AGENT_PEER_SCRIPT_POST_DOWN_LINE>` — Set PostDown script line(s) for agent. Can be specified multiple times for multiple script lines.
* `--default-peer-kind <DEFAULT_PEER_KIND>` — Set peer kind for new peers by default
* `--default-peer-icon-enabled <DEFAULT_PEER_ICON_ENABLED>` — Enable peer icon for new peers by default

  Possible values: `true`, `false`

* `--default-peer-icon-src <DEFAULT_PEER_ICON_SRC>` — Set peer icon for new peers by default
* `--default-peer-dns-enabled <DEFAULT_PEER_DNS_ENABLED>` — Enable DNS for new peers by default

  Possible values: `true`, `false`

* `--default-peer-dns-addresses <1.1.1.1>` — Set default DNS address for new peers
* `--default-peer-mtu-enabled <DEFAULT_PEER_MTU_ENABLED>` — Enable MTU for new peers by default

  Possible values: `true`, `false`

* `--default-peer-mtu-value <1420>` — Set default MTU value for new peers
* `--default-peer-script-pre-up-enabled <DEFAULT_PEER_SCRIPT_PRE_UP_ENABLED>` — Enable PreUp script for new peers by default

  Possible values: `true`, `false`

* `--default-peer-script-pre-up-line <DEFAULT_PEER_SCRIPT_PRE_UP_LINE>` — Set default PreUp script line(s) for new peers. Can be specified multiple times for multiple script lines.
* `--default-peer-script-post-up-enabled <DEFAULT_PEER_SCRIPT_POST_UP_ENABLED>` — Enable PostUp script for new peers by default

  Possible values: `true`, `false`

* `--default-peer-script-post-up-line <DEFAULT_PEER_SCRIPT_POST_UP_LINE>` — Set default PostUp script line(s) for new peers. Can be specified multiple times for multiple script lines.
* `--default-peer-script-pre-down-enabled <DEFAULT_PEER_SCRIPT_PRE_DOWN_ENABLED>` — Enable PreDown script for new peers by default

  Possible values: `true`, `false`

* `--default-peer-script-pre-down-line <DEFAULT_PEER_SCRIPT_PRE_DOWN_LINE>` — Set default PreDown script line(s) for new peers. Can be specified multiple times for multiple script lines.
* `--default-peer-script-post-down-enabled <DEFAULT_PEER_SCRIPT_POST_DOWN_ENABLED>` — Enable PostDown script for new peers by default

  Possible values: `true`, `false`

* `--default-peer-script-post-down-line <DEFAULT_PEER_SCRIPT_POST_DOWN_LINE>` — Set default PostDown script line(s) for new peers. Can be specified multiple times for multiple script lines.
* `--default-connection-persistent-keepalive-enabled <DEFAULT_CONNECTION_PERSISTENT_KEEPALIVE_ENABLED>` — Enable PersistentKeepalive for new connections by default

  Possible values: `true`, `false`

* `--default-connection-persistent-keepalive-period <25>` — Set default PersistentKeepalive period in seconds
* `--no-prompt <NO_PROMPT>` — Disable interactive setup prompts

  Possible values: `true`, `false`




### `agent run`

Run the wg-quickrs agent

**Usage:** `agent run`




---

## `wg-quickrs config`


**Subcommand Overview:**

* [`config`↴](#config)
* [`config enable`↴](#config-enable)
* [`config enable agent`↴](#config-enable-agent)
* [`config enable agent web`↴](#config-enable-agent-web)
* [`config enable agent web http`↴](#config-enable-agent-web-http)
* [`config enable agent web https`↴](#config-enable-agent-web-https)
* [`config enable agent web password`↴](#config-enable-agent-web-password)
* [`config enable agent vpn`↴](#config-enable-agent-vpn)
* [`config enable network`↴](#config-enable-network)
* [`config enable network peer`↴](#config-enable-network-peer)
* [`config enable network peer endpoint`↴](#config-enable-network-peer-endpoint)
* [`config enable network peer icon`↴](#config-enable-network-peer-icon)
* [`config enable network peer dns`↴](#config-enable-network-peer-dns)
* [`config enable network peer mtu`↴](#config-enable-network-peer-mtu)
* [`config enable network connection`↴](#config-enable-network-connection)
* [`config enable network defaults`↴](#config-enable-network-defaults)
* [`config enable network defaults peer`↴](#config-enable-network-defaults-peer)
* [`config enable network defaults peer icon`↴](#config-enable-network-defaults-peer-icon)
* [`config enable network defaults peer dns`↴](#config-enable-network-defaults-peer-dns)
* [`config enable network defaults peer mtu`↴](#config-enable-network-defaults-peer-mtu)
* [`config enable network defaults connection`↴](#config-enable-network-defaults-connection)
* [`config enable network defaults connection persistent-keepalive`↴](#config-enable-network-defaults-connection-persistent-keepalive)
* [`config disable`↴](#config-disable)
* [`config disable agent`↴](#config-disable-agent)
* [`config disable agent web`↴](#config-disable-agent-web)
* [`config disable agent web http`↴](#config-disable-agent-web-http)
* [`config disable agent web https`↴](#config-disable-agent-web-https)
* [`config disable agent web password`↴](#config-disable-agent-web-password)
* [`config disable agent vpn`↴](#config-disable-agent-vpn)
* [`config disable network`↴](#config-disable-network)
* [`config disable network peer`↴](#config-disable-network-peer)
* [`config disable network peer endpoint`↴](#config-disable-network-peer-endpoint)
* [`config disable network peer icon`↴](#config-disable-network-peer-icon)
* [`config disable network peer dns`↴](#config-disable-network-peer-dns)
* [`config disable network peer mtu`↴](#config-disable-network-peer-mtu)
* [`config disable network connection`↴](#config-disable-network-connection)
* [`config disable network defaults`↴](#config-disable-network-defaults)
* [`config disable network defaults peer`↴](#config-disable-network-defaults-peer)
* [`config disable network defaults peer icon`↴](#config-disable-network-defaults-peer-icon)
* [`config disable network defaults peer dns`↴](#config-disable-network-defaults-peer-dns)
* [`config disable network defaults peer mtu`↴](#config-disable-network-defaults-peer-mtu)
* [`config disable network defaults connection`↴](#config-disable-network-defaults-connection)
* [`config disable network defaults connection persistent-keepalive`↴](#config-disable-network-defaults-connection-persistent-keepalive)
* [`config set`↴](#config-set)
* [`config set agent`↴](#config-set-agent)
* [`config set agent web`↴](#config-set-agent-web)
* [`config set agent web address`↴](#config-set-agent-web-address)
* [`config set agent web http`↴](#config-set-agent-web-http)
* [`config set agent web http port`↴](#config-set-agent-web-http-port)
* [`config set agent web https`↴](#config-set-agent-web-https)
* [`config set agent web https port`↴](#config-set-agent-web-https-port)
* [`config set agent web https tls-cert`↴](#config-set-agent-web-https-tls-cert)
* [`config set agent web https tls-key`↴](#config-set-agent-web-https-tls-key)
* [`config set agent vpn`↴](#config-set-agent-vpn)
* [`config set agent vpn port`↴](#config-set-agent-vpn-port)
* [`config set network`↴](#config-set-network)
* [`config set network name`↴](#config-set-network-name)
* [`config set network subnet`↴](#config-set-network-subnet)
* [`config set network peer`↴](#config-set-network-peer)
* [`config set network peer name`↴](#config-set-network-peer-name)
* [`config set network peer address`↴](#config-set-network-peer-address)
* [`config set network peer endpoint`↴](#config-set-network-peer-endpoint)
* [`config set network peer kind`↴](#config-set-network-peer-kind)
* [`config set network peer icon`↴](#config-set-network-peer-icon)
* [`config set network peer dns`↴](#config-set-network-peer-dns)
* [`config set network peer mtu`↴](#config-set-network-peer-mtu)
* [`config set network connection`↴](#config-set-network-connection)
* [`config set network connection allowed-ips-a-to-b`↴](#config-set-network-connection-allowed-ips-a-to-b)
* [`config set network connection allowed-ips-b-to-a`↴](#config-set-network-connection-allowed-ips-b-to-a)
* [`config set network connection persistent-keepalive`↴](#config-set-network-connection-persistent-keepalive)
* [`config set network defaults`↴](#config-set-network-defaults)
* [`config set network defaults peer`↴](#config-set-network-defaults-peer)
* [`config set network defaults peer kind`↴](#config-set-network-defaults-peer-kind)
* [`config set network defaults peer icon`↴](#config-set-network-defaults-peer-icon)
* [`config set network defaults peer dns`↴](#config-set-network-defaults-peer-dns)
* [`config set network defaults peer mtu`↴](#config-set-network-defaults-peer-mtu)
* [`config set network defaults connection`↴](#config-set-network-defaults-connection)
* [`config set network defaults connection persistent-keepalive`↴](#config-set-network-defaults-connection-persistent-keepalive)
* [`config reset`↴](#config-reset)
* [`config reset agent`↴](#config-reset-agent)
* [`config reset agent web`↴](#config-reset-agent-web)
* [`config reset agent web password`↴](#config-reset-agent-web-password)
* [`config reset network`↴](#config-reset-network)
* [`config reset network peer`↴](#config-reset-network-peer)
* [`config reset network peer private-key`↴](#config-reset-network-peer-private-key)
* [`config reset network connection`↴](#config-reset-network-connection)
* [`config reset network connection pre-shared-key`↴](#config-reset-network-connection-pre-shared-key)
* [`config get`↴](#config-get)
* [`config get agent`↴](#config-get-agent)
* [`config get agent web`↴](#config-get-agent-web)
* [`config get agent web address`↴](#config-get-agent-web-address)
* [`config get agent web http`↴](#config-get-agent-web-http)
* [`config get agent web http enabled`↴](#config-get-agent-web-http-enabled)
* [`config get agent web http port`↴](#config-get-agent-web-http-port)
* [`config get agent web https`↴](#config-get-agent-web-https)
* [`config get agent web https enabled`↴](#config-get-agent-web-https-enabled)
* [`config get agent web https port`↴](#config-get-agent-web-https-port)
* [`config get agent web https tls-cert`↴](#config-get-agent-web-https-tls-cert)
* [`config get agent web https tls-key`↴](#config-get-agent-web-https-tls-key)
* [`config get agent web password`↴](#config-get-agent-web-password)
* [`config get agent web password enabled`↴](#config-get-agent-web-password-enabled)
* [`config get agent web password hash`↴](#config-get-agent-web-password-hash)
* [`config get agent vpn`↴](#config-get-agent-vpn)
* [`config get agent vpn enabled`↴](#config-get-agent-vpn-enabled)
* [`config get agent vpn port`↴](#config-get-agent-vpn-port)
* [`config get network`↴](#config-get-network)
* [`config get network name`↴](#config-get-network-name)
* [`config get network subnet`↴](#config-get-network-subnet)
* [`config get network this-peer`↴](#config-get-network-this-peer)
* [`config get network peers`↴](#config-get-network-peers)
* [`config get network peers name`↴](#config-get-network-peers-name)
* [`config get network peers address`↴](#config-get-network-peers-address)
* [`config get network peers endpoint`↴](#config-get-network-peers-endpoint)
* [`config get network peers endpoint enabled`↴](#config-get-network-peers-endpoint-enabled)
* [`config get network peers endpoint address`↴](#config-get-network-peers-endpoint-address)
* [`config get network peers kind`↴](#config-get-network-peers-kind)
* [`config get network peers icon`↴](#config-get-network-peers-icon)
* [`config get network peers icon enabled`↴](#config-get-network-peers-icon-enabled)
* [`config get network peers icon src`↴](#config-get-network-peers-icon-src)
* [`config get network peers dns`↴](#config-get-network-peers-dns)
* [`config get network peers dns enabled`↴](#config-get-network-peers-dns-enabled)
* [`config get network peers dns addresses`↴](#config-get-network-peers-dns-addresses)
* [`config get network peers mtu`↴](#config-get-network-peers-mtu)
* [`config get network peers mtu enabled`↴](#config-get-network-peers-mtu-enabled)
* [`config get network peers mtu value`↴](#config-get-network-peers-mtu-value)
* [`config get network peers scripts`↴](#config-get-network-peers-scripts)
* [`config get network peers private-key`↴](#config-get-network-peers-private-key)
* [`config get network peers created-at`↴](#config-get-network-peers-created-at)
* [`config get network peers updated-at`↴](#config-get-network-peers-updated-at)
* [`config get network connections`↴](#config-get-network-connections)
* [`config get network connections enabled`↴](#config-get-network-connections-enabled)
* [`config get network connections pre-shared-key`↴](#config-get-network-connections-pre-shared-key)
* [`config get network connections persistent-keepalive`↴](#config-get-network-connections-persistent-keepalive)
* [`config get network connections persistent-keepalive enabled`↴](#config-get-network-connections-persistent-keepalive-enabled)
* [`config get network connections persistent-keepalive period`↴](#config-get-network-connections-persistent-keepalive-period)
* [`config get network connections allowed-ips-a-to-b`↴](#config-get-network-connections-allowed-ips-a-to-b)
* [`config get network connections allowed-ips-b-to-a`↴](#config-get-network-connections-allowed-ips-b-to-a)
* [`config get network defaults`↴](#config-get-network-defaults)
* [`config get network defaults peer`↴](#config-get-network-defaults-peer)
* [`config get network defaults peer kind`↴](#config-get-network-defaults-peer-kind)
* [`config get network defaults peer icon`↴](#config-get-network-defaults-peer-icon)
* [`config get network defaults peer icon enabled`↴](#config-get-network-defaults-peer-icon-enabled)
* [`config get network defaults peer icon src`↴](#config-get-network-defaults-peer-icon-src)
* [`config get network defaults peer dns`↴](#config-get-network-defaults-peer-dns)
* [`config get network defaults peer dns enabled`↴](#config-get-network-defaults-peer-dns-enabled)
* [`config get network defaults peer dns addresses`↴](#config-get-network-defaults-peer-dns-addresses)
* [`config get network defaults peer mtu`↴](#config-get-network-defaults-peer-mtu)
* [`config get network defaults peer mtu enabled`↴](#config-get-network-defaults-peer-mtu-enabled)
* [`config get network defaults peer mtu value`↴](#config-get-network-defaults-peer-mtu-value)
* [`config get network defaults peer scripts`↴](#config-get-network-defaults-peer-scripts)
* [`config get network defaults connection`↴](#config-get-network-defaults-connection)
* [`config get network defaults connection persistent-keepalive`↴](#config-get-network-defaults-connection-persistent-keepalive)
* [`config get network defaults connection persistent-keepalive enabled`↴](#config-get-network-defaults-connection-persistent-keepalive-enabled)
* [`config get network defaults connection persistent-keepalive period`↴](#config-get-network-defaults-connection-persistent-keepalive-period)
* [`config get network reservations`↴](#config-get-network-reservations)
* [`config get network reservations peer-id`↴](#config-get-network-reservations-peer-id)
* [`config get network reservations valid-until`↴](#config-get-network-reservations-valid-until)
* [`config get network updated-at`↴](#config-get-network-updated-at)
* [`config list`↴](#config-list)
* [`config list peers`↴](#config-list-peers)
* [`config list connections`↴](#config-list-connections)
* [`config list reservations`↴](#config-list-reservations)
* [`config remove`↴](#config-remove)
* [`config remove peer`↴](#config-remove-peer)
* [`config remove connection`↴](#config-remove-connection)
* [`config remove reservation`↴](#config-remove-reservation)
* [`config add`↴](#config-add)
* [`config add peer`↴](#config-add-peer)
* [`config add connection`↴](#config-add-connection)

### `config`

Edit agent configuration options

**Usage:** `config <COMMAND>`

###### **Subcommands:**

* `enable` — Enable a configuration option
* `disable` — Disable a configuration option
* `set` — Set a configuration value
* `reset` — Reset a configuration option
* `get` — Get a configuration value
* `list` — List network entities in human-readable format
* `remove` — Remove network entities
* `add` — Add network entities



### `config enable`

Enable a configuration option

**Usage:** `config enable <COMMAND>`

###### **Subcommands:**

* `agent` — Enable agent configuration options
* `network` — Enable network configuration options



### `config enable agent`

Enable agent configuration options

**Usage:** `config enable agent <COMMAND>`

###### **Subcommands:**

* `web` — Enable web server options
* `vpn` — Enable VPN server



### `config enable agent web`

Enable web server options

**Usage:** `config enable agent web <COMMAND>`

###### **Subcommands:**

* `http` — Enable HTTP on web server
* `https` — Enable HTTPS on web server
* `password` — Enable password authentication for web server



### `config enable agent web http`

Enable HTTP on web server

**Usage:** `config enable agent web http`



### `config enable agent web https`

Enable HTTPS on web server

**Usage:** `config enable agent web https`



### `config enable agent web password`

Enable password authentication for web server

**Usage:** `config enable agent web password`



### `config enable agent vpn`

Enable VPN server

**Usage:** `config enable agent vpn`



### `config enable network`

Enable network configuration options

**Usage:** `config enable network <COMMAND>`

###### **Subcommands:**

* `peer` — Enable peer options
* `connection` — Enable connection
* `defaults` — Enable default configuration options



### `config enable network peer`

Enable peer options

**Usage:** `config enable network peer <ID> <COMMAND>`

###### **Subcommands:**

* `endpoint` — Enable peer endpoint
* `icon` — Enable peer icon
* `dns` — Enable peer DNS
* `mtu` — Enable peer MTU

###### **Arguments:**

* `<ID>` — Peer UUID



### `config enable network peer endpoint`

Enable peer endpoint

**Usage:** `config enable network peer endpoint`



### `config enable network peer icon`

Enable peer icon

**Usage:** `config enable network peer icon`



### `config enable network peer dns`

Enable peer DNS

**Usage:** `config enable network peer dns`



### `config enable network peer mtu`

Enable peer MTU

**Usage:** `config enable network peer mtu`



### `config enable network connection`

Enable connection

**Usage:** `config enable network connection <ID>`

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config enable network defaults`

Enable default configuration options

**Usage:** `config enable network defaults <COMMAND>`

###### **Subcommands:**

* `peer` — Enable default peer options
* `connection` — Enable default connection options



### `config enable network defaults peer`

Enable default peer options

**Usage:** `config enable network defaults peer <COMMAND>`

###### **Subcommands:**

* `icon` — Enable default peer icon
* `dns` — Enable default peer DNS
* `mtu` — Enable default peer MTU



### `config enable network defaults peer icon`

Enable default peer icon

**Usage:** `config enable network defaults peer icon`



### `config enable network defaults peer dns`

Enable default peer DNS

**Usage:** `config enable network defaults peer dns`



### `config enable network defaults peer mtu`

Enable default peer MTU

**Usage:** `config enable network defaults peer mtu`



### `config enable network defaults connection`

Enable default connection options

**Usage:** `config enable network defaults connection <COMMAND>`

###### **Subcommands:**

* `persistent-keepalive` — Enable default connection persistent keepalive



### `config enable network defaults connection persistent-keepalive`

Enable default connection persistent keepalive

**Usage:** `config enable network defaults connection persistent-keepalive`



### `config disable`

Disable a configuration option

**Usage:** `config disable <COMMAND>`

###### **Subcommands:**

* `agent` — Disable agent configuration options
* `network` — Disable network configuration options



### `config disable agent`

Disable agent configuration options

**Usage:** `config disable agent <COMMAND>`

###### **Subcommands:**

* `web` — Disable web server options
* `vpn` — Disable VPN server



### `config disable agent web`

Disable web server options

**Usage:** `config disable agent web <COMMAND>`

###### **Subcommands:**

* `http` — Disable HTTP on web server
* `https` — Disable HTTPS on web server
* `password` — Disable password authentication for web server



### `config disable agent web http`

Disable HTTP on web server

**Usage:** `config disable agent web http`



### `config disable agent web https`

Disable HTTPS on web server

**Usage:** `config disable agent web https`



### `config disable agent web password`

Disable password authentication for web server

**Usage:** `config disable agent web password`



### `config disable agent vpn`

Disable VPN server

**Usage:** `config disable agent vpn`



### `config disable network`

Disable network configuration options

**Usage:** `config disable network <COMMAND>`

###### **Subcommands:**

* `peer` — Disable peer options
* `connection` — Disable connection
* `defaults` — Disable default configuration options



### `config disable network peer`

Disable peer options

**Usage:** `config disable network peer <ID> <COMMAND>`

###### **Subcommands:**

* `endpoint` — Disable peer endpoint
* `icon` — Disable peer icon
* `dns` — Disable peer DNS
* `mtu` — Disable peer MTU

###### **Arguments:**

* `<ID>` — Peer UUID



### `config disable network peer endpoint`

Disable peer endpoint

**Usage:** `config disable network peer endpoint`



### `config disable network peer icon`

Disable peer icon

**Usage:** `config disable network peer icon`



### `config disable network peer dns`

Disable peer DNS

**Usage:** `config disable network peer dns`



### `config disable network peer mtu`

Disable peer MTU

**Usage:** `config disable network peer mtu`



### `config disable network connection`

Disable connection

**Usage:** `config disable network connection <ID>`

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config disable network defaults`

Disable default configuration options

**Usage:** `config disable network defaults <COMMAND>`

###### **Subcommands:**

* `peer` — Disable default peer options
* `connection` — Disable default connection options



### `config disable network defaults peer`

Disable default peer options

**Usage:** `config disable network defaults peer <COMMAND>`

###### **Subcommands:**

* `icon` — Disable default peer icon
* `dns` — Disable default peer DNS
* `mtu` — Disable default peer MTU



### `config disable network defaults peer icon`

Disable default peer icon

**Usage:** `config disable network defaults peer icon`



### `config disable network defaults peer dns`

Disable default peer DNS

**Usage:** `config disable network defaults peer dns`



### `config disable network defaults peer mtu`

Disable default peer MTU

**Usage:** `config disable network defaults peer mtu`



### `config disable network defaults connection`

Disable default connection options

**Usage:** `config disable network defaults connection <COMMAND>`

###### **Subcommands:**

* `persistent-keepalive` — Disable default connection persistent keepalive



### `config disable network defaults connection persistent-keepalive`

Disable default connection persistent keepalive

**Usage:** `config disable network defaults connection persistent-keepalive`



### `config set`

Set a configuration value

**Usage:** `config set <COMMAND>`

###### **Subcommands:**

* `agent` — Set agent configuration values
* `network` — Set network configuration values



### `config set agent`

Set agent configuration values

**Usage:** `config set agent <COMMAND>`

###### **Subcommands:**

* `web` — Set web server configuration
* `vpn` — Set VPN configuration



### `config set agent web`

Set web server configuration

**Usage:** `config set agent web <COMMAND>`

###### **Subcommands:**

* `address` — Set agent web server bind IPv4 address
* `http` — Set HTTP configuration
* `https` — Set HTTPS configuration



### `config set agent web address`

Set agent web server bind IPv4 address

**Usage:** `config set agent web address <VALUE>`

###### **Arguments:**

* `<VALUE>` — IPv4 address



### `config set agent web http`

Set HTTP configuration

**Usage:** `config set agent web http <COMMAND>`

###### **Subcommands:**

* `port` — Set web server HTTP port



### `config set agent web http port`

Set web server HTTP port

**Usage:** `config set agent web http port <VALUE>`

###### **Arguments:**

* `<VALUE>` — Port number (0-65535)



### `config set agent web https`

Set HTTPS configuration

**Usage:** `config set agent web https <COMMAND>`

###### **Subcommands:**

* `port` — Set web server HTTPS port
* `tls-cert` — Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS
* `tls-key` — Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS



### `config set agent web https port`

Set web server HTTPS port

**Usage:** `config set agent web https port <VALUE>`

###### **Arguments:**

* `<VALUE>` — Port number (0-65535)



### `config set agent web https tls-cert`

Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS

**Usage:** `config set agent web https tls-cert <VALUE>`

###### **Arguments:**

* `<VALUE>` — File path



### `config set agent web https tls-key`

Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS

**Usage:** `config set agent web https tls-key <VALUE>`

###### **Arguments:**

* `<VALUE>` — File path



### `config set agent vpn`

Set VPN configuration

**Usage:** `config set agent vpn <COMMAND>`

###### **Subcommands:**

* `port` — Set VPN server listening port



### `config set agent vpn port`

Set VPN server listening port

**Usage:** `config set agent vpn port <VALUE>`

###### **Arguments:**

* `<VALUE>` — Port number (0-65535)



### `config set network`

Set network configuration values

**Usage:** `config set network <COMMAND>`

###### **Subcommands:**

* `name` — Set network name
* `subnet` — Set network subnet
* `peer` — Set peer configuration
* `connection` — Set connection configuration
* `defaults` — Set default configuration



### `config set network name`

Set network name

**Usage:** `config set network name <NAME>`

###### **Arguments:**

* `<NAME>` — New network name



### `config set network subnet`

Set network subnet

**Usage:** `config set network subnet <SUBNET>`

###### **Arguments:**

* `<SUBNET>` — New subnet (e.g., 10.0.0.0/24)



### `config set network peer`

Set peer configuration

**Usage:** `config set network peer <ID> <COMMAND>`

###### **Subcommands:**

* `name` — Set peer name
* `address` — Set peer address
* `endpoint` — Set peer endpoint address
* `kind` — Set peer kind
* `icon` — Set peer icon source
* `dns` — Set peer DNS addresses
* `mtu` — Set peer MTU value

###### **Arguments:**

* `<ID>` — Peer UUID



### `config set network peer name`

Set peer name

**Usage:** `config set network peer name <NAME>`

###### **Arguments:**

* `<NAME>` — New peer name



### `config set network peer address`

Set peer address

**Usage:** `config set network peer address <ADDRESS>`

###### **Arguments:**

* `<ADDRESS>` — New IPv4 address



### `config set network peer endpoint`

Set peer endpoint address

**Usage:** `config set network peer endpoint <ENDPOINT>`

###### **Arguments:**

* `<ENDPOINT>` — Endpoint address (hostname:port or ipv4:port)



### `config set network peer kind`

Set peer kind

**Usage:** `config set network peer kind <KIND>`

###### **Arguments:**

* `<KIND>` — Peer kind (e.g., laptop, server, phone)



### `config set network peer icon`

Set peer icon source

**Usage:** `config set network peer icon <SRC>`

###### **Arguments:**

* `<SRC>` — Icon source (URL or path)



### `config set network peer dns`

Set peer DNS addresses

**Usage:** `config set network peer dns <ADDRESSES>`

###### **Arguments:**

* `<ADDRESSES>` — Comma-separated list of IPv4 addresses (e.g., 8.8.8.8,8.8.4.4)



### `config set network peer mtu`

Set peer MTU value

**Usage:** `config set network peer mtu <VALUE>`

###### **Arguments:**

* `<VALUE>` — MTU value



### `config set network connection`

Set connection configuration

**Usage:** `config set network connection <ID> <COMMAND>`

###### **Subcommands:**

* `allowed-ips-a-to-b` — Set allowed IPs from peer A to peer B
* `allowed-ips-b-to-a` — Set allowed IPs from peer B to peer A
* `persistent-keepalive` — Set persistent keepalive period

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config set network connection allowed-ips-a-to-b`

Set allowed IPs from peer A to peer B

**Usage:** `config set network connection allowed-ips-a-to-b <IPS>`

###### **Arguments:**

* `<IPS>` — Comma-separated list of CIDR blocks (e.g., 0.0.0.0/0,10.0.0.0/8)



### `config set network connection allowed-ips-b-to-a`

Set allowed IPs from peer B to peer A

**Usage:** `config set network connection allowed-ips-b-to-a <IPS>`

###### **Arguments:**

* `<IPS>` — Comma-separated list of CIDR blocks



### `config set network connection persistent-keepalive`

Set persistent keepalive period

**Usage:** `config set network connection persistent-keepalive <PERIOD>`

###### **Arguments:**

* `<PERIOD>` — Keepalive period in seconds



### `config set network defaults`

Set default configuration

**Usage:** `config set network defaults <COMMAND>`

###### **Subcommands:**

* `peer` — Set default peer configuration
* `connection` — Set default connection configuration



### `config set network defaults peer`

Set default peer configuration

**Usage:** `config set network defaults peer <COMMAND>`

###### **Subcommands:**

* `kind` — Set default peer kind
* `icon` — Set default peer icon source
* `dns` — Set default peer DNS addresses
* `mtu` — Set default peer MTU value



### `config set network defaults peer kind`

Set default peer kind

**Usage:** `config set network defaults peer kind <KIND>`

###### **Arguments:**

* `<KIND>` — Peer kind (e.g., laptop, server, phone)



### `config set network defaults peer icon`

Set default peer icon source

**Usage:** `config set network defaults peer icon <SRC>`

###### **Arguments:**

* `<SRC>` — Icon source (URL or path)



### `config set network defaults peer dns`

Set default peer DNS addresses

**Usage:** `config set network defaults peer dns <ADDRESSES>`

###### **Arguments:**

* `<ADDRESSES>` — Comma-separated list of IPv4 addresses (e.g., 8.8.8.8,8.8.4.4)



### `config set network defaults peer mtu`

Set default peer MTU value

**Usage:** `config set network defaults peer mtu <VALUE>`

###### **Arguments:**

* `<VALUE>` — MTU value



### `config set network defaults connection`

Set default connection configuration

**Usage:** `config set network defaults connection <COMMAND>`

###### **Subcommands:**

* `persistent-keepalive` — Set default connection persistent keepalive period



### `config set network defaults connection persistent-keepalive`

Set default connection persistent keepalive period

**Usage:** `config set network defaults connection persistent-keepalive <PERIOD>`

###### **Arguments:**

* `<PERIOD>` — Keepalive period in seconds



### `config reset`

Reset a configuration option

**Usage:** `config reset <COMMAND>`

###### **Subcommands:**

* `agent` — Reset agent configuration options
* `network` — Reset network configuration options



### `config reset agent`

Reset agent configuration options

**Usage:** `config reset agent <COMMAND>`

###### **Subcommands:**

* `web` — Reset web server configuration



### `config reset agent web`

Reset web server configuration

**Usage:** `config reset agent web <COMMAND>`

###### **Subcommands:**

* `password` — Reset password for web server access



### `config reset agent web password`

Reset password for web server access

**Usage:** `config reset agent web password [OPTIONS]`

###### **Options:**

* `--password <PASSWORD>` — The use of this option is HIGHLY DISCOURAGED because the plaintext password might show up in the shell history! THIS IS HIGHLY INSECURE! Please set the password without the --password flag, and the script will prompt for the password.



### `config reset network`

Reset network configuration options

**Usage:** `config reset network <COMMAND>`

###### **Subcommands:**

* `peer` — Reset peer options
* `connection` — Reset connection options



### `config reset network peer`

Reset peer options

**Usage:** `config reset network peer <ID> <COMMAND>`

###### **Subcommands:**

* `private-key` — Reset peer private key (generates new WireGuard key)

###### **Arguments:**

* `<ID>` — Peer UUID



### `config reset network peer private-key`

Reset peer private key (generates new WireGuard key)

**Usage:** `config reset network peer private-key`



### `config reset network connection`

Reset connection options

**Usage:** `config reset network connection <ID> <COMMAND>`

###### **Subcommands:**

* `pre-shared-key` — Reset connection pre-shared key (generates new WireGuard key)

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config reset network connection pre-shared-key`

Reset connection pre-shared key (generates new WireGuard key)

**Usage:** `config reset network connection pre-shared-key`



### `config get`

Get a configuration value

**Usage:** `config get <COMMAND>`

###### **Subcommands:**

* `agent` — Get agent configuration values
* `network` — Get network configuration values



### `config get agent`

Get agent configuration values

**Usage:** `config get agent [COMMAND]`

###### **Subcommands:**

* `web` — Get web server configuration
* `vpn` — Get VPN configuration



### `config get agent web`

Get web server configuration

**Usage:** `config get agent web [COMMAND]`

###### **Subcommands:**

* `address` — Get agent web server bind IPv4 address
* `http` — Get HTTP configuration
* `https` — Get HTTPS configuration
* `password` — Get password authentication configuration



### `config get agent web address`

Get agent web server bind IPv4 address

**Usage:** `config get agent web address`



### `config get agent web http`

Get HTTP configuration

**Usage:** `config get agent web http [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether HTTP is enabled
* `port` — Get web server HTTP port



### `config get agent web http enabled`

Get whether HTTP is enabled

**Usage:** `config get agent web http enabled`



### `config get agent web http port`

Get web server HTTP port

**Usage:** `config get agent web http port`



### `config get agent web https`

Get HTTPS configuration

**Usage:** `config get agent web https [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether HTTPS is enabled
* `port` — Get web server HTTPS port
* `tls-cert` — Get path to TLS certificate file for HTTPS
* `tls-key` — Get path to TLS private key file for HTTPS



### `config get agent web https enabled`

Get whether HTTPS is enabled

**Usage:** `config get agent web https enabled`



### `config get agent web https port`

Get web server HTTPS port

**Usage:** `config get agent web https port`



### `config get agent web https tls-cert`

Get path to TLS certificate file for HTTPS

**Usage:** `config get agent web https tls-cert`



### `config get agent web https tls-key`

Get path to TLS private key file for HTTPS

**Usage:** `config get agent web https tls-key`



### `config get agent web password`

Get password authentication configuration

**Usage:** `config get agent web password [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether password authentication is enabled
* `hash` — Get password hash



### `config get agent web password enabled`

Get whether password authentication is enabled

**Usage:** `config get agent web password enabled`



### `config get agent web password hash`

Get password hash

**Usage:** `config get agent web password hash`



### `config get agent vpn`

Get VPN configuration

**Usage:** `config get agent vpn [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether VPN server is enabled
* `port` — Get VPN server listening port



### `config get agent vpn enabled`

Get whether VPN server is enabled

**Usage:** `config get agent vpn enabled`



### `config get agent vpn port`

Get VPN server listening port

**Usage:** `config get agent vpn port`



### `config get network`

Get network configuration values

**Usage:** `config get network [COMMAND]`

###### **Subcommands:**

* `name` — Get network name
* `subnet` — Get network subnet
* `this-peer` — Get this peer's UUID
* `peers` — Get network peers
* `connections` — Get network connections
* `defaults` — Get network defaults
* `reservations` — Get network reservations
* `updated-at` — Get network last updated timestamp



### `config get network name`

Get network name

**Usage:** `config get network name`



### `config get network subnet`

Get network subnet

**Usage:** `config get network subnet`



### `config get network this-peer`

Get this peer's UUID

**Usage:** `config get network this-peer`



### `config get network peers`

Get network peers

**Usage:** `config get network peers [ID] [COMMAND]`

###### **Subcommands:**

* `name` — Get peer name
* `address` — Get peer IP address
* `endpoint` — Get peer endpoint
* `kind` — Get peer kind
* `icon` — Get peer icon
* `dns` — Get peer DNS
* `mtu` — Get peer MTU
* `scripts` — Get peer scripts
* `private-key` — Get peer private key
* `created-at` — Get peer creation timestamp
* `updated-at` — Get peer last updated timestamp

###### **Arguments:**

* `<ID>` — Peer UUID



### `config get network peers name`

Get peer name

**Usage:** `config get network peers name`



### `config get network peers address`

Get peer IP address

**Usage:** `config get network peers address`



### `config get network peers endpoint`

Get peer endpoint

**Usage:** `config get network peers endpoint [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether endpoint is enabled
* `address` — Get endpoint address



### `config get network peers endpoint enabled`

Get whether endpoint is enabled

**Usage:** `config get network peers endpoint enabled`



### `config get network peers endpoint address`

Get endpoint address

**Usage:** `config get network peers endpoint address`



### `config get network peers kind`

Get peer kind

**Usage:** `config get network peers kind`



### `config get network peers icon`

Get peer icon

**Usage:** `config get network peers icon [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether icon is enabled
* `src` — Get icon source



### `config get network peers icon enabled`

Get whether icon is enabled

**Usage:** `config get network peers icon enabled`



### `config get network peers icon src`

Get icon source

**Usage:** `config get network peers icon src`



### `config get network peers dns`

Get peer DNS

**Usage:** `config get network peers dns [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether DNS is enabled
* `addresses` — Get DNS addresses



### `config get network peers dns enabled`

Get whether DNS is enabled

**Usage:** `config get network peers dns enabled`



### `config get network peers dns addresses`

Get DNS addresses

**Usage:** `config get network peers dns addresses`



### `config get network peers mtu`

Get peer MTU

**Usage:** `config get network peers mtu [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether MTU is enabled
* `value` — Get MTU value



### `config get network peers mtu enabled`

Get whether MTU is enabled

**Usage:** `config get network peers mtu enabled`



### `config get network peers mtu value`

Get MTU value

**Usage:** `config get network peers mtu value`



### `config get network peers scripts`

Get peer scripts

**Usage:** `config get network peers scripts`



### `config get network peers private-key`

Get peer private key

**Usage:** `config get network peers private-key`



### `config get network peers created-at`

Get peer creation timestamp

**Usage:** `config get network peers created-at`



### `config get network peers updated-at`

Get peer last updated timestamp

**Usage:** `config get network peers updated-at`



### `config get network connections`

Get network connections

**Usage:** `config get network connections [ID] [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether connection is enabled
* `pre-shared-key` — Get connection pre-shared key
* `persistent-keepalive` — Get connection persistent keepalive
* `allowed-ips-a-to-b` — Get allowed IPs from A to B
* `allowed-ips-b-to-a` — Get allowed IPs from B to A

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config get network connections enabled`

Get whether connection is enabled

**Usage:** `config get network connections enabled`



### `config get network connections pre-shared-key`

Get connection pre-shared key

**Usage:** `config get network connections pre-shared-key`



### `config get network connections persistent-keepalive`

Get connection persistent keepalive

**Usage:** `config get network connections persistent-keepalive [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether persistent keepalive is enabled
* `period` — Get persistent keepalive period



### `config get network connections persistent-keepalive enabled`

Get whether persistent keepalive is enabled

**Usage:** `config get network connections persistent-keepalive enabled`



### `config get network connections persistent-keepalive period`

Get persistent keepalive period

**Usage:** `config get network connections persistent-keepalive period`



### `config get network connections allowed-ips-a-to-b`

Get allowed IPs from A to B

**Usage:** `config get network connections allowed-ips-a-to-b`



### `config get network connections allowed-ips-b-to-a`

Get allowed IPs from B to A

**Usage:** `config get network connections allowed-ips-b-to-a`



### `config get network defaults`

Get network defaults

**Usage:** `config get network defaults [COMMAND]`

###### **Subcommands:**

* `peer` — Get default peer configuration
* `connection` — Get default connection configuration



### `config get network defaults peer`

Get default peer configuration

**Usage:** `config get network defaults peer [COMMAND]`

###### **Subcommands:**

* `kind` — Get default peer kind
* `icon` — Get default peer icon
* `dns` — Get default peer DNS
* `mtu` — Get default peer MTU
* `scripts` — Get default peer scripts



### `config get network defaults peer kind`

Get default peer kind

**Usage:** `config get network defaults peer kind`



### `config get network defaults peer icon`

Get default peer icon

**Usage:** `config get network defaults peer icon [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether icon is enabled
* `src` — Get icon source



### `config get network defaults peer icon enabled`

Get whether icon is enabled

**Usage:** `config get network defaults peer icon enabled`



### `config get network defaults peer icon src`

Get icon source

**Usage:** `config get network defaults peer icon src`



### `config get network defaults peer dns`

Get default peer DNS

**Usage:** `config get network defaults peer dns [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether DNS is enabled
* `addresses` — Get DNS addresses



### `config get network defaults peer dns enabled`

Get whether DNS is enabled

**Usage:** `config get network defaults peer dns enabled`



### `config get network defaults peer dns addresses`

Get DNS addresses

**Usage:** `config get network defaults peer dns addresses`



### `config get network defaults peer mtu`

Get default peer MTU

**Usage:** `config get network defaults peer mtu [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether MTU is enabled
* `value` — Get MTU value



### `config get network defaults peer mtu enabled`

Get whether MTU is enabled

**Usage:** `config get network defaults peer mtu enabled`



### `config get network defaults peer mtu value`

Get MTU value

**Usage:** `config get network defaults peer mtu value`



### `config get network defaults peer scripts`

Get default peer scripts

**Usage:** `config get network defaults peer scripts`



### `config get network defaults connection`

Get default connection configuration

**Usage:** `config get network defaults connection [COMMAND]`

###### **Subcommands:**

* `persistent-keepalive` — Get default connection persistent keepalive



### `config get network defaults connection persistent-keepalive`

Get default connection persistent keepalive

**Usage:** `config get network defaults connection persistent-keepalive [COMMAND]`

###### **Subcommands:**

* `enabled` — Get whether persistent keepalive is enabled
* `period` — Get persistent keepalive period



### `config get network defaults connection persistent-keepalive enabled`

Get whether persistent keepalive is enabled

**Usage:** `config get network defaults connection persistent-keepalive enabled`



### `config get network defaults connection persistent-keepalive period`

Get persistent keepalive period

**Usage:** `config get network defaults connection persistent-keepalive period`



### `config get network reservations`

Get network reservations

**Usage:** `config get network reservations [IP] [COMMAND]`

###### **Subcommands:**

* `peer-id` — Get reservation peer ID
* `valid-until` — Get reservation validity timestamp

###### **Arguments:**

* `<IP>` — IPv4 address



### `config get network reservations peer-id`

Get reservation peer ID

**Usage:** `config get network reservations peer-id`



### `config get network reservations valid-until`

Get reservation validity timestamp

**Usage:** `config get network reservations valid-until`



### `config get network updated-at`

Get network last updated timestamp

**Usage:** `config get network updated-at`



### `config list`

List network entities in human-readable format

**Usage:** `config list <COMMAND>`

###### **Subcommands:**

* `peers` — List all peers in human-readable format
* `connections` — List all connections in human-readable format
* `reservations` — List all reservations in human-readable format



### `config list peers`

List all peers in human-readable format

**Usage:** `config list peers`



### `config list connections`

List all connections in human-readable format

**Usage:** `config list connections`



### `config list reservations`

List all reservations in human-readable format

**Usage:** `config list reservations`



### `config remove`

Remove network entities

**Usage:** `config remove <COMMAND>`

###### **Subcommands:**

* `peer` — Remove a peer by UUID
* `connection` — Remove a connection by connection ID
* `reservation` — Remove a reservation by IPv4 address



### `config remove peer`

Remove a peer by UUID

**Usage:** `config remove peer <ID>`

###### **Arguments:**

* `<ID>` — Peer UUID to remove



### `config remove connection`

Remove a connection by connection ID

**Usage:** `config remove connection <ID>`

###### **Arguments:**

* `<ID>` — Connection ID (format: uuid*uuid)



### `config remove reservation`

Remove a reservation by IPv4 address

**Usage:** `config remove reservation <ADDRESS>`

###### **Arguments:**

* `<ADDRESS>` — IPv4 address of the reservation to remove



### `config add`

Add network entities

**Usage:** `config add <COMMAND>`

###### **Subcommands:**

* `peer` — Add a peer to the network
* `connection` — Add a connection between two peers



### `config add peer`

Add a peer to the network

**Usage:** `config add peer [OPTIONS]`

###### **Options:**

* `--no-prompt <NO_PROMPT>` — Skip all prompts and exit with error if required options are not provided

  Possible values: `true`, `false`

* `--name <NAME>` — Set peer name
* `--address <ADDRESS>` — Set peer IPv4 address
* `--endpoint-enabled <ENDPOINT_ENABLED>` — Enable endpoint

  Possible values: `true`, `false`

* `--endpoint-address <ENDPOINT_ADDRESS>` — Set peer endpoint (hostname:port or ipv4:port)
* `--kind <laptop>` — Set peer kind (e.g., laptop, server, phone)
* `--icon-enabled <ICON_ENABLED>` — Enable icon

  Possible values: `true`, `false`

* `--icon-src <ICON_SRC>` — Set peer icon source (URL or path)
* `--dns-enabled <DNS_ENABLED>` — Enable DNS

  Possible values: `true`, `false`

* `--dns-addresses <1.1.1.1>` — Set DNS address
* `--mtu-enabled <MTU_ENABLED>` — Enable MTU

  Possible values: `true`, `false`

* `--mtu-value <1420>` — Set MTU value
* `--script-pre-up-enabled <SCRIPT_PRE_UP_ENABLED>` — Enable PreUp script

  Possible values: `true`, `false`

* `--script-pre-up-line <SCRIPT_PRE_UP_LINE>` — Set PreUp script line(s). Can be specified multiple times for multiple script lines.
* `--script-post-up-enabled <SCRIPT_POST_UP_ENABLED>` — Enable PostUp script

  Possible values: `true`, `false`

* `--script-post-up-line <SCRIPT_POST_UP_LINE>` — Set PostUp script line(s). Can be specified multiple times for multiple script lines.
* `--script-pre-down-enabled <SCRIPT_PRE_DOWN_ENABLED>` — Enable PreDown script

  Possible values: `true`, `false`

* `--script-pre-down-line <SCRIPT_PRE_DOWN_LINE>` — Set PreDown script line(s). Can be specified multiple times for multiple script lines.
* `--script-post-down-enabled <SCRIPT_POST_DOWN_ENABLED>` — Enable PostDown script

  Possible values: `true`, `false`

* `--script-post-down-line <SCRIPT_POST_DOWN_LINE>` — Set PostDown script line(s). Can be specified multiple times for multiple script lines.



### `config add connection`

Add a connection between two peers

**Usage:** `config add connection [OPTIONS]`

###### **Options:**

* `--no-prompt <NO_PROMPT>` — Skip all prompts and exit with error if required options are not provided

  Possible values: `true`, `false`

* `--first-peer <FIRST_PEER>` — Set first peer UUID
* `--second-peer <SECOND_PEER>` — Set second peer UUID
* `--persistent-keepalive-enabled <PERSISTENT_KEEPALIVE_ENABLED>` — Enable persistent keepalive

  Possible values: `true`, `false`

* `--persistent-keepalive-period <25>` — Set persistent keepalive period in seconds
* `--allowed-ips-first-to-second <10.0.34.0/24>` — Set allowed IPs from the first peer to the second peer
* `--allowed-ips-second-to-first <10.0.34.0/24>` — Set allowed IPs from the second peer to the first peer



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
