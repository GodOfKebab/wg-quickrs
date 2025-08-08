
pub mod types;
pub mod helpers;
pub mod validation;


// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_peer_wg_config_frontend(network_js: JsValue, peer_id: String, version: &str) -> String {
    let network: types::Network = serde_wasm_bindgen::from_value(network_js).unwrap();
    helpers::get_peer_wg_config(&network, peer_id, version).unwrap()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_connection_id_frontend(peer1: &str, peer2: &str) -> String {
    helpers::get_connection_id(peer1, peer2)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn check_field_frontend(field_name: &str, field_variable_json: &str) -> String {
    println!(
        "Checking field: {} with value: {}",
        field_name, field_variable_json
    );
    match serde_json::from_str::<validation::FieldValue>(field_variable_json) {
        Ok(field_variable) => {
            let ret = validation::check_field(field_name, &field_variable);
            serde_json::to_string(&ret)
                .unwrap_or_else(|_| r#"{"status":false,"msg":"Failed to serialize result"}"#.into())
        }
        Err(_) => r#"{"status":false,"msg":"Invalid JSON input"}"#.into(),
    }
}
