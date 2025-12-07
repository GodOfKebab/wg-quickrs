# wg-quickrs schemas

To see schemas from earlier versions, see the schema.md file in that release's attachments.

## conf.yml: 2.x.x

```yaml
# version of the wg-quickrs binary used to generate this config
version: '2.0.0'
# wg-quickrs agent configuration (not sent over network)
agent:
  web:
    # bind address for the web server
    address: '127.0.0.1'
    http:
      # enable/disable HTTP server
      enabled: true
      # bind port for the HTTP server (valid range: 1-65535)
      port: 8080
    https:
      # enable/disable HTTPS server
      enabled: true
      # bind port for the HTTPS server (valid range: 1-65535)
      port: 8443
      # TLS certificate path (relative to config folder: /etc/wg-quickrs or ~/.wg-quickrs)
      tls_cert: certs/servers/127.0.0.1/cert.pem
      # TLS key path (relative to config folder: /etc/wg-quickrs or ~/.wg-quickrs)
      tls_key: certs/servers/127.0.0.1/key.pem
    password:
      # enable/disable password protection for the API
      enabled: true
      # password hash to protect the API (Argon2id PHC format, generate with: wg-quickrs agent init or wg-quickrs config reset password)
      hash: $argon2id$...
  vpn:
    # enable/disable VPN service (if false, it won't be possible to toggle later)
    enabled: false
    # port for the VPN service to listen on (valid range: 1-65535, WireGuard default: 51820)
    port: 51820
    # path to the wireguard-tools utility (wg/awg)
    wg: /usr/bin/wg
    wg_userspace:
      # enable to use userspace WireGuard implementation, disable for kernel module (Linux only)
      enabled: true
      # path to the wireguard-go/amneziawg-go utility
      binary: /usr/bin/wireguard-go
  firewall:
    # firewall scripts for http server
    # Every script gets a PORT variable prepended (PORT=agent.web.http.port)
    http:
      pre_up:
        - enabled: true
          script: iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT;
      post_down:
        - enabled: true
          script: iptables -D INPUT -p tcp --dport "$PORT" -j ACCEPT;
    # firewall scripts for https server
    # Every script gets a PORT variable prepended (PORT=agent.web.https.port)
    https:
      pre_up:
        - enabled: true
          script: iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT;
      post_down:
        - enabled: true
          script: iptables -D INPUT -p tcp --dport "$PORT" -j ACCEPT;
    # firewall scripts for wireguard
    # Every script gets a WG_SUBNET variable prepended (WG_SUBNET=network.subnet)
    # Every script gets a WG_PORT variable prepended (WG_PORT=agent.vpn.port)
    # Every script gets a WG_INTERFACE variable prepended (WG_INTERFACE=network.name for linux, utunX(whatever the utun prefixed interface ends up being created) for macOS)
    vpn:
      pre_up: []
      post_up:
        - enabled: true
          script: iptables -t nat -I POSTROUTING -s "$WG_SUBNET" -o "eth0" -j MASQUERADE;
        - enabled: true
          script: iptables -I INPUT -p udp -m udp --dport "$WG_PORT" -j ACCEPT;
        - enabled: true
          script: iptables -I FORWARD -i "$WG_INTERFACE" -j ACCEPT;
        - enabled: true
          script: iptables -I FORWARD -o "$WG_INTERFACE" -j ACCEPT;
      pre_down: []
      post_down:
        - enabled: true
          script: iptables -t nat -D POSTROUTING -s "$WG_SUBNET" -o "eth0" -j MASQUERADE;
        - enabled: true
          script: iptables -D INPUT -p udp -m udp --dport "$WG_PORT" -j ACCEPT;
        - enabled: true
          script: iptables -D FORWARD -i "$WG_INTERFACE" -j ACCEPT;
        - enabled: true
          script: iptables -D FORWARD -o "$WG_INTERFACE" -j ACCEPT;
# wg-quickrs network configuration (sent over network)
network:
  name: wg-quickrs-home
  # CIDR for the network (must be valid IPv4 network in CIDR notation)
  subnet: '10.0.34.0/24'
  # id of the peer that the wg-quickrs agent will assume (UUID v4 format)
  this_peer: f923f1f6-0aea-4806-86a4-e88a8d336298
  peers:
    f923f1f6-0aea-4806-86a4-e88a8d336298:
      name: wg-quickrs-host
      # internal IPv4 address of the peer (must be within the network subnet)
      address: '10.0.34.1'
      endpoint:
        # enable/disable endpoint (if false, other peers can't discover this peer)
        enabled: true
        # IPv4-based endpoint to advertise to other peers (use for static IPs)
        address: !ipv4_and_port
          ipv4: '172.31.31.130'
          port: 51820
        # hostname-based endpoint to advertise to other peers (use for dynamic DNS)
#       address: !hostname_and_port
#         hostname: 'example.com'
#         port: 51820
        # no endpoint (for peers without public endpoints)
#       address: none
      # peer kind (values: server, desktop, laptop, tablet, phone, IoT, other - only used for UI icon selection)
      kind: server
      # if a custom icon is enabled, kind is ignored
      icon:
        # enable/disable custom icon
        enabled: false
        # custom icon source URL
        src: ''
      dns:
        # enable/disable DNS server(s)
        enabled: true
        # List of DNS servers
        addresses:
          - '1.1.1.1'
      mtu:
        # enable/disable MTU setting
        enabled: false
        # MTU value (WireGuard default: 1420, typical range: 1280-1500, adjust for lower MTU networks or nested VPNs)
        value: 1420
      # scripts to run before and after the peer is up/down (run with agent's permissions)
      scripts:
        # list of pre_up scripts to run before the interface is brought up
        pre_up:
          - enabled: true
            script: echo 'hi';
        # list of post_up scripts to run after the interface is up
        post_up: []
        # list of pre_down scripts to run before the interface goes down
        pre_down: []
        # list of post_down scripts to run after the interface is down
        post_down: []
      # private key for the peer (base64-encoded 32-byte WireGuard key)
      private_key: KU...=
      # peer-level parameters for the Amnezia VPN client (https://github.com/amnezia-vpn/amneziawg-linux-kernel-module?tab=readme-ov-file#configuration)
      amnezia_parameters:
        jc: 30
        jmin: 60
        jmax: 120
      # timestamps in RFC3339 format (YYYY-MM-DDTHH:MM:SS.SSSSSSSZ)
      created_at: '2025-11-18T00:38:38.388252Z'
      updated_at: '2025-11-18T00:38:38.388252Z'
    da2a9c32-410d-4d4d-8c9b-7dffdeeb8b9c:
      name: demo
      address: '10.0.34.2'
      endpoint:
        enabled: false
        address: none
      kind: laptop
      icon:
        enabled: false
        src: ''
      dns:
        enabled: true
        addresses:
        - '1.1.1.1'
      mtu:
        enabled: false
        value: 1420
      scripts:
        pre_up: []
        post_up: []
        pre_down: []
        post_down: []
      private_key: /9...=
      amnezia_parameters:
        jc: 30
        jmin: 60
        jmax: 120
      created_at: '2025-11-18T00:40:16.391330Z'
      updated_at: '2025-11-18T00:40:16.391330Z'
  connections:
    # connection id format: peer_a_id*peer_b_id (UUIDs are ordered lexicographically to ensure uniqueness)
    f923f1f6-0aea-4806-86a4-e88a8d336298*da2a9c32-410d-4d4d-8c9b-7dffdeeb8b9c:
      # enable/disable the connection
      enabled: true
      # pre-shared key for the connection (base64-encoded 32-byte WireGuard key for post-quantum security)
      pre_shared_key: xo...=
      persistent_keepalive:
        # enable/disable persistent keepalive (useful for NAT traversal)
        enabled: true
        # keepalive period in seconds (valid range: 1-65535, typical: 25)
        period: 25
      # list of allowed IPs for peer_a (wg-quickrs-host) to peer_b (demo)
      # common patterns: x.x.x.x/32 (single peer), x.x.x.0/24 (subnet), 0.0.0.0/0 (all traffic/full tunnel)
      allowed_ips_a_to_b:
      - '10.0.34.2/32'
      # list of allowed IPs for peer_b (demo) to peer_a (wg-quickrs-host)
      allowed_ips_b_to_a:
      - '0.0.0.0/0'
  # default values for new peers and connections
  defaults:
    peer:
      # default new peer kind
      kind: laptop
      # default new peer icon
      icon:
        enabled: false
        src: ''
      # default new peer DNS servers
      dns:
        enabled: true
        addresses:
        - '1.1.1.1'
      # default new peer MTU
      mtu:
        enabled: false
        value: 1420
      # default new peer scripts
      scripts:
        pre_up: []
        post_up: []
        pre_down: []
        post_down: []
      # default new peer-level parameters for the Amnezia VPN client (https://github.com/amnezia-vpn/amneziawg-linux-kernel-module?tab=readme-ov-file#configuration)
      amnezia_parameters:
        jc: 30
        jmin: 60
        jmax: 120
    connection:
      # default new connection persistent keepalive
      persistent_keepalive:
        enabled: true
        period: 25
  # peer reservations (when add a peer button is pressed, the browser reserves an address for 10 minutes to prevent conflicts)
  reservations:
    '10.0.34.3':
      peer_id: f857bbe1-0063-4dff-98da-78b47efd6453
      # reservation expiry in RFC3339 format
      valid_until: '2025-11-18T00:50:10.911311Z'
  # network-level parameters for the Amnezia VPN client (https://github.com/amnezia-vpn/amneziawg-linux-kernel-module?tab=readme-ov-file#configuration)
  amnezia_parameters:
    enabled: true
    s1: 55
    s2: 155
    h1: 1965538070
    h2: 1556073336
    h3: 1369216251
    h4: 4226881876
  # network last updated timestamp in RFC3339 format
  updated_at: '2025-11-18T00:40:10.911311Z'
```

