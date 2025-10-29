pub mod helpers;
pub mod types;
pub mod validation;
pub mod macros;
pub mod validation_helpers;

// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
mod wasm {
    use serde::{Deserialize, Serialize};
    use serde_wasm_bindgen;
    use wasm_bindgen::prelude::*;
    use uuid::Uuid;
    use crate::{helpers, types, validation};
    use crate::types::conf::WireGuardKey;

    #[derive(Serialize, Deserialize)]
    struct ValidationResultWasm {
        failed: bool,
        message: String,
    }

    impl<T> From<Result<T, validation::ValidationError>> for ValidationResultWasm {
        fn from(result: Result<T, validation::ValidationError>) -> Self {
            match result {
                Ok(_) => Self { failed: false, message: String::new() },
                Err(e) => Self { failed: true, message: e.to_string() },
            }
        }
    }

    #[wasm_bindgen]
    pub fn get_peer_wg_config_wasm(network_js: JsValue, peer_id: &str) -> String {
        let network: types::conf::Network = serde_wasm_bindgen::from_value(network_js).unwrap();
        helpers::get_peer_wg_config(&network, &Uuid::parse_str(peer_id).unwrap(), false).unwrap()
    }

    #[wasm_bindgen]
    pub fn wg_public_key_from_private_key_wasm(base64_priv: &str) -> String {
        helpers::wg_public_key_from_private_key(&WireGuardKey::from_base64(base64_priv).unwrap()).to_base64()
    }

    #[wasm_bindgen]
    pub fn wg_generate_key_wasm() -> String {
        helpers::wg_generate_key().to_base64()
    }

    #[wasm_bindgen]
    pub fn validate_peer_name_wasm(name: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_name(name));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_address_wasm(address: &str, network_js: JsValue) -> Result<JsValue, JsValue> {
        let network: types::conf::Network = serde_wasm_bindgen::from_value(network_js)?;
        let res = ValidationResultWasm::from(validation::validate_peer_address(address, &network));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_endpoint_wasm(enabled: bool, endpoint: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_endpoint(enabled, endpoint));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_kind_wasm(kind: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_kind(kind));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_icon_wasm(enabled: bool, src: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_icon(enabled, src));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_dns_wasm(enabled: bool, dns: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_dns(enabled, dns));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_mtu_wasm(enabled: bool, mtu: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_mtu(enabled, mtu));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_script_wasm(enabled: bool, script: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_peer_script(enabled, script));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_persistent_keepalive_wasm(enabled: bool, persistent_keepalive: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_conn_persistent_keepalive(enabled, persistent_keepalive));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_allowed_ips_wasm(allowed_ips: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validation::validate_conn_allowed_ips(allowed_ips));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

}
