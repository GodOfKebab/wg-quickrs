use crate::WG_QUICKRS_CONFIG_FOLDER;
use crate::web::api;
use crate::web::app;
#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use wg_quickrs_wasm::types::Config;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;
use tokio::try_join;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("web::server::error::tls_setup_failed -> failed to configure TLS for HTTPS: {0}")]
    TlsSetupFailed(String),
}

fn setup_firewall_rules(utility: PathBuf, port: u16, is_add_action: bool) {
    if let Some(utility_fn) = utility.file_name()
        && utility_fn.to_string_lossy() == "iptables"
    {
        // iptables -A/-D INPUT -p tcp --dport PORT -j ACCEPT
        let readable_command = format!(
            "$ {} {} INPUT -p tcp --dport {} -j ACCEPT",
            utility.display(),
            if is_add_action { "-A" } else { "-D" },
            port
        );
        match Command::new(utility)
            .arg(if is_add_action { "-A" } else { "-D" })
            .arg("INPUT")
            .arg("-p")
            .arg("tcp")
            .arg("--dport")
            .arg(format!("{}", port))
            .arg("-j")
            .arg("ACCEPT")
            .output()
        {
            Ok(output) => {
                log::info!("{readable_command}");
                if !output.stdout.is_empty() {
                    log::debug!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    log::warn!("{}", String::from_utf8_lossy(&output.stderr));
                }
                if !output.status.success() {
                    log::warn!("firewall input rule update for http(s) failed");
                }
            }
            Err(_) => {
                log::warn!("firewall input rule update for http(s) failed");
            }
        };
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
            .service(api::get_network_lease_id_address)
            .service(api::get_wireguard_pre_shared_key)
            .service(api::get_wireguard_private_key)
            .service(api::get_version)
            .service(api::patch_network_config)
            .service(api::post_wireguard_server_status)
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
        Some(Box::pin(async move {
            if config.agent.firewall.enabled {
                setup_firewall_rules(
                    config.agent.firewall.utility.clone(),
                    config.agent.web.http.port,
                    true,
                );
            }

            let bind_addr = (config.agent.web.address.clone(), config.agent.web.http.port);
            match HttpServer::new(app_factory).bind(bind_addr.clone()) {
                Ok(http_server) => {
                    log::info!(
                        "Starting HTTP server at http://{}:{}/",
                        bind_addr.0,
                        bind_addr.1
                    );
                    http_server.run().await.unwrap_or_else(|e| {
                        log::error!("Unable to run the http server: {e}");
                    });
                }
                Err(e) => {
                    log::error!(
                        "Unable bind the http server to {}:{} => {}",
                        bind_addr.0,
                        bind_addr.1,
                        e
                    );
                    return Ok(());
                }
            };

            log::info!("Stopped HTTP server");
            if config.agent.firewall.enabled {
                setup_firewall_rules(
                    config.agent.firewall.utility.clone(),
                    config.agent.web.http.port,
                    false,
                );
            }
            Ok(())
        }))
    } else {
        log::info!("HTTP server is disabled.");
        None
    };

    let https_future = if config.agent.web.https.enabled {
        if config.agent.firewall.enabled {
            setup_firewall_rules(
                config.agent.firewall.utility.clone(),
                config.agent.web.https.port,
                true,
            );
        }
        let bind_addr = (
            config.agent.web.address.clone(),
            config.agent.web.https.port,
        );
        let mut tls_cert = WG_QUICKRS_CONFIG_FOLDER.get().unwrap().clone();
        tls_cert.push(config.agent.web.https.tls_cert.clone());
        let mut tls_key = WG_QUICKRS_CONFIG_FOLDER.get().unwrap().clone();
        tls_key.push(config.agent.web.https.tls_key.clone());
        match load_tls_config(&tls_cert, &tls_key) {
            Ok(tls_config) => Some(Box::pin(async move {
                match HttpServer::new(app_factory).bind_rustls_0_23(bind_addr.clone(), tls_config) {
                    Ok(https_server) => {
                        log::info!(
                            "Starting HTTPS server at https://{}:{}/",
                            bind_addr.0,
                            bind_addr.1
                        );
                        https_server.run().await.unwrap_or_else(|e| {
                            log::error!("Unable to run the https server: {e}");
                        });
                    }
                    Err(e) => {
                        log::error!(
                            "Unable bind the https server to {}:{} => {}",
                            bind_addr.0,
                            bind_addr.1,
                            e
                        );
                        return Ok(());
                    }
                };

                log::info!("Stopped HTTPS server");
                if config.agent.firewall.enabled {
                    setup_firewall_rules(
                        config.agent.firewall.utility.clone(),
                        config.agent.web.https.port,
                        false,
                    );
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
