# Upgrading Guide

Versioning (vMAJOR.MINOR.PATCH) follows semantic rules for compatibility:
- PATCH
  - **Full backwards compatibility**
  - Bug fixes and UI/CLI improvements
  - No API or conf.yml schema changes
- MINOR
  - **Partial backwards compatibility**
  - API schema changes only
  - conf.yml remains compatible
- MAJOR
  - **No backwards compatibility**
  - conf.yml schema changes.
  - Old configs won’t work without manual/guided updates.

## Upgrading from v1.0.0 to v2.0.0

With the introduction of [AmneziaWG support](amneziawg.md) and custom firewall rules, the configuration file format has changed slightly.
Now, you can swap out which WireGuard backend you want to use, and configure custom firewall rules for the agent.
The automated firewall (for `iptables` and `pf`) setup will fill the firewall fields in `wg-quickrs agent init`.
In short, new fields of `conf.yml` are shown below. For more details, see [schema.md](schema.md).

```yaml
# ...
agent:
  # ...
  vpn:
    # ...
    wg: /usr/bin/wg
    wg_userspace:
      enabled: false
      binary: ''
  firewall:
    http:
      pre_up:
        - enabled: true
          script: iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT;
      post_down:
        - enabled: true
          script: iptables -D INPUT -p tcp --dport "$PORT" -j ACCEPT;
    https:
      pre_up:
        - enabled: true
          script: iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT;
      post_down:
        - enabled: true
          script: iptables -D INPUT -p tcp --dport "$PORT" -j ACCEPT;
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
        - enabled: true
          script: sysctl -w net.ipv4.ip_forward=1;
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
        - enabled: true
          script: sysctl -w net.ipv4.ip_forward=0;
network:
  # ...
  peers:
    # ...
    '2f8e5a24-0ef2-4ea7-9c1d-3fc61fcfc908':
      # ...
      amnezia_parameters:
        jc: 3
        jmin: 60
        jmax: 120
  defaults:
    # ...
    amnezia_parameters:
      jc: 30
      jmin: 60
      jmax: 120
  amnezia_parameters:
    enabled: false
    s1: 55
    s2: 155
    h1: 1584778214
    h2: 3054966602
    h3: 312806646
    h4: 2010469403
```

