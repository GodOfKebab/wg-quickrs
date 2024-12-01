mod api;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use mime_guess::from_path;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/dist"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::get("/")]
async fn web_ui_index() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/{_:.*}")]
async fn web_ui_dist(path: web::Path<String>) -> impl Responder { handle_embedded_file(path.as_str()) }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hosting the frontend at http://{}:{}/", "127.0.0.1", 8080);
    HttpServer::new(|| {
        App::new()
            .service(web_ui_index)
            .service(api::network)
            .service(web_ui_dist)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

