use actix_web::{App, HttpServer, Responder};

mod api;
mod app;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hosting the frontend at http://{}:{}/", "0.0.0.0", 8080);
    HttpServer::new(|| {
        App::new()
            .service(app::web_ui_index)
            .service(api::network)
            .service(api::server_status)
            .service(app::web_ui_dist)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

