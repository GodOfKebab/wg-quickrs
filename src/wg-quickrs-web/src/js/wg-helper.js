'use strict';

import {
    get_peer_wg_config_wasm,
    wg_public_key_from_private_key_wasm,
    wg_generate_key_wasm,
    get_connection_id_wasm,
} from '@/pkg/wg_quickrs_lib.js';
import FastEqual from "fast-deep-equal";

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

    static validateField(fieldName, validator, originalValue, island_change_sum, field_color_lookup, ...validatorArgs) {
        const result = validator(...validatorArgs);

        if (result.error) {
            island_change_sum.errors[fieldName] = result.error;
            return [field_color_lookup["error"], island_change_sum];
        }

        if (!FastEqual(result.value, originalValue)) {
            island_change_sum.changed_fields[fieldName] = result.value;
            return [field_color_lookup["changed"], island_change_sum];
        }

        island_change_sum.changed_fields[fieldName] = null;
        island_change_sum.errors[fieldName] = null;
        return [field_color_lookup["unchanged"], island_change_sum];
    }

    static getConnectionId(peer1, peer2) {
        return get_connection_id_wasm(peer1, peer2);
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

    static get_field_colors(is_new) {
        return {
            unchanged: is_new ? 'enabled:bg-green-200' : 'bg-white',
            changed: is_new ? 'enabled:bg-green-200' : 'enabled:bg-blue-200',
            error: 'enabled:bg-red-200',
        }
    }

    static get_div_colors(is_new) {
        return {
            unchanged: is_new ? 'bg-green-100' : 'bg-green-50',
            changed: is_new ? 'bg-green-100' : 'bg-blue-50',
            error: 'bg-red-50',
        }
    }

}
