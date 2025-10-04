use crate::conf;
use crate::wireguard::cmd;
use crate::wireguard::cmd::{disable_tunnel, enable_tunnel, WG_STATUS};
use actix_web::{web, HttpResponse};
use serde_json::json;

pub(crate) fn get_wireguard_public_private_keys() -> HttpResponse {
    match cmd::get_public_private_keys() {
        Ok(keys) => HttpResponse::Ok().json(keys),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().into()
        }
    }
}

pub(crate) fn get_wireguard_pre_shared_key() -> HttpResponse {
    match cmd::get_pre_shared_key() {
        Ok(key) => HttpResponse::Ok().json(key),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().into()
        }
    }
}

pub(crate) fn post_wireguard_server_status(body: web::Bytes) -> HttpResponse {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct StatusBody {
        status: u8,
    }
    let body_raw = String::from_utf8_lossy(&body);
    let status_body: StatusBody = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Invalid JSON: {err}")
            }));
        }
    };
    if status_body.status == WG_STATUS.lock().unwrap().value() {
        return HttpResponse::Ok().json(json!(status_body));
    }

    let conf = match conf::util::get_config() {
        Ok(conf) => conf,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Unable to get config");
        }
    };
    let action = if status_body.status == rust_wasm::types::WireGuardStatus::UP.value() {
        enable_tunnel
    } else if status_body.status == rust_wasm::types::WireGuardStatus::DOWN.value() {
        disable_tunnel
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid status value"
        }));
    };

    match action(&conf) {
        Ok(_) => HttpResponse::Ok().json(json!(status_body)),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().into()
        }
    }
}
