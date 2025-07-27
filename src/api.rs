use crate::conf;
use crate::macros::*;
use crate::wireguard;
use actix_web::{get, patch, web, HttpResponse, Responder};
use serde::Serialize;

#[get("/version")]
async fn get_version() -> impl Responder {
    #[derive(Serialize)]
    struct VersionInfo {
        backend: &'static str,
        frontend: &'static str,
        built: &'static str,
    }
    let version_info = VersionInfo {
        backend: backend_version!(),
        frontend: frontend_version!(),
        built: build_timestamp!(),
    };
    return HttpResponse::Ok().json(version_info);
}


#[derive(serde::Deserialize)]
pub(crate) struct SummaryBody {
    pub(crate) only_network_digest: bool,
}

#[get("/api/network/summary")]
async fn get_network_summary(params: web::Query<SummaryBody>) -> impl Responder {
    return conf::logic::respond_get_network_summary(params);
}

#[get("/api/wireguard/public_private_key")]
async fn get_wireguard_public_private_key() -> impl Responder {
    return wireguard::util::respond_get_wireguard_public_private_key();
}

#[get("/api/wireguard/pre_shared_key")]
async fn get_wireguard_pre_shared_key() -> impl Responder {
    return wireguard::util::respond_get_wireguard_pre_shared_key();
}

#[patch("/api/network/config")]
async fn patch_network_config(body: web::Bytes) -> impl Responder {
    return conf::logic::respond_patch_network_config(body);
}

#[get("/api/network/lease/id-address")]
async fn get_network_lease_id_address() -> impl Responder {
    return conf::logic::respond_get_network_lease_id_address();
}
