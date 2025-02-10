use actix_web::{get, web, HttpResponse, Responder};
use sha2::{Digest, Sha256};
use std::fs;
use std::hash::Hash;

mod conf;

#[derive(serde::Deserialize)]
struct SummaryBody {
    only_network_digest: bool,
}

#[get("/api/summary")]
async fn get_summary(params: web::Query<SummaryBody>) -> impl Responder {
    let file_contents = fs::read_to_string(conf::DEFAULT_CONF_FILE).expect("Unable to open file");
    let mut buf = [0u8; 64];
    let network_digest: &str = base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf).expect("TODO: panic message");

    let mut resp_body = String::new();
    if (params.only_network_digest) {
        let mut config_digest = conf::ConfigDigest {
            network_digest: network_digest.to_string(),
            status: conf::WireGuardStatus::UP.value(),
            timestamp: 0,
        };
        config_digest.put_timestamp();
        resp_body = serde_json::to_string(&config_digest).unwrap();
    } else {
        let mut config: conf::Config = serde_yml::from_str(&file_contents).unwrap();
        config.network_digest = network_digest.to_string();
        config.status = conf::WireGuardStatus::UP.value();  // TODO: check wg status
        config.put_timestamp();
        resp_body = serde_json::to_string(&config).unwrap();
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body(resp_body)
}
