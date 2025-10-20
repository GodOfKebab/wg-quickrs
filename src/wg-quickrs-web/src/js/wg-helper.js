'use strict';

import {
    check_field_enabled_value_frontend,
    check_internal_address,
    check_field_str_frontend,
    get_connection_id_frontend,
    get_peer_wg_config_frontend,
    wg_public_key_from_private_key_frontend,
    wg_generate_key_frontend
} from '../../pkg/wg_quickrs_wasm.js';

export default class WireGuardHelper {

    static getPeerConfig(network, peerId, version) {
        return get_peer_wg_config_frontend(network, peerId, version);
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
            if (fieldName === 'address')
                return check_internal_address(fieldVariable, network)
            else
                return check_field_str_frontend(fieldName, fieldVariable);
        else if (fieldVariable.enabled !== undefined && fieldVariable.value !== undefined)
            return check_field_enabled_value_frontend(fieldName, fieldVariable);
        else
            return { status: false, message: `Invalid field type for ${fieldName}` };
    }

    static getConnectionId(peer1, peer2) {
        return get_connection_id_frontend(peer1, peer2);
    }

    static getConnectionPeers(connectionId) {
        return {a: connectionId.split('*')[0], b: connectionId.split('*')[1]};
    }

    static wg_public_key_from_private_key(private_key) {
        return wg_public_key_from_private_key_frontend(private_key);
    }

    static wg_generate_key() {
        return wg_generate_key_frontend();
    }

}
