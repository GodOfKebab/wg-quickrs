'use strict';

export default class WireGuardHelper {

    static getPeerConfig(network, peerId) {
        const peer = network.peers[peerId];
        
        let conf = `[Interface]
PrivateKey = ${peer.private_key}
Address = ${peer.address}/24
${peer.mobility === 'static' ? `ListenPort = ${peer.endpoint.toString().split(':')[1]}` : 'DEL'}
${peer.dns.enabled ? `DNS = ${peer.dns.value}` : 'DEL'}
${peer.mtu.enabled ? `MTU = ${peer.mtu.value}` : 'DEL'}
${peer.scripts.pre_up.enabled ? `pre_up = ${peer.scripts.pre_up.value}` : 'DEL'}
${peer.scripts.post_up.enabled ? `post_up = ${peer.scripts.post_up.value}` : 'DEL'}
${peer.scripts.pre_down.enabled ? `pre_down = ${peer.scripts.pre_down.value}` : 'DEL'}
${peer.scripts.post_down.enabled ? `post_down = ${peer.scripts.post_down.value}` : 'DEL'}\n`.replaceAll('DEL\n', '');

        for (const [connectionPeers, connectionDetails] of Object.entries(network.connections)) {
            if (!connectionPeers.includes(peerId)) continue;
            if (!connectionDetails.enabled) continue;

            let otherPeerId = '';
            let allowedIPsThisPeer = '';
            if (connectionPeers.split('*')[0] === peerId) {
                otherPeerId = connectionPeers.split('*')[1];
                allowedIPsThisPeer = connectionDetails.allowed_ips_a_to_b;
            } else {
                otherPeerId = connectionPeers.split('*')[0];
                allowedIPsThisPeer = connectionDetails.allowed_ips_b_to_a;
            }

            conf += `
# Peer: ${network.peers[otherPeerId].name} (${otherPeerId})
[Peer]
PublicKey = ${network.peers[otherPeerId].public_key}
PresharedKey = ${connectionDetails.pre_shared_key}
AllowedIPs = ${allowedIPsThisPeer}
${connectionDetails.persistent_keepalive.enabled ? `PersistentKeepalive = ${connectionDetails.persistentKeepalive.value}` : 'DEL'}\n`.replaceAll('DEL\n', '');

            // Add the Endpoint line if known TODO: get roaming endpoints as well
            if (network.peers[otherPeerId].mobility === 'static') {
                conf += `Endpoint = ${network.peers[otherPeerId].endpoint}\n`;
            }
        }

        return conf;
    }

    static downloadPeerConfig(network, peerId) {
        const peerConfigFileContents = WireGuardHelper.getPeerConfig(network, peerId);
        const peerConfigFileName = network.peers[peerId].name.replace(/[^a-zA-Z0-9_=+.-]/g, '-').replace(/(-{2,}|-$)/g, '-').replace(/-$/, '').substring(0, 32);

        const element = document.createElement('a');
        element.setAttribute('href', `data:text/plain;charset=utf-8,${encodeURIComponent(peerConfigFileContents)}`);
        element.setAttribute('download', `${peerConfigFileName}.conf`);

        element.style.display = 'none';
        document.body.appendChild(element);

        element.click();

        document.body.removeChild(element);
    }

