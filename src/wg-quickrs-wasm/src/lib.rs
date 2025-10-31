pub mod types;
pub mod validation;
pub mod helpers;
pub mod macros;

// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
mod wasm {
    use serde::{Deserialize, Serialize};
    use serde_wasm_bindgen;
    use wasm_bindgen::prelude::*;
    use uuid::Uuid;
    use crate::helpers;
    use crate::validation::network::*;
    use crate::validation::error::*;
    use crate::types::network::{Network, WireGuardKey};

    #[derive(Serialize, Deserialize)]
    struct ValidationResultWasm {
        failed: bool,
        message: String,
    }

    impl<T> From<Result<T, ValidationError>> for ValidationResultWasm {
        fn from(result: Result<T, ValidationError>) -> Self {
            match result {
                Ok(_) => Self { failed: false, message: String::new() },
                Err(e) => Self { failed: true, message: e.to_string() },
            }
        }
    }

    #[wasm_bindgen]
    pub fn get_peer_wg_config_wasm(network_js: JsValue, peer_id: &str) -> String {
        let network: Network = serde_wasm_bindgen::from_value(network_js).unwrap();
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
        let res = ValidationResultWasm::from(parse_and_validate_peer_name(name));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_address_wasm(address: &str, network_js: JsValue) -> Result<JsValue, JsValue> {
        let network: Network = serde_wasm_bindgen::from_value(network_js)?;
        let res = ValidationResultWasm::from(parse_and_validate_peer_address(address, &network));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_endpoint_wasm(endpoint: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_endpoint(endpoint));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_kind_wasm(kind: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_kind(kind));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_icon_wasm(src: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_icon_src(src));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_dns_wasm(dns: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_dns_addresses(dns));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_mtu_wasm(mtu: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_mtu_value(mtu));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_script_wasm(script: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_script(script));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_persistent_keepalive_wasm(persistent_keepalive: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_conn_persistent_keepalive_period(persistent_keepalive));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_allowed_ips_wasm(allowed_ips: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_conn_allowed_ips(allowed_ips));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

}
