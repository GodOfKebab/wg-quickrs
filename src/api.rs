use actix_web::{get, web, HttpResponse, Responder};
use sha2::Digest;
use std::hash::Hash;

pub(crate) mod conf;

#[derive(serde::Deserialize)]
struct SummaryBody {
    only_network_digest: bool,
}

#[get("/api/summary")]
async fn get_summary(params: web::Query<SummaryBody>) -> impl Responder {
    let mut resp_body = String::new();
    if (params.only_network_digest) {
        resp_body = serde_json::to_string(&conf::ConfigDigest::from(&conf::get_config())).unwrap();
    } else {
        resp_body = serde_json::to_string(&conf::get_config()).unwrap();
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body(resp_body)
}
