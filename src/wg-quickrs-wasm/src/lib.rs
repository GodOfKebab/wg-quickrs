pub mod helpers;
pub mod types;
pub mod validation;
pub mod timestamp;

// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_peer_wg_config_frontend(network_js: JsValue, peer_id: String, version: &str) -> String {
    let network: types::Network = serde_wasm_bindgen::from_value(network_js).unwrap();
    helpers::get_peer_wg_config(&network, peer_id, version, None).unwrap()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_connection_id_frontend(peer1: &str, peer2: &str) -> String {
    helpers::get_connection_id(peer1, peer2)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn check_internal_address(address: &str, network_js: JsValue) -> Result<JsValue, JsValue> {
    let network: types::Network = serde_wasm_bindgen::from_value(network_js).unwrap();
    Ok(serde_wasm_bindgen::to_value(&validation::check_internal_address(address, &network))?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn check_field_str_frontend(field_name: &str, field_variable: &str) -> Result<JsValue, JsValue> {
    Ok(serde_wasm_bindgen::to_value(&validation::check_field_str(field_name, &field_variable))?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn check_field_enabled_value_frontend(field_name: &str, field_variable_js: JsValue) -> Result<JsValue, JsValue> {
    let field_variable: types::EnabledValue = serde_wasm_bindgen::from_value(field_variable_js).unwrap();
    Ok(serde_wasm_bindgen::to_value(&validation::check_field_enabled_value(field_name, &field_variable))?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wg_public_key_from_private_key_frontend(base64_priv: &str) -> String {
    helpers::wg_public_key_from_private_key(base64_priv).unwrap()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wg_generate_key_frontend() -> String {
    helpers::wg_generate_key()
}
