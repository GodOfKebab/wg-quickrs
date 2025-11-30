use std::net::{IpAddr, SocketAddr};
use crate::WG_QUICKRS_CONFIG_FOLDER;
use crate::web::api;
use crate::web::app;
use crate::helpers::shell_cmd;
use crate::wireguard::wg_quick::HookType;
#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use wg_quickrs_lib::types::config::Config;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::path::PathBuf;
use thiserror::Error;
use tokio::try_join;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("failed to configure tls for https: {0}")]
    TlsSetupFailed(String),
}

fn execute_script(script: &str, port: u16, hook_type: HookType) {
    log::debug!("[#] Executing http(s) {:?} hooks", hook_type);
    let script_w_vars = format!("PORT={port}\n{script}");
    match shell_cmd(&["sh", "-c", &script_w_vars]) {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Warning: http(s) firewall script failed: {}", stderr);
            }
        }
        Err(e) => {
            log::warn!("Warning: http(s) firewall script failed: {}", e);
        }
    }
}

pub(crate) async fn run_web_server(config: &Config) -> std::io::Result<()> {
    // Closure building your Actix app, reused for both HTTP and HTTPS
    let app_factory = || {
        let app = App::new()
            .wrap(middleware::Compress::default())
            .service(app::web_ui_index)
            .service(api::post_token)
            .service(api::get_network_summary)
            .service(api::post_network_reserve_address)
            .service(api::get_version)
            .service(api::patch_network_config)
            .service(api::post_wireguard_status)
            .service(app::web_ui_dist);

        #[cfg(debug_assertions)]
        {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);
            app.wrap(cors)
        }

        #[cfg(not(debug_assertions))]
        {
            app
        }
    };
    
    // Futures for HTTP/HTTPS servers
    let http_future = if config.agent.web.http.enabled {
        let http_addr = config.agent.web.address;
        let http_port = config.agent.web.http.port;
        let http_scripts = config.agent.firewall.http.clone();

        Some(Box::pin(async move {
            for hook in &http_scripts.pre_up {
                if hook.enabled {
                    execute_script(&hook.script, config.agent.web.http.port, HookType::PreUp);
                }
            }
            
            let bind_addr = SocketAddr::new(IpAddr::from(http_addr), http_port);
            match HttpServer::new(app_factory).bind(bind_addr) {
                Ok(http_server) => {
                    log::info!("HTTP server listening on http://{}", bind_addr);
                    http_server.run().await.unwrap_or_else(|e| {
                        log::error!("Unable to run the http server: {e}");
                    });
                }
                Err(e) => {
                    log::info!("Unable bind the http server to {}: {}", bind_addr, e);
                    for hook in &http_scripts.post_down {
                        if hook.enabled {
                            execute_script(&hook.script, config.agent.web.http.port, HookType::PostDown);
                        }
                    }
                    return Ok(());
                }
            };

            log::info!("Stopped HTTP server");
            for hook in &http_scripts.post_down {
                if hook.enabled {
                    execute_script(&hook.script, config.agent.web.http.port, HookType::PostDown);
                }
            }
            Ok(())
        }))
    } else {
        log::info!("HTTP server is disabled.");
        None
    };

    let https_future = if config.agent.web.https.enabled {
        let https_addr = config.agent.web.address;
        let https_port = config.agent.web.https.port;
        let https_scripts = config.agent.firewall.https.clone();

        let bind_addr = SocketAddr::new(IpAddr::from(https_addr), https_port);
        let mut tls_cert = WG_QUICKRS_CONFIG_FOLDER.get().unwrap().clone();
        tls_cert.push(config.agent.web.https.tls_cert.clone());
        let mut tls_key = WG_QUICKRS_CONFIG_FOLDER.get().unwrap().clone();
        tls_key.push(config.agent.web.https.tls_key.clone());
        match load_tls_config(&tls_cert, &tls_key) {
            Ok(tls_config) => Some(Box::pin(async move {
                for hook in &https_scripts.pre_up {
                    if hook.enabled {
                        execute_script(&hook.script, config.agent.web.https.port, HookType::PreUp);
                    }
                }

                match HttpServer::new(app_factory).bind_rustls_0_23(bind_addr, tls_config) {
                    Ok(https_server) => {
                        log::info!("HTTPS server listening on https://{}", bind_addr);
                        https_server.run().await.unwrap_or_else(|e| {
                            log::error!("Unable to run the https server: {e}");
                        });
                    }
                    Err(e) => {
                        log::info!("Unable bind the https server to {}: {}", bind_addr, e);
                        for hook in &https_scripts.post_down {
                            if hook.enabled {
                                execute_script(&hook.script, config.agent.web.https.port, HookType::PostDown);
                            }
                        }
                        return Ok(());
                    }
                };

                log::info!("Stopped HTTPS server");
                for hook in &https_scripts.post_down {
                    if hook.enabled {
                        execute_script(&hook.script, config.agent.web.https.port, HookType::PostDown);
                    }
                }
                Ok(())
            })),
            Err(e) => {
                log::error!("Failed to load TLS config (cert/key), HTTPS disabled: {e}");
                None
            }
        }
    } else {
        log::info!("HTTPS server is disabled.");
        None
    };

    // Run both concurrently if enabled
    match (http_future, https_future) {
        (Some(http), Some(https)) => try_join!(http, https).map(|_| ()),
        (Some(http), None) => http.await,
        (None, Some(https)) => https.await,
        (None, None) => {
            log::warn!("Neither HTTP nor HTTPS server is enabled.");
            Ok(())
        }
    }
}

fn load_tls_config(tls_cert: &PathBuf, tls_key: &PathBuf) -> Result<ServerConfig, ServerError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_e| {
            ServerError::TlsSetupFailed(
                "Failed to install aws-lc-rs default crypto provider".to_string(),
            )
        })?;

    let cert_chain = CertificateDer::pem_file_iter(tls_cert)
        .map_err(|_e| {
            ServerError::TlsSetupFailed("Failed to read TLS certificate file".to_string())
        })?
        .flatten()
        .collect();

    let key_der = PrivateKeyDer::from_pem_file(tls_key).map_err(|_e| {
        ServerError::TlsSetupFailed(
            "Failed to read TLS private key (expecting PKCS#8 format)".to_string(),
        )
    })?;

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .map_err(|_e| {
            ServerError::TlsSetupFailed(
                "Failed to build TLS config with provided certificate and key".to_string(),
            )
        })?;

    Ok(tls_config)
}
