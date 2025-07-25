use actix_web::{get, web, HttpResponse, Responder};
use sha2::Digest;
use std::hash::Hash;
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) mod conf;

#[derive(serde::Deserialize)]
struct SummaryBody {
    only_network_digest: bool,
}

#[get("/api/summary")]
async fn get_summary(params: web::Query<SummaryBody>) -> impl Responder {
    let resp_body = if params.only_network_digest {
        serde_json::to_string(&conf::ConfigDigest::from(&conf::get_config()))
    } else {
        serde_json::to_string(&conf::get_config())
    }.unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(resp_body)
}

#[get("/api/public_private_key")]
async fn get_public_private_key() -> impl Responder {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct PublicPrivateKeyBody {
        public_key: String,
        private_key: String,
    }

    // execute $ wg genkey
    let priv_key_resp = Command::new("wg")
        .arg("genkey")
        .output()
        .unwrap();
    if !priv_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let priv_key = String::from_utf8_lossy(&priv_key_resp.stdout);

    // execute $ echo ~PRIVATE_KEY~ | wg pubkey
    let mut pub_key_child = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdin) = pub_key_child.stdin.as_mut() {
        stdin.write_all(priv_key.as_bytes()).unwrap();
    }
    let pub_key_resp = pub_key_child.wait_with_output().unwrap();
    if !pub_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let pub_key = String::from_utf8_lossy(&pub_key_resp.stdout);

    let body = PublicPrivateKeyBody {
        public_key: pub_key.trim().to_string(),
        private_key: priv_key.trim().to_string(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}

#[get("/api/pre_shared_key")]
async fn get_pre_shared_key() -> impl Responder {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct PreSharedKeyBody {
        pre_shared_key: String,
    }

    // execute $ wg genpsk
    let pre_shared_key_resp = Command::new("wg")
        .arg("genpsk")
        .output()
        .unwrap();
    if !pre_shared_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let pre_shared_key = String::from_utf8_lossy(&pre_shared_key_resp.stdout);

    let body = PreSharedKeyBody {
        pre_shared_key: pre_shared_key.trim().to_string(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}
