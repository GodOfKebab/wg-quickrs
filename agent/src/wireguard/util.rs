use crate::conf;
use crate::wireguard::cmd;
use crate::wireguard::cmd::{disable_wireguard, enable_wireguard};
use actix_web::{web, HttpResponse, Responder};

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

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct StatusBody {
    pub(crate) status: u8,
}
pub(crate) fn respond_post_wireguard_server_status(body: web::Bytes) -> impl Responder {
    let body_raw = String::from_utf8_lossy(&body);
    let status_body: StatusBody = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(format!(r#"{{"error":"Invalid JSON: {}"}}"#, err));
        }
    };

    let conf = conf::logic::get_config();
    if status_body.status == config_wasm::types::WireGuardStatus::UP.value() {
        match enable_wireguard(&conf) {
            Ok(_) => {
                return HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(e) => {
                log::error!("{}", e);
                return HttpResponse::InternalServerError().into();
            }
        }
    } else if status_body.status == config_wasm::types::WireGuardStatus::DOWN.value() {
        match disable_wireguard(&conf) {
            Ok(_) => {
                return HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(e) => {
                log::error!("{}", e);
                return HttpResponse::InternalServerError().into();
            }
        }
    }
    return HttpResponse::BadRequest().into();
}
