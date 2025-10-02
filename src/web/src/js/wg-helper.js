'use strict';

import {check_field_str_frontend, check_field_enabled_value_frontend, get_connection_id_frontend, get_peer_wg_config_frontend} from '../../pkg/rust_wasm.js';

export default class WireGuardHelper {

    static getPeerConfig(network, peerId, version) {
        return get_peer_wg_config_frontend(network, peerId, version);
    }

    static downloadPeerConfig(network, peerId, version) {
        const peerConfigFileContents = WireGuardHelper.getPeerConfig(network, peerId, version);
        const peerConfigFileName = network.peers[peerId].name.replace(/[^a-zA-Z0-9_=+.-]/g, '-').replace(/(-{2,}|-$)/g, '-').replace(/-$/, '').substring(0, 32);

        const element = document.createElement('a');
        element.setAttribute('href', `data:text/plain;charset=utf-8,${encodeURIComponent(peerConfigFileContents)}`);
        element.setAttribute('download', `${this.network.identifier}-${peerConfigFileName}.conf`);

        element.style.display = 'none';
        document.body.appendChild(element);

        element.click();

        document.body.removeChild(element);
    }

    static checkField(fieldName, fieldVariable) {
        if (typeof fieldVariable === 'string')
            return JSON.parse(check_field_str_frontend(fieldName, fieldVariable));
        else if (fieldVariable.enabled !== undefined && fieldVariable.value !== undefined)
            return JSON.parse(check_field_enabled_value_frontend(fieldName, JSON.stringify(fieldVariable)));
        else
            return false;
    }

    static getConnectionId(peer1, peer2) {
        return get_connection_id_frontend(peer1, peer2);
    }

    static getConnectionPeers(connectionId) {
        return {a: connectionId.split('*')[0], b: connectionId.split('*')[1]};
    }

    static getNextAvailableAddress(network) {
        const takenAddresses = Object.values(network.peers).map(p => p.address);
        const [ip, cidr] = network.subnet.split('/');
        const startIPv4 = WireGuardHelper.IPv4ToBinary(ip) & WireGuardHelper.cidrToBinary(cidr);
        for (let i = 0; i < 2 ** (32 - parseInt(cidr, 10)); i++) {
            const possibleIPv4 = WireGuardHelper.binaryToIPv4(startIPv4 + i);
            if (!possibleIPv4.endsWith('.0')
                && !possibleIPv4.endsWith('.255')
                && !takenAddresses.includes(possibleIPv4)) {
                return possibleIPv4;
            }
        }
        return null;
    }

    static cidrToBinary(cidr) {
        let binary = 0xFFFFFFFF;
        for (let i = 0; i < 32 - cidr; i++) {
            binary -= 1 << i;
        }
        return binary;
    }

    static IPv4ToBinary(ipv4) {
        let binary = 0;
        for (const ipv4Element of ipv4.split('.')) {
            binary <<= 8;
            binary += parseInt(ipv4Element, 10);
        }
        return binary;
    }

    static binaryToIPv4(binary) {
        const ipv4List = [];
        for (let i = 0; i < 4; i++) {
            ipv4List.push(`${binary & 0xFF}`);
            binary >>= 8;
        }
        return ipv4List.reverse().join('.');
    }

}
