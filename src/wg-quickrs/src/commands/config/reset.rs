use std::io;
use std::io::Write;
use uuid::Uuid;
use crate::commands::config::{parse_connection_id, ConfigCommandError};
use crate::commands::helpers;
use crate::conf;

pub fn reset_web_password(reset_web_password_opts: &Option<String>) -> Result<(), ConfigCommandError> {
    // get the wireguard config a file path
    let mut config = conf::util::get_config()?;

    log::info!("Resetting the web password...");
    let password = match reset_web_password_opts {
        Some(pwd) => {
            log::warn!(
                "THIS IS HIGHLY INSECURE! Please set the password without the --password flag. The plaintext password could be visible in your shell history."
            );
            pwd.clone()
        }
        None => {
            print!("Enter your new password: ");
            io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

            let mut pwd = String::new();
            io::stdin().read_line(&mut pwd).map_err(|e| ConfigCommandError::ReadFailed(e))?;
            pwd.trim().to_string()
        }
    };
    let password_hash = helpers::calculate_password_hash(password.trim())?;

    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config)?;
    Ok(())
}


/// Reset peer private key (generates new WireGuard key)
pub fn reset_peer_private_key(id: &Uuid) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.private_key = wg_quickrs_lib::helpers::wg_generate_key();
    log::info!("Reset peer {} private key", id);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Reset connection pre-shared key (generates new WireGuard key)
pub fn reset_connection_pre_shared_key(id_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
    connection.pre_shared_key = wg_quickrs_lib::helpers::wg_generate_key();
    log::info!("Reset connection {} pre-shared key", id_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}
