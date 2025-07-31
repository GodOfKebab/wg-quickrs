'use strict';

export default class API {
    static async call({method, path, headers, body}) {
        const res = await fetch(`${import.meta.env.VITE_API_FETCH_URL_PREFIX}${path}`, {
            method,
            headers,
            body: body
                ? JSON.stringify(body)
                : undefined,
        });

        if (res.status === 204) {
            return undefined;
        }

        const json = await res.json();

        if (!res.ok) {
            throw new Error(json.error || res.statusText);
        }

        return json;
    }


    static async get_version() {
        return API.call({
            method: 'get',
            path: `/version`,
            headers: {}
        });
    }

    static async get_summary(url_encoded_params) {
        return API.call({
            method: 'get',
            path: `/api/network/summary${url_encoded_params}`,
            headers: {'Content-Type': 'application/x-www-form-urlencoded'}
        });
    }

    static async get_public_private_keys() {
        return API.call({
            method: 'get',
            path: `/api/wireguard/public_private_keys`,
            headers: {}
        });
    }

    static async get_pre_shared_key() {
        return API.call({
            method: 'get',
            path: `/api/wireguard/pre_shared_key`,
            headers: {}
        });
    }

    static async patch_network_config(change_sum) {
        return API.call({
            method: 'patch',
            path: `/api/network/config`,
            headers: {},
            body: change_sum
        });
    }

    static async get_lease_id_address() {
        return API.call({
            method: 'get',
            path: `/api/network/lease/id-address`,
            headers: {},
        });
    }
    
    static async post_wireguard_server_status(body) {
        return this.call({
            method: 'post',
            path: '/api/wireguard/server/status',
            headers: {"Content-Type": "application/json"},
            body: body
        });
    }

}