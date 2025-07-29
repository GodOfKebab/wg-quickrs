#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use simple_logger::SimpleLogger;

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
        short,
        long,
        default_value = ".wg-rusteze/conf.yml",
        value_name = "CONFIG_PATH"
    )]
    wg_rusteze_config_file: String,
}
pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<String> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    log::info!(full_version!());

    let args = Args::parse();
    WG_RUSTEZE_CONFIG_FILE.set(args.wg_rusteze_config_file).unwrap();
    log::info!("using the wg-rusteze config file at \"{}\"", WG_RUSTEZE_CONFIG_FILE.get().expect("CONFIG_FILE not set"));

    let config: conf::types::Config = conf::logic::get_config();
    log::info!("wg-rusteze config digest: {}", config.digest);

    log::info!("wg-rusteze frontend accessible at {}://{}:{}/", config.agent.web.scheme, config.agent.address, config.agent.web.port);
    HttpServer::new(|| {
        let app = App::new()
            .wrap(middleware::Compress::default())
            .service(app::web_ui_index)
            .service(api::get_network_summary)
            .service(api::get_wireguard_public_private_key)
            .service(api::get_wireguard_pre_shared_key)
            .service(api::patch_network_config)
            .service(api::get_network_lease_id_address)
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

