#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

mod api;
mod app;
mod conf;
mod wireguard;
mod macros;

#[derive(Parser, Debug)]
#[command(
    version = full_version!(),
    about = "Run the wg-rusteze network agent",
    long_about = "Starts the wg-rusteze agent with the default/provided configuration file. \
                  Use this tool to manage the peer and network configuration of the \
                  WireGuard-based overlay network over the web console."
)]
struct Args {
    #[arg(
        long,
        default_value = ".wg-rusteze/conf.yml",
        value_name = "WG_RUSTEZE_CONFIG_FILE_PATH"
    )]
    wg_rusteze_config_file: PathBuf,
    #[arg(
        long,
        default_value = "/opt/homebrew/etc/wireguard/",
        value_name = "WIREGUARD_CONFIG_FOLDER_PATH"
    )]
    wireguard_config_folder: PathBuf,
}

pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // start logger and print version
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    log::info!(full_version!());

    // get the wg_rusteze config file path
    let args = Args::parse();
    WG_RUSTEZE_CONFIG_FILE.set(args.wg_rusteze_config_file).unwrap();
    log::info!("using the wg-rusteze config file at \"{}\"", WG_RUSTEZE_CONFIG_FILE.get().unwrap().display());

    // get the wireguard config file path
    let config: config_wasm::types::Config = conf::logic::get_config();
    let mut wireguard_config_folder = PathBuf::from(&args.wireguard_config_folder);
    wireguard_config_folder.push(format!("{}.conf", config.network.identifier));
    WIREGUARD_CONFIG_FILE.set(wireguard_config_folder).unwrap();
    log::info!("using the wireguard config file at \"{}\"", WIREGUARD_CONFIG_FILE.get().unwrap().display());

    // start the tunnel
    let _wg_startup_success = wireguard::cmd::start_wireguard_tunnel(&config).unwrap_or_else(|e| {
        log::error!("{}", e);
    });

    // start the HTTP server for frontend and API control
    log::info!("frontend/API accessible at {}://{}:{}/", config.agent.web.scheme, config.agent.address, config.agent.web.port);
    HttpServer::new(|| {
        let app = App::new()
            .wrap(middleware::Compress::default())
            .service(app::web_ui_index)
            .service(api::get_network_summary)
            .service(api::get_wireguard_public_private_keys)
            .service(api::get_wireguard_pre_shared_key)
            .service(api::patch_network_config)
            .service(api::get_network_lease_id_address)
            .service(api::post_wireguard_server_status)
            .service(api::get_version)
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
        .bind((config.agent.address, config.agent.web.port))?
        .run()
        .await
}

