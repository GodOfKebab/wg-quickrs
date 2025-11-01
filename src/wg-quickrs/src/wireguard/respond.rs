use crate::wireguard::cmd::{disable_tunnel, enable_tunnel, WG_STATUS};
use actix_web::{web, HttpResponse};
use serde_json::json;
use wg_quickrs_wasm::types::misc::WireGuardStatus;

pub(crate) fn post_wireguard_server_status(body: web::Bytes) -> HttpResponse {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct StatusBody {
        status: WireGuardStatus,
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

    let action = if status_body.status == WireGuardStatus::UP {
        enable_tunnel
    } else if status_body.status == WireGuardStatus::DOWN {
        disable_tunnel
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid status value"
        }));
    };

    if status_body.status == *WG_STATUS.lock().unwrap() {
        return HttpResponse::Ok().json(json!(status_body));
    }

    match action() {
        Ok(_) => HttpResponse::Ok().json(json!(status_body)),
        Err(e) => {
            log::error!("{e}");
            HttpResponse::InternalServerError().into()
        }
    }
}
