use crate::conf;
use crate::macros::*;
use crate::wireguard;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, post, web};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use once_cell::sync::Lazy;
use rand::{RngCore, rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String, // Subject (user id)
    exp: u64,    // Expiration time as a timestamp
}

// Secret key for signing tokens
static JWT_SECRETS: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    let mut key = [0u8; 32];
    rng().fill_bytes(&mut key);
    (
        EncodingKey::from_secret(&key),
        DecodingKey::from_secret(&key),
    )
});

#[get("/api/version")]
async fn get_version(req: HttpRequest) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }

    HttpResponse::Ok().json(json!({
        "version": wg_quickrs_version!(),
        "build": build_info!(),
    }))
}

#[derive(serde::Deserialize)]
pub(crate) struct SummaryBody {
    pub(crate) only_digest: bool,
}

#[get("/api/network/summary")]
async fn get_network_summary(req: HttpRequest, query: web::Query<SummaryBody>) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    conf::respond::get_network_summary(query)
}

#[patch("/api/network/config")]
async fn patch_network_config(req: HttpRequest, body: web::Bytes) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    conf::respond::patch_network_config(body)
}

#[post("/api/network/reserve/address")]
async fn post_network_reserve_address(req: HttpRequest) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    conf::respond::post_network_reserve_address()
}

#[post("/api/wireguard/status")]
async fn post_wireguard_status(req: HttpRequest, body: web::Bytes) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    wireguard::respond::post_wireguard_server_status(body)
}

#[post("/api/token")]
async fn post_token(body: web::Bytes) -> impl Responder {
    // check password-based auth
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Unable to get config");
        }
    };
    if !config.agent.web.password.enabled {
        return HttpResponse::NoContent().body("Token authentication not enabled");
    }

    #[derive(Serialize, Deserialize)]
    struct LoginBody {
        client_id: String,
        password: String,
    }
    let body_raw = String::from_utf8_lossy(&body);
    let status_body: LoginBody = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Invalid JSON: {err}")
            }));
        }
    };
    let client_id = &status_body.client_id;
    let password = &status_body.password;

    // check password-based auth
    let parsed_hash = PasswordHash::new(&config.agent.web.password.hash).expect("Invalid hash format");
    if Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_err() {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    let expiration = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() + 3600, // 1-hour expiry
        Err(_) => return HttpResponse::InternalServerError().body("SystemTime before UNIX EPOCH!"),
    };

    let claims = Claims {
        sub: client_id.clone(),
        exp: expiration,
    };

    match encode(&Header::default(), &claims, &JWT_SECRETS.0) {
        Ok(token) => HttpResponse::Ok().body(token),
        Err(_) => HttpResponse::InternalServerError().body("Token creation error"),
    }
}

fn enforce_auth(req: HttpRequest) -> Result<(), HttpResponse> {
    // check password-based auth
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(_) => {
            return Err(HttpResponse::InternalServerError().body("Unable to get config"));
        }
    };
    if !config.agent.web.password.enabled {
        return Ok(());
    }

    if let Some(auth_header) = req.headers().get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        let validation = Validation::new(Algorithm::HS256);

        return match decode::<Claims>(token, &JWT_SECRETS.1, &validation) {
            Ok(_) => Ok(()),
            Err(_) => Err(HttpResponse::Unauthorized()
                .content_type("text/plain; charset=utf-8")
                .body("Invalid token")),
        };
    }

    Err(HttpResponse::Unauthorized()
        .content_type("text/plain; charset=utf-8")
        .body("Authorization header missing or invalid"))
}