    static checkField(fieldName, fieldVariable) {
        const ret = {status: false, msg: ""};
        // check peerId
        if (fieldName === 'peerId') {
            ret.status = fieldVariable.match('^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$');
            if (!ret.status) ret.msg = "peerId needs to follow uuid4 standards"
            return ret;
        }

        // check name
        if (fieldName === 'name') {
            ret.status = fieldVariable.length > 0;
            if (!ret.status) ret.msg = "name cannot be empty";
            return ret;
        }

        // TODO: check subnet
        // TODO: check to see if a duplicate exists
        if (fieldName === 'address') {
            ret.status = fieldVariable.match('^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$');
            if (!ret.status) ret.msg = "address is not IPv4";
            return ret;
        }

        // check mobility
        if (fieldName === 'mobility') {
            ret.status = fieldVariable === 'static' || fieldVariable === 'roaming';
            if (!ret.status) ret.msg = "mobility is invalid (needs to be either 'static' or 'roaming')";
            return ret;
        }

        // check endpoint
        if (fieldName === 'endpoint') {
            ret.status = fieldVariable.match('^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?):(0|6[0-5][0-5][0-3][0-5]|[1-5][0-9][0-9][0-9][0-9]|[1-9][0-9]{0,3})$');
            ret.status ||= fieldVariable.match('^(((?!\\-))(xn\\-\\-)?[a-z0-9\\-_]{0,61}[a-z0-9]{1,1}\\.)*(xn\\-\\-)?([a-z0-9\\-]{1,61}|[a-z0-9\\-]{1,30})\\.[a-z]{2,}:(0|6[0-5][0-5][0-3][0-5]|[1-5][0-9][0-9][0-9][0-9]|[1-9][0-9]{0,3})$');
            if (!ret.status) ret.msg = "endpoint is not IPv4 nor an FQDN";
            return ret;
        }

        // check dns
        if (fieldName === 'dns') {
            ret.status = fieldVariable.enabled === true || fieldVariable.enabled === false;
            ret.status &&= !(fieldVariable.enabled === true && !fieldVariable.value.toString().match('^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))(,(|\\s)*((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)))*$'));
            if (!ret.status) ret.msg = "DNS is invalid";
            return ret;
        }

        // check mtu
        if (fieldName === 'mtu') {
            ret.status = fieldVariable.enabled === true || fieldVariable.enabled === false;
            ret.status &&= !(fieldVariable.enabled === true && !(fieldVariable.value > 0 && fieldVariable.value < 65536));
            if (!ret.status) ret.msg = "MTU is invalid";
            return ret;
        }

        // check script
        if (fieldName === 'script') {
            ret.status = fieldVariable.enabled === true || fieldVariable.enabled === false;
            if ((typeof fieldVariable.value === 'string' || fieldVariable.value instanceof String)) {
                ret.status &&= fieldVariable.value.match('^.*;\\s*$') !== null;
            }
            if (!ret.status) ret.msg = "script needs to end with a semicolon";
            return ret;
        }

        // check scripts
        if (fieldName === 'scripts') {
            ret.status = true;
            for (const scriptField of ['pre_up', 'post_up', 'pre_down', 'post_down']) {
                if (Object.keys(fieldVariable).includes(scriptField)) {
                    if (fieldVariable[scriptField].enabled) {
                        const _ret = WireGuardHelper.checkField('script', fieldVariable[scriptField]);
                        ret.status &&= _ret.status;
                        if (!_ret.status) ret.msg = _ret.msg;
                    }
                } else {
                    ret.status = false;
                    ret.msg = `'scripts' must include '${scriptField}'`;
                    return ret;
                }
            }
            return ret;
        }

        // // check peer count
        // if (fieldName === 'peerCount') {
        //     return fieldVariable.length > 0;
        // }
        //
        // // check allowedIPs
        // if (fieldName === 'allowedIPs') {
        //     return fieldVariable.match('^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\/(3[0-2]|2[0-9]|1[0-9]|[0-9]))(,(|\\s)*((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\/(3[0-2]|2[0-9]|1[0-9]|[0-9])))*$');
        // }
        //
        // // check allowedIPs
        // if (fieldName === 'persistent_keepalive') {
        //     return fieldVariable.match('^([0-9][0-9]|[0-9])$');
        // }

        return {status: false, msg: "field doesn't exist"};
    }

    static getConnectionId(peer1, peer2) {
        if (peer1.localeCompare(peer2, 'en') === 1) return `${peer1}*${peer2}`;
        return `${peer2}*${peer1}`;
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
