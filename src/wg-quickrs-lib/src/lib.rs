pub mod types;
pub mod validation;
pub mod helpers;
pub mod macros;

// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::str::FromStr;
    use serde::Serialize;
    use serde_wasm_bindgen;
    use wasm_bindgen::prelude::*;
    use uuid::Uuid;
    use crate::helpers;
    use crate::validation::network::*;
    use crate::validation::error::*;
    use crate::types::network::{Dns, Endpoint, Icon, Mtu, Network, PersistentKeepalive, Script, WireGuardKey};

    #[derive(Serialize)]
    struct ValidationResultWasm<T: Serialize> {
        error: String,
        value: Option<T>,
    }

    impl<T: Serialize> From<Result<T, ValidationError>> for ValidationResultWasm<T> {
        fn from(result: Result<T, ValidationError>) -> Self {
            match result {
                Ok(v) => Self { error: String::new(), value: Some(v) },
                Err(e) => Self { error: e.to_string(), value: None },
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
    pub fn get_connection_id_wasm(peer1: &str, peer2: &str) -> String {
        let peer1_uuid: Uuid = Uuid::from_str(peer1).unwrap();
        let peer2_uuid: Uuid = Uuid::from_str(peer2).unwrap();
        helpers::get_connection_id(peer1_uuid, peer2_uuid).to_string()
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
    pub fn validate_peer_endpoint_wasm(enabled: bool, endpoint: &str) -> Result<JsValue, JsValue> {
        let res = match parse_and_validate_peer_endpoint(endpoint) {
            Ok(v) => {ValidationResultWasm::from(Ok(Endpoint{
                enabled,
                address: v,
            }))}
            Err(e) => ValidationResultWasm{ error: e.to_string(), value: None }
        };
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_kind_wasm(kind: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_peer_kind(kind));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_icon_wasm(enabled: bool, src: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validate_peer_icon(&Icon{ enabled, src: src.to_string() }));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_dns_wasm(enabled: bool, addresses: &str) -> Result<JsValue, JsValue> {
        let addresses_res = parse_and_validate_peer_dns_addresses(addresses);
        let res = match addresses_res {
            Ok(addresses_vec) => {
                ValidationResultWasm::from(validate_peer_dns(&Dns{ enabled, addresses: addresses_vec }))
            }
            Err(e) => ValidationResultWasm{ error: e.to_string(), value: None }
        };
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_mtu_wasm(enabled: bool, mtu: &str) -> Result<JsValue, JsValue> {
        let mtu_val_res = parse_and_validate_peer_mtu_value(mtu);
        let res = match mtu_val_res {
            Ok(value) => {
                ValidationResultWasm::from(validate_peer_mtu(&Mtu{ enabled, value }))
            }
            Err(e) => ValidationResultWasm{ error: e.to_string(), value: None }
        };
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_peer_script_wasm(enabled: bool, script_str: String) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(validate_peer_script(&Script{ enabled, script: script_str }));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_conn_persistent_keepalive_wasm(enabled: bool, persistent_keepalive_period: &str) -> Result<JsValue, JsValue> {
        let pkp_res = parse_and_validate_conn_persistent_keepalive_period(persistent_keepalive_period);
        let res = match pkp_res {
            Ok(period) => {
                ValidationResultWasm::from(validate_conn_persistent_keepalive_period(&PersistentKeepalive{ enabled, period }))
            }
            Err(e) => ValidationResultWasm{ error: e.to_string(), value: None }
        };
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn validate_conn_allowed_ips_wasm(allowed_ips: &str) -> Result<JsValue, JsValue> {
        let res = ValidationResultWasm::from(parse_and_validate_conn_allowed_ips(allowed_ips));
        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

}
