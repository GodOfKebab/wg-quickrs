use crate::{api, app};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use config_wasm::types::Config;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::path::PathBuf;

pub(crate) async fn run_http_server(
    config: &Config,
    tls_cert: &PathBuf,
    tls_key: &PathBuf,
) -> std::io::Result<()> {
    // start the HTTP server with TLS for frontend and API control
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install aws-lc-rs default crypto provider");

    // load TLS key/cert files
    let cert_chain = CertificateDer::pem_file_iter(tls_cert)
        .expect("Failed to read TLS certificate file")
        .flatten()
        .collect();

    let key_der = PrivateKeyDer::from_pem_file(tls_key)
        .expect("Failed to read TLS private key (expecting PKCS#8 format)");

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .expect("Failed to build TLS config with provided certificate and key");

    log::info!(
        "frontend/API accessible at {}://{}:{}/",
        config.agent.web.scheme.clone(),
        config.agent.address.clone(),
        config.agent.web.port.clone()
    );
    HttpServer::new(|| {
        let app = App::new()
            .wrap(middleware::Compress::default())
            .service(app::web_ui_index)
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
    })
    .bind_rustls_0_23(
        (config.agent.address.clone(), config.agent.web.port),
        tls_config,
    )?
    .run()
    .await
}
