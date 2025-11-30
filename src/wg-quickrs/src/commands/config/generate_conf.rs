use wg_quickrs_cli::config::conf::ConfOptions;
use crate::commands::config::ConfigCommandError;
use crate::conf;
use wg_quickrs_lib::helpers::get_peer_wg_config;
use std::fs;

pub fn generate_peer_conf(options: &ConfOptions) -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    // Generate the WireGuard configuration
    let wg_conf = get_peer_wg_config(&config.network, &options.peer_id, options.stripped)?;

    // Output to file or stdout
    if let Some(out_path) = &options.out {
        fs::write(out_path, wg_conf)
            .map_err(|e| ConfigCommandError::ReadFailed(e))?;
    } else {
        print!("{}", wg_conf);
    }

    Ok(())
}
