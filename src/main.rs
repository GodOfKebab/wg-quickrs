use actix_web::{App, HttpServer};
use log::LevelFilter;
use std::fs;

mod api;
mod app;

use simple_logger::SimpleLogger;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let file_contents = fs::read_to_string(api::conf::DEFAULT_CONF_FILE).expect("Unable to open file");
    let mut config: api::conf::Config = serde_yml::from_str(&file_contents).unwrap();

    log::info!("Hosting the frontend at {}://{}:{}/", config.agent.web.scheme, config.agent.address, config.agent.web.port);

    HttpServer::new(|| {
        App::new()
            .service(app::web_ui_index)
            .service(api::get_summary)
            .service(app::web_ui_dist)
    })
        .bind((config.agent.address, config.agent.web.port))?
        .run()
        .await
}

