use actix_web::{App, HttpServer};

mod api;
mod app;

const DEFAULT_SERVER_ADDRESS_PORT: (&str, u16) = ("127.0.0.1", 8080);


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hosting the frontend at http://{}:{}/", DEFAULT_SERVER_ADDRESS_PORT.0, DEFAULT_SERVER_ADDRESS_PORT.1);
    HttpServer::new(|| {
        App::new()
            .service(app::web_ui_index)
            .service(api::get_network)
            .service(api::get_server_status)
            .service(app::web_ui_dist)
    })
        .bind(DEFAULT_SERVER_ADDRESS_PORT)?
        .run()
        .await
}

