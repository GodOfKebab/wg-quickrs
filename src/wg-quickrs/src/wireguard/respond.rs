use crate::wireguard::cmd::{disable_tunnel, enable_tunnel, WG_STATUS};
use actix_web::{web, HttpResponse};
use serde_json::json;
use wg_quickrs_lib::types::misc::WireGuardStatus;
use crate::conf;

pub(crate) fn post_wireguard_server_status(body: web::Bytes) -> Result<HttpResponse, HttpResponse> {
    let config = conf::util::get_config()
        .map_err(|e| HttpResponse::InternalServerError().body(format!("failed to get config: {e}")))?;
    if !config.agent.vpn.enabled {
        return Err(HttpResponse::Forbidden().body("VPN is disabled in configuration"));
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct StatusBody {
        status: WireGuardStatus,
    }
    let body_raw = String::from_utf8_lossy(&body);
    let status_body: StatusBody = serde_json::from_str(&body_raw)
        .map_err(|e| HttpResponse::BadRequest().body(format!("invalid JSON: {}", e)))?;

    let action = if status_body.status == WireGuardStatus::UP {
        enable_tunnel
    } else if status_body.status == WireGuardStatus::DOWN {
        disable_tunnel
    } else {
        return Err(HttpResponse::BadRequest().body("invalid status value"));
    };

    match WG_STATUS.read() {
        Ok(current_status) if status_body.status == *current_status => {
            return Ok(HttpResponse::Ok().json(json!(status_body)));
        }
        Err(e) => {
            log::error!("Failed to acquire WG_STATUS lock: {}", e);
            return Err(HttpResponse::InternalServerError().body("failed to check current WireGuard status"));
        }
        _ => {}
    }

    match action() {
        Ok(_) => Ok(HttpResponse::Ok().json(json!(status_body))),
        Err(e) => {
            log::error!("{e}");
            Err(HttpResponse::InternalServerError().body(format!("failed to run command: {e}")))
        }
    }
}
