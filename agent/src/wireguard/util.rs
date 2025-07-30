use crate::wireguard::cmd;
use actix_web::{HttpResponse, Responder};

pub(crate) fn respond_get_wireguard_public_private_keys() -> impl Responder {
    match cmd::get_wireguard_public_private_keys() {
        Ok(keys) => {
            return HttpResponse::Ok().content_type("application/json")
                .body(serde_json::to_string(&keys).unwrap());
        },
        Err(e) => {
            log::error!("{}", e);
            return HttpResponse::InternalServerError().into();
        }
    }
}

pub(crate) fn respond_get_wireguard_pre_shared_key() -> impl Responder {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct PreSharedKey {
        pre_shared_key: String,
    }

    match cmd::get_wireguard_pre_shared_key() {
        Ok(output) => {
            let body = PreSharedKey {
                pre_shared_key: output,
            };

            return HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string(&body).unwrap())
        }
        Err(e) => {
            log::error!("{}", e);
            return HttpResponse::InternalServerError().into();
        }
    }
}

