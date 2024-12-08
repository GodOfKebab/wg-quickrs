use actix_web::{HttpResponse, Responder};
use std::fs::File;

mod conf;


#[actix_web::get("/api/network")]
async fn network() -> impl Responder {
    let file = File::open(conf::CONF_FILE).expect("Unable to open file");
    let resp: conf::Config = serde_yml::from_reader(file).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&resp).unwrap())
}

#[actix_web::get("/api/server/status")]
async fn server_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "up"}"#)
}
