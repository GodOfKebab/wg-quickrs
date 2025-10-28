use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use ipnet::Ipv4Net;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub agent: Agent,
    pub network: Network,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Agent {
    pub web: AgentWeb,
    pub vpn: AgentVpn,
    pub firewall: AgentFirewall,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWeb {
    pub address: Ipv4Addr,
    pub http: AgentWebHttp,
    pub https: AgentWebHttps,
    pub password: Password,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWebHttp {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWebHttps {
    pub enabled: bool,
    pub port: u16,
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Password {
    pub enabled: bool,
    pub hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentVpn {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentFirewall {
    pub enabled: bool,
    pub utility: PathBuf,
    pub gateway: String,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ConnectionId {
    pub a: Uuid,
    pub b: Uuid
}

impl ConnectionId {
    pub fn contains(&self, id: &Uuid) -> bool {
        &self.a == id || &self.b == id
    }
}

impl Hash for ConnectionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
    }
}

impl Serialize for ConnectionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}*{}", self.a, self.b))
    }
}

impl<'de> Deserialize<'de> for ConnectionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('*').collect();

        if parts.len() != 2 {
            return Err(serde::de::Error::custom("expected format 'uuid*uuid'"));
        }

        let a = Uuid::parse_str(parts[0])
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid 'a': {}", e)))?;
        let b = Uuid::parse_str(parts[1])
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid 'b': {}", e)))?;

        Ok(ConnectionId { a, b })
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Network {
    pub identifier: String,
    pub subnet: Ipv4Net,
    pub this_peer: Uuid,
    pub peers: HashMap<Uuid, Peer>,
    pub connections: HashMap<ConnectionId, Connection>,
    pub defaults: Defaults,
    pub reservations: HashMap<Ipv4Addr, ReservationData>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub address: Ipv4Addr,
    pub endpoint: Endpoint,
    pub kind: String,
    pub icon: Icon,
    pub dns: Dns,
    pub mtu: Mtu,
    pub scripts: Scripts,
    pub private_key: WireGuardKey,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct Endpoint {
    pub enabled: bool,
    pub address: EndpointAddress,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EndpointAddress {
    None,
    Ipv4(Ipv4Addr),
    Hostname(String),
}

impl Default for EndpointAddress {
    fn default() -> Self {
        EndpointAddress::None
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Icon {
    pub enabled: bool,
    pub src: String,
}


#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Dns {
    pub enabled: bool,
    pub addresses: Vec<Ipv4Addr>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Mtu {
    pub enabled: bool,
    pub value: u16,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Script {
    pub enabled: bool,
    pub script: String,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Scripts {
    pub pre_up: Vec<Script>,
    pub post_up: Vec<Script>,
    pub pre_down: Vec<Script>,
    pub post_down: Vec<Script>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WireGuardKey(pub [u8; 32]);

impl WireGuardKey {
    pub fn from_base64(s: &str) -> Result<Self, String> {
        let bytes = STANDARD.decode(s)
            .map_err(|e| format!("invalid base64: {}", e))?;
        bytes.try_into()
            .map_err(|_| "key must be exactly 32 bytes".to_string())
            .map(WireGuardKey)
    }

    pub fn to_base64(&self) -> String {
        STANDARD.encode(self.0)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for WireGuardKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl Serialize for WireGuardKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

impl<'de> Deserialize<'de> for WireGuardKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WireGuardKey::from_base64(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    pub enabled: bool,
    pub pre_shared_key: WireGuardKey,
    pub allowed_ips_a_to_b: AllowedIPs,
    pub allowed_ips_b_to_a: AllowedIPs,
    pub persistent_keepalive: PersistentKeepalive,
}

pub type AllowedIPs = Vec<Ipv4Net>;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct PersistentKeepalive {
    pub enabled: bool,
    pub period: u16,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Defaults {
    pub peer: DefaultPeer,
    pub connection: DefaultConnection,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct DefaultPeer {
    pub endpoint: Endpoint,
    pub kind: String,
    pub icon: Icon,
    pub dns: Dns,
    pub mtu: Mtu,
    pub scripts: Scripts,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct DefaultConnection {
    pub persistent_keepalive: PersistentKeepalive,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ReservationData {
    pub peer_id: Uuid,
    pub valid_until: DateTime<Utc>,
}

