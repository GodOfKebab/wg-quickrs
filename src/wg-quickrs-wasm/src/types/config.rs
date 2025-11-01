use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use crate::macros::*;
use crate::types::misc::WireGuardLibError;
use crate::types::network::{Network, NetworkWDigest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub version: String,
    pub agent: Agent,
    pub network: Network,
}

impl From<&ConfigFile> for Config {
    fn from(file_config: &ConfigFile) -> Self {
        Config {
            agent: file_config.agent.clone(),
            network: file_config.network.clone(),
        }
    }
}


impl From<&Config> for ConfigFile {
    fn from(config: &Config) -> Self {
        ConfigFile {
            version: wg_quickrs_version!().into(),
            agent: config.agent.clone(),
            network: config.network.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigWNetworkDigest {
    pub agent: Agent,
    pub network_w_digest: NetworkWDigest,
}

impl ConfigWNetworkDigest {
    pub fn from_config(config: Config) -> Result<Self, WireGuardLibError> {
        let network_w_digest = NetworkWDigest::try_from(config.network)?;
        Ok(ConfigWNetworkDigest { agent: config.agent, network_w_digest })
    }

    pub fn to_config(&self) -> Config {
        Config{ agent: self.agent.clone(), network: self.network_w_digest.network.clone() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub agent: Agent,
    pub network: Network,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Agent {
    pub web: AgentWeb,
    pub vpn: AgentVpn,
    pub firewall: AgentFirewall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentWeb {
    pub address: Ipv4Addr,
    pub http: AgentWebHttp,
    pub https: AgentWebHttps,
    pub password: Password,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentWebHttp {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentWebHttps {
    pub enabled: bool,
    pub port: u16,
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Password {
    pub enabled: bool,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentVpn {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentFirewall {
    pub enabled: bool,
    pub utility: PathBuf,
    pub gateway: String,
}

