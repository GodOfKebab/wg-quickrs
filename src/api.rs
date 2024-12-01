use std::fs::File;
use actix_web::{HttpResponse, Responder};

mod conf;


#[actix_web::get("/api/network")]
async fn network() -> impl Responder {
    let file = File::open(conf::CONFIG).expect("Unable to open file");
    let deserialized_map: conf::Config = serde_yml::from_reader(file).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&deserialized_map).unwrap())
}
