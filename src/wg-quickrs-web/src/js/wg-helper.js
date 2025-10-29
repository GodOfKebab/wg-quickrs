'use strict';

import {
    get_peer_wg_config_wasm,
    wg_public_key_from_private_key_wasm,
    wg_generate_key_wasm,
    validate_peer_name_wasm,
    validate_peer_address_wasm,
    validate_peer_endpoint_wasm,
    validate_peer_kind_wasm,
    validate_peer_icon_wasm,
    validate_peer_dns_wasm,
    validate_peer_mtu_wasm,
    validate_peer_script_wasm,
    validate_persistent_keepalive_wasm,
    validate_allowed_ips_wasm,
} from '../../pkg/wg_quickrs_wasm.js';

export default class WireGuardHelper {

    static getPeerConfig(network, peerId) {
        return get_peer_wg_config_wasm(network, peerId);
    }

    static downloadPeerConfig(network, peerId, version) {
        const peerConfigFileContents = WireGuardHelper.getPeerConfig(network, peerId, version);
        const peerConfigFileName = network.peers[peerId].name.replace(/[^a-zA-Z0-9_=+.-]/g, '-').replace(/(-{2,}|-$)/g, '-').replace(/-$/, '').substring(0, 32);

        const element = document.createElement('a');
        element.setAttribute('href', `data:text/plain;charset=utf-8,${encodeURIComponent(peerConfigFileContents)}`);
        element.setAttribute('download', `${network.identifier}-${peerConfigFileName}.conf`);

        element.style.display = 'none';
        document.body.appendChild(element);

        element.click();

        document.body.removeChild(element);
    }

    static checkField(fieldName, fieldVariable, network=null) {
        if (typeof fieldVariable === 'string')
            if (fieldName === 'name')
                return validate_peer_name_wasm(fieldVariable)
            else if (fieldName === 'address')
                return validate_peer_address_wasm(fieldVariable, network)
            else if (fieldName === 'kind')
                return validate_peer_kind_wasm(fieldVariable)
            else if (fieldName === 'allowed_ips')
                return validate_allowed_ips_wasm(fieldVariable)
            else
                return { failed: true, message: `Invalid field key '${fieldName}'` };
        else if (fieldVariable.enabled !== undefined && fieldVariable.value !== undefined)
            if (fieldName === 'endpoint')
                return validate_peer_endpoint_wasm(fieldVariable.enabled, fieldVariable.value)
            else if (fieldName === 'icon')
                return validate_peer_icon_wasm(fieldVariable.enabled, fieldVariable.value)
            else if (fieldName === 'dns')
                return validate_peer_dns_wasm(fieldVariable.enabled, fieldVariable.value)
            else if (fieldName === 'mtu')
                return validate_peer_mtu_wasm(fieldVariable.enabled, fieldVariable.value)
            else if (fieldName === 'script')
                return validate_peer_script_wasm(fieldVariable.enabled, fieldVariable.value)
            else if (fieldName === 'persistent_keepalive')
                return validate_persistent_keepalive_wasm(fieldVariable.enabled, fieldVariable.value)
            else
                return { failed: true, message: `Invalid field key '${fieldName}'` };
        else
            return { failed: true, message: `Invalid field type for ${fieldName}` };
    }

    static getConnectionId(peer1, peer2) {
        return `${peer1}*${peer2}`;
    }

    static getConnectionPeers(connectionId) {
        return {a: connectionId.split('*')[0], b: connectionId.split('*')[1]};
    }

    static wg_public_key_from_private_key(private_key) {
        return wg_public_key_from_private_key_wasm(private_key);
    }

    static wg_generate_key() {
        return wg_generate_key_wasm();
    }

}
