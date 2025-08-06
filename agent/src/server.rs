use crate::{api, app};
#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use config_wasm::types::Config;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("unable to configure TLS setup for HTTPS: {0}")]
    TLSConfiguration(String),
}

pub(crate) async fn run_http_server(
    config: &Config,
    tls_cert: &PathBuf,
    tls_key: &PathBuf,
) -> std::io::Result<()> {
    // start the HTTP server with TLS for frontend and API control
    let bind_addr = (config.agent.address.clone(), config.agent.web.port);

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

    if config.agent.web.use_tls {
        // Try to build TLS config — if fails, fallback immediately
        let tls_config = match load_tls_config(tls_cert, tls_key) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!("Failed to load TLS config (cert/key), falling back to HTTP: {e}");
                // Fallback to HTTP server immediately
                let http_server = HttpServer::new(app_factory)
                    .bind(&bind_addr)
                    .expect("Failed to bind HTTP fallback server");
                log::info!(
                    "Started HTTP frontend/API at http://{}:{}/",
                    bind_addr.0,
                    bind_addr.1
                );
                return http_server.run().await;
            }
        };

        // Try to bind HTTPS server
        match HttpServer::new(app_factory).bind_rustls_0_23(bind_addr.clone(), tls_config) {
            Ok(server) => {
                log::info!(
                    "Started HTTPS frontend/API at https://{}:{}/",
                    bind_addr.0,
                    bind_addr.1
                );
                return server.run().await;
            }
            Err(err) => {
                log::warn!(
                    "Failed to bind HTTPS server on {}:{}, falling back to HTTP: {}",
                    bind_addr.0,
                    bind_addr.1,
                    err
                );
            }
        };
    }

    let http_server = HttpServer::new(app_factory)
        .bind(&bind_addr)
        .unwrap_or_else(|_| {
            panic!(
                "Failed to bind HTTP server on {}:{}",
                bind_addr.0, bind_addr.1
            )
        });
    log::info!(
        "Started HTTP frontend/API at http://{}:{}/",
        bind_addr.0,
        bind_addr.1
    );
    http_server.run().await
}

fn load_tls_config(tls_cert: &PathBuf, tls_key: &PathBuf) -> Result<ServerConfig, ServerError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_e| {
            ServerError::TLSConfiguration(
                "Failed to install aws-lc-rs default crypto provider".to_string(),
            )
        })?;

    let cert_chain = CertificateDer::pem_file_iter(tls_cert)
        .map_err(|_e| {
            ServerError::TLSConfiguration("Failed to read TLS certificate file".to_string())
        })?
        .flatten()
        .collect();

    let key_der = PrivateKeyDer::from_pem_file(tls_key).map_err(|_e| {
        ServerError::TLSConfiguration(
            "Failed to read TLS private key (expecting PKCS#8 format)".to_string(),
        )
    })?;

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .map_err(|_e| {
            ServerError::TLSConfiguration(
                "Failed to build TLS config with provided certificate and key".to_string(),
            )
        })?;

    Ok(tls_config)
}
