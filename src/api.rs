use actix_web::{HttpResponse, Responder};
use std::fs::File;

mod conf;


#[actix_web::get("/api/network")]
async fn network() -> impl Responder {
    let file = File::open(conf::CONF_FILE).expect("Unable to open file");
    let deserialized_map: conf::Config = serde_yml::from_reader(file).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&deserialized_map).unwrap())
}
