#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::fs;

mod api;
mod app;
mod conf;
mod wireguard;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let file_contents = fs::read_to_string(conf::DEFAULT_CONF_FILE).expect("Unable to open file");
    let config: conf::types::Config = serde_yml::from_str(&file_contents).unwrap();

    log::info!("Hosting the frontend at {}://{}:{}/", config.agent.web.scheme, config.agent.address, config.agent.web.port);

    HttpServer::new(|| {
        let app = App::new()
            .wrap(middleware::Compress::default())
            .service(app::web_ui_index)
            .service(api::get_network_summary)
            .service(api::get_wireguard_public_private_key)
            .service(api::get_wireguard_pre_shared_key)
            .service(api::patch_network_config)
            .service(api::get_network_lease_id_address)
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

