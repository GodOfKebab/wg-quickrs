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

/// Macro for peer-specific toggle functions
macro_rules! impl_peer_toggle {
    ($enable_fn:ident, $disable_fn:ident, $($field:ident).+, $field_name:expr) => {
        pub fn $enable_fn(id: &Uuid) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
            peer.$($field).+.enabled = true;
            log::info!("Enabled peer {} {}", id, $field_name);
            conf::util::set_config(&mut config)?;
            Ok(())
        }

        pub fn $disable_fn(id: &Uuid) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
            peer.$($field).+.enabled = false;
            log::info!("Disabled peer {} {}", id, $field_name);
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

/// Macro for defaults-specific toggle functions
macro_rules! impl_defaults_toggle {
    ($enable_fn:ident, $disable_fn:ident, $($field:ident).+, $field_name:expr) => {
        pub fn $enable_fn() -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            config.network.defaults.$($field).+.enabled = true;
            log::info!("Enabled default {}", $field_name);
            conf::util::set_config(&mut config)?;
            Ok(())
        }

        pub fn $disable_fn() -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            config.network.defaults.$($field).+.enabled = false;
            log::info!("Disabled default {}", $field_name);
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

/// Macro for connection-specific toggle functions
macro_rules! impl_connection_toggle {
    ($enable_fn:ident, $disable_fn:ident) => {
        pub fn $enable_fn(id_str: &str) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            let conn_id = parse_connection_id(id_str)?;
            let connection = config.network.connections.get_mut(&conn_id)
                .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
            connection.enabled = true;
            log::info!("Enabled connection {}", id_str);
            conf::util::set_config(&mut config)?;
            Ok(())
        }

        pub fn $disable_fn(id_str: &str) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            let conn_id = parse_connection_id(id_str)?;
            let connection = config.network.connections.get_mut(&conn_id)
                .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
            connection.enabled = false;
            log::info!("Disabled connection {}", id_str);
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

// Peer toggles
impl_peer_toggle!(enable_peer_endpoint, disable_peer_endpoint, endpoint, "endpoint");
impl_peer_toggle!(enable_peer_icon, disable_peer_icon, icon, "icon");
impl_peer_toggle!(enable_peer_dns, disable_peer_dns, dns, "DNS");
impl_peer_toggle!(enable_peer_mtu, disable_peer_mtu, mtu, "MTU");

// Connection toggles
impl_connection_toggle!(enable_connection, disable_connection);

// Defaults peer toggles
impl_defaults_toggle!(enable_defaults_peer_endpoint, disable_defaults_peer_endpoint, peer.endpoint, "peer endpoint");
impl_defaults_toggle!(enable_defaults_peer_icon, disable_defaults_peer_icon, peer.icon, "peer icon");
impl_defaults_toggle!(enable_defaults_peer_dns, disable_defaults_peer_dns, peer.dns, "peer DNS");
impl_defaults_toggle!(enable_defaults_peer_mtu, disable_defaults_peer_mtu, peer.mtu, "peer MTU");

// Defaults connection toggles
impl_defaults_toggle!(enable_defaults_connection_persistent_keepalive, disable_defaults_connection_persistent_keepalive, connection.persistent_keepalive, "connection persistent keepalive");

