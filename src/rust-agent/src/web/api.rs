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

#[derive(Debug, Serialize, Deserialize)]
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

#[get("/version")]
async fn get_version() -> impl Responder {
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

#[get("/api/wireguard/public_private_keys")]
async fn get_wireguard_public_private_keys(req: HttpRequest) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    wireguard::respond::get_wireguard_public_private_keys()
}

#[get("/api/wireguard/pre_shared_key")]
async fn get_wireguard_pre_shared_key(req: HttpRequest) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    wireguard::respond::get_wireguard_pre_shared_key()
}

#[patch("/api/network/config")]
async fn patch_network_config(req: HttpRequest, body: web::Bytes) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    conf::respond::patch_network_config(body)
}

#[get("/api/network/lease/id-address")]
async fn get_network_lease_id_address(req: HttpRequest) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    conf::respond::get_network_lease_id_address()
}

#[post("/api/wireguard/server/status")]
async fn post_wireguard_server_status(req: HttpRequest, body: web::Bytes) -> impl Responder {
    if let Err(e) = enforce_auth(req) {
        return e;
    }
    wireguard::respond::post_wireguard_server_status(body)
}

#[derive(Deserialize)]
struct LoginRequest {
    client_id: String,
    password: String,
}
#[post("/api/token")]
async fn post_token(query: web::Query<LoginRequest>) -> impl Responder {
    let client_id = &query.client_id;
    let password = &query.password;

    // check password-based auth
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Unable to get config");
        }
    };
    if config.agent.web.password.enabled {
        let parsed_hash =
            PasswordHash::new(&config.agent.web.password.hash).expect("Invalid hash format");
        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return HttpResponse::Unauthorized().body("Invalid credentials");
        }
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
