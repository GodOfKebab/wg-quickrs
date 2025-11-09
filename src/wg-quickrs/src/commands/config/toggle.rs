use argon2::PasswordHash;
use uuid::Uuid;
use wg_quickrs_lib::validation::agent::{validate_fw_utility, validate_tls_file};
use crate::conf;
use crate::commands::config::{parse_connection_id, ConfigCommandError};
use crate::WG_QUICKRS_CONFIG_FOLDER;

/// Macro for implementing toggle (enable/disable) functions
macro_rules! impl_toggle {
    // Without validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // With validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr, validate: $validator:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            if status {
                $validator(&config)?;
            }
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

impl_toggle!(
    toggle_agent_web_http,
    agent.web.http =>
    |c: &wg_quickrs_lib::types::config::Config| format!("HTTP web server (port={})...", c.agent.web.http.port)
);

impl_toggle!(
    toggle_agent_web_https,
    agent.web.https =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "HTTPS web server (port={}, tls_cert={}, tls_key={})...",
        c.agent.web.https.port,
        c.agent.web.https.tls_cert.display(),
        c.agent.web.https.tls_key.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_cert)?;
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_key)?;
        Ok(())
    }
);

impl_toggle!(
    toggle_agent_web_password,
    agent.web.password =>
    |_: &wg_quickrs_lib::types::config::Config| "password for the web server...".to_string(),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        PasswordHash::new(&c.agent.web.password.hash)?;
        Ok(())
    }
);

impl_toggle!(
    toggle_agent_vpn,
    agent.vpn =>
    |c: &wg_quickrs_lib::types::config::Config| format!("VPN server (port={})...", c.agent.vpn.port)
);

impl_toggle!(
    toggle_agent_firewall,
    agent.firewall =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "firewall setting up NAT and input rules (utility={})...",
        c.agent.firewall.utility.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        if c.agent.firewall.gateway.is_empty() {
            return Err(ConfigCommandError::GatewayNotSet());
        }
        validate_fw_utility(&c.agent.firewall.utility)?;
        Ok(())
    }
);

/// Enable peer endpoint
pub fn enable_peer_endpoint(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.endpoint.enabled = true;
    log::info!("Enabled peer {} endpoint", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable peer endpoint
pub fn disable_peer_endpoint(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.endpoint.enabled = false;
    log::info!("Disabled peer {} endpoint", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable peer icon
pub fn enable_peer_icon(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.icon.enabled = true;
    log::info!("Enabled peer {} icon", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable peer icon
pub fn disable_peer_icon(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.icon.enabled = false;
    log::info!("Disabled peer {} icon", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable peer DNS
pub fn enable_peer_dns(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.dns.enabled = true;
    log::info!("Enabled peer {} DNS", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable peer DNS
pub fn disable_peer_dns(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.dns.enabled = false;
    log::info!("Disabled peer {} DNS", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable peer MTU
pub fn enable_peer_mtu(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.mtu.enabled = true;
    log::info!("Enabled peer {} MTU", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable peer MTU
pub fn disable_peer_mtu(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.mtu.enabled = false;
    log::info!("Disabled peer {} MTU", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable connection
pub fn enable_connection(id_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
    connection.enabled = true;
    log::info!("Enabled connection {}", id_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable connection
pub fn disable_connection(id_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
    connection.enabled = false;
    log::info!("Disabled connection {}", id_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable default peer endpoint
pub fn enable_defaults_peer_endpoint() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.endpoint.enabled = true;
    log::info!("Enabled default peer endpoint");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable default peer endpoint
pub fn disable_defaults_peer_endpoint() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.endpoint.enabled = false;
    log::info!("Disabled default peer endpoint");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable default peer icon
pub fn enable_defaults_peer_icon() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.icon.enabled = true;
    log::info!("Enabled default peer icon");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable default peer icon
pub fn disable_defaults_peer_icon() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.icon.enabled = false;
    log::info!("Disabled default peer icon");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable default peer DNS
pub fn enable_defaults_peer_dns() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.dns.enabled = true;
    log::info!("Enabled default peer DNS");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable default peer DNS
pub fn disable_defaults_peer_dns() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.dns.enabled = false;
    log::info!("Disabled default peer DNS");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable default peer MTU
pub fn enable_defaults_peer_mtu() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.mtu.enabled = true;
    log::info!("Enabled default peer MTU");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable default peer MTU
pub fn disable_defaults_peer_mtu() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.mtu.enabled = false;
    log::info!("Disabled default peer MTU");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Enable default connection persistent keepalive
pub fn enable_defaults_connection_persistent_keepalive() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.connection.persistent_keepalive.enabled = true;
    log::info!("Enabled default connection persistent keepalive");
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Disable default connection persistent keepalive
pub fn disable_defaults_connection_persistent_keepalive() -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.connection.persistent_keepalive.enabled = false;
    log::info!("Disabled default connection persistent keepalive");
    conf::util::set_config(&mut config)?;
    Ok(())
}

