use crate::WG_RUSTEZE_CONFIG_FOLDER;
use crate::web::api;
use crate::web::app;
#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use rust_wasm::types::Config;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::path::PathBuf;
use thiserror::Error;
use tokio::try_join;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("web::server::error::tls_setup_failed -> failed to configure TLS for HTTPS: {0}")]
    TlsSetupFailed(String),
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
            .service(api::get_wireguard_public_private_keys)
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
        let http_bind_addr = (config.agent.web.address.clone(), config.agent.web.http.port);
        let http_server = HttpServer::new(app_factory)
            .bind(http_bind_addr.clone())
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to bind HTTP server on {}:{}",
                    http_bind_addr.0, http_bind_addr.1
                )
            });
        log::info!(
            "Started HTTP frontend/API at http://{}:{}/",
            http_bind_addr.0,
            http_bind_addr.1
        );
        Some(http_server.run())
    } else {
        None
    };

    let https_future = if config.agent.web.https.enabled {
        let https_bind_addr = (
            config.agent.web.address.clone(),
            config.agent.web.https.port,
        );
        let mut tls_cert = WG_RUSTEZE_CONFIG_FOLDER.get().unwrap().clone();
        tls_cert.push(config.agent.web.https.tls_cert.clone());
        let mut tls_key = WG_RUSTEZE_CONFIG_FOLDER.get().unwrap().clone();
        tls_key.push(config.agent.web.https.tls_key.clone());
        match load_tls_config(&tls_cert, &tls_key) {
            Ok(tls_config) => {
                let https_server = HttpServer::new(app_factory)
                    .bind_rustls_0_23(https_bind_addr.clone(), tls_config)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to bind HTTPS server on {}:{}",
                            https_bind_addr.0, https_bind_addr.1
                        )
                    });
                log::info!(
                    "Started HTTPS frontend/API at https://{}:{}/",
                    https_bind_addr.0,
                    https_bind_addr.1
                );
                Some(https_server.run())
            }
            Err(e) => {
                log::error!("Failed to load TLS config (cert/key), HTTPS disabled: {e}");
                None
            }
        }
    } else {
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