## API: 2.0.x

If password is enabled, all API endpoints except `/api/token` require authentication via JWT bearer token.

### Authentication

#### `POST /api/token`

Authenticate and obtain a JWT token for API access.
For now, anything can be passed to the `client_id` field --I was mostly using it for testing purposes.

**Request:**
```json
{
  "client_id": "string",
  "password": "string"
}
```

**Response:** `200 OK`
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```
Returns a JWT token (plain text, not JSON). Token expires after 1 hour.

**Usage:**
Include the token in subsequent requests via the `Authorization` header:
```
Authorization: Bearer <token>
```

**Error Responses:**
- `204 No Content` - Token authentication not enabled
- `400 Bad Request` - Invalid JSON
- `401 Unauthorized` - Invalid credentials
- `500 Internal Server Error` - Configuration error or token creation failed

---

### System Information

#### `GET /api/version`

Get version and build information.

**Response:** `200 OK`
```json
{
  "version": "2.0.0",
  "build": {
    "branch": "main",
    "commit": "ea0e...",
    "timestamp": "2025-01-15T12:34:56Z"
  }
}
```

**Error Responses:**
- `401 Unauthorized` - Missing/invalid authorization token (if password enabled)
- `500 Internal Server Error` - Unable to get config

---

### Network Management

#### `GET /api/network/summary?only_digest=<bool>`

Get a network configuration summary with telemetry data.

**Required Query Parameters:**
- `only_digest` (boolean): If `true`, returns only digest and status without a full network config

**Response (only_digest=false):** `200 OK`
```json
{
  "network": { /* Full Network object (see conf.yml schema) */ },
  "telemetry": {
    "max_len": 100,
    "data": [
      {
        "datum": {
          "peer_a_id*peer_b_id": {
            "latest_handshake_at": 1234567890,
            "transfer_a_to_b": 1048576,
            "transfer_b_to_a": 2097152
          }
        },
        "timestamp": 1234567890000
      }
    ]
  },
  "digest": "base64-encoded-sha256-hash",
  "status": "up",
  "timestamp": "2025-01-15T12:34:56.123456Z"
}
```

**Response (only_digest=true):** `200 OK`
```json
{
  "telemetry": { /* same as above */ },
  "digest": "base64-encoded-sha256-hash",
  "status": "down",
  "timestamp": "2025-01-15T12:34:56.123456Z"
}
```

**Status values:**
- `"unknown"` - WireGuard status cannot be determined/not managed by wg-quickrs
- `"down"` - WireGuard tunnel is down
- `"up"` - WireGuard tunnel is running

**Error Responses:**
- `401 Unauthorized` - Missing/invalid authorization token (if password enabled)
- `500 Internal Server Error` - Unable to get summary or config

---

#### `PATCH /api/network/config`

Update network configuration with partial changes.

**Request:**
```json
{
  "changed_fields": {
    "peers": {
      "peer-uuid": {
        "name": "new-name",
        "endpoint": { /* Endpoint object */ }
      }
    },
    "connections": {
      "peer-a-id*peer-b-id": {
        "enabled": true
      }
    }
  },
  "added_peers": {
    "new-peer-uuid": {
      "name": "string",
      "address": "10.0.34.x",
      "endpoint": { /* Endpoint object */ },
      "kind": "string",
      "icon": { /* Icon object */ },
      "dns": { /* Dns object */ },
      "mtu": { /* Mtu object */ },
      "scripts": { /* Scripts object */ },
      "private_key": "base64-encoded-key"
    }
  },
  "added_connections": {
    "peer-a-id*peer-b-id": { /* Connection object */ }
  },
  "removed_peers": ["peer-uuid-1", "peer-uuid-2"],
  "removed_connections": ["peer-a-id*peer-b-id"]
}
```

All fields are optional. Only include fields you want to change/add/remove.

**Response:** `200 OK`
```json
/* Echoes back change_sum request */
```

**Error Responses:**
- `400 Bad Request` - Invalid JSON or validation error for specific field (e.g., "changed_fields.peers.{uuid}.name: {error}")
- `401 Unauthorized` - Missing/invalid authorization token (if password enabled)
- `403 Forbidden` - Cannot modify scripts for this peer remotely, peer already exists, or address reserved for another peer
- `404 Not Found` - Peer or connection does not exist
- `500 Internal Server Error` - Config lock error, serialization error, or unable to write config

---

#### `POST /api/network/reserve/address`

Reserve an available IP address for a new peer (reservation valid for 10 minutes).

**Request:** Empty body

**Response:** `200 OK`
```json
{
  "address": "10.0.34.x",
  "peer_id": "uuid-v4",
  "valid_until": "2025-01-15T12:44:56.123456Z"
}
```

**Error Responses:**
- `401 Unauthorized` - Missing/invalid authorization token (if password enabled)
- `409 Conflict` - No more IP addresses available in the pool
- `500 Internal Server Error` - Config lock error, serialization error, or unable to write config

---

### WireGuard Control

#### `POST /api/wireguard/status`

Enable or disable the WireGuard tunnel.

**Request:**
```json
{
  "status": "up"
}
```

**Status values:**
- `"up"` - Start the WireGuard tunnel
- `"down"` - Stop the WireGuard tunnel

**Response:** `200 OK`
```json
{
  "status": "up"
}
```

**Error Responses:**
- `400 Bad Request` - Invalid JSON or invalid status value
- `401 Unauthorized` - Missing/invalid authorization token (if password enabled)
- `403 Forbidden` - VPN is disabled in configuration (agent.vpn.enabled: false)
- `500 Internal Server Error` - Failed to get config or check WireGuard status

