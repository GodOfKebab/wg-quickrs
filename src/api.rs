use actix_web::{HttpResponse, Responder};
use std::fs::File;

mod conf;


#[actix_web::get("/api/network")]
async fn get_network() -> impl Responder {
    let file = File::open(conf::DEFAULT_CONF_FILE).expect("Unable to open file");
    let mut resp: conf::Config = serde_yml::from_reader(file).unwrap();

    resp.set_status(conf::WireGuardStatus::UP.value());  // TODO: check wg status
    resp.put_timestamp();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&resp).unwrap())
}

#[actix_web::get("/api/server/status")]
async fn get_server_status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "up"}"#)
}
