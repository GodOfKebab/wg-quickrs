use chrono::naive::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;
use crate::types::misc::*;
use crate::types::network::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Summary {
    pub network: Network,
    pub telemetry: Option<Telemetry>,
    pub digest: String,
    pub status: WireGuardStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SummaryDigest {
    pub telemetry: Option<Telemetry>,
    pub digest: String,
    pub status: WireGuardStatus,
    pub timestamp: DateTime<Utc>,
}

impl From<&Summary> for SummaryDigest {
    fn from(summary: &Summary) -> Self {
        SummaryDigest {
            telemetry: summary.telemetry.clone(),
            digest: summary.digest.clone(),
            status: summary.status.clone(),
            timestamp: summary.timestamp,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Telemetry {
    pub max_len: u8,
    pub data: Vec<TelemetryData>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TelemetryData {
    pub datum: BTreeMap<ConnectionId, TelemetryDatum>,
    #[serde(with = "ts_milliseconds")]
    pub timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TelemetryDatum {
    pub latest_handshake_at: u64,
    pub transfer_a_to_b: u64,
    pub transfer_b_to_a: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangeSum {
    pub changed_fields: Option<ChangedFields>,
    pub added_peers: Option<BTreeMap<Uuid, AddedPeer>>,
    pub added_connections: Option<BTreeMap<ConnectionId, Connection>>,
    pub removed_peers: Option<Vec<Uuid>>,
    pub removed_connections: Option<Vec<ConnectionId>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangedFields {
    pub peers: Option<BTreeMap<Uuid, OptionalPeer>>,
    pub connections: Option<BTreeMap<ConnectionId, OptionalConnection>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalPeer {
    pub name: Option<String>,
    pub address: Option<Ipv4Addr>,
    pub endpoint: Option<Endpoint>,
    pub kind: Option<String>,
    pub icon: Option<Icon>,
    pub dns: Option<Dns>,
    pub mtu: Option<Mtu>,
    pub scripts: Option<OptionalScripts>,
    pub private_key: Option<WireGuardKey>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalScripts {
    pub pre_up: Option<Vec<Script>>,
    pub post_up: Option<Vec<Script>>,
    pub pre_down: Option<Vec<Script>>,
    pub post_down: Option<Vec<Script>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalConnection {
    pub enabled: Option<bool>,
    pub pre_shared_key: Option<WireGuardKey>,
    pub persistent_keepalive: Option<PersistentKeepalive>,
    pub allowed_ips_a_to_b: Option<AllowedIPs>,
    pub allowed_ips_b_to_a: Option<AllowedIPs>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AddedPeer {
    pub name: String,
    pub address: Ipv4Addr,
    pub endpoint: Endpoint,
    pub kind: String,
    pub icon: Icon,
    pub dns: Dns,
    pub mtu: Mtu,
    pub scripts: Scripts,
    pub private_key: WireGuardKey,
    pub amnezia_parameters: AmneziaPeerParameters,
}

impl From<&AddedPeer> for Peer {
    fn from(added_peer: &AddedPeer) -> Self {
        Peer {
            name: added_peer.name.clone(),
            address: added_peer.address,
            endpoint: added_peer.endpoint.clone(),
            kind: added_peer.kind.clone(),
            icon: added_peer.icon.clone(),
            dns: added_peer.dns.clone(),
            mtu: added_peer.mtu.clone(),
            scripts: added_peer.scripts.clone(),
            private_key: added_peer.private_key,
            amnezia_parameters: added_peer.amnezia_parameters.clone(),
            created_at: Utc::now(), // TODO: use time from arg
            updated_at: Utc::now(),
        }
    }
}

