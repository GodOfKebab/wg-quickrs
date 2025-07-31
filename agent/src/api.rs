use crate::conf;
use crate::macros::*;
use crate::wireguard;
use actix_web::{HttpResponse, Responder, get, patch, post, web};
use serde_json::json;

#[get("/version")]
async fn get_version() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "backend": backend_version!(),
        "frontend": frontend_version!(),
        "built": build_timestamp!(),
    }))
}

#[derive(serde::Deserialize)]
pub(crate) struct SummaryBody {
    pub(crate) only_digest: bool,
}

#[get("/api/network/summary")]
async fn get_network_summary(params: web::Query<SummaryBody>) -> impl Responder {
    conf::respond::get_network_summary(params)
}

#[get("/api/wireguard/public_private_keys")]
async fn get_wireguard_public_private_keys() -> impl Responder {
    wireguard::respond::get_wireguard_public_private_keys()
}

#[get("/api/wireguard/pre_shared_key")]
async fn get_wireguard_pre_shared_key() -> impl Responder {
    wireguard::respond::get_wireguard_pre_shared_key()
}

#[patch("/api/network/config")]
async fn patch_network_config(body: web::Bytes) -> impl Responder {
    conf::respond::patch_network_config(body)
}

#[get("/api/network/lease/id-address")]
async fn get_network_lease_id_address() -> impl Responder {
    conf::respond::get_network_lease_id_address()
}

#[post("/api/wireguard/server/status")]
async fn post_wireguard_server_status(body: web::Bytes) -> impl Responder {
    wireguard::respond::post_wireguard_server_status(body)
}
