'use strict';

export default class API {
    token;
    does_need_auth = false;

    async call({method, path, headers, body}) {
        if (this.does_need_auth) {
            throw new Error(`A valid token required for ${method} ${path}!`);
        }

        headers = headers ? headers : {};
        if (this.token !== '') {
            headers["Authorization"] = `Bearer ${this.token}`;
        }
        const res = await fetch(`${import.meta.env.VITE_API_FETCH_URL_PREFIX}${path}`, {
            method,
            headers: headers,
            body: body
                ? JSON.stringify(body)
                : undefined,
        });

        // get a new token
        if (res.status === 401) {
            console.error(`Unauthorized: ${res.status}`);
            this.does_need_auth = true;
        }

        if (res.status === 204) {
            return undefined;
        }

        const json = await res.json();

        if (!res.ok) {
            throw new Error(json.error || res.statusText);
        }

        return json;
    }

    async update_api_token(password) {
        const token_res = await fetch(`${import.meta.env.VITE_API_FETCH_URL_PREFIX}/api/token`, {
            method: "post",
            body: JSON.stringify({client_id: 'web', password}),
        });
        const token = await token_res.text();
        if (token_res.status === 200) {
            this.does_need_auth = false;
            this.token = token;
        } else {
            throw new Error("Unauthorized access");
        }
    }

    async get_version() {
        return this.call({
            method: 'get',
            path: `/api/version`,
        });
    }

    async get_network_summary(url_encoded_params) {
        return this.call({
            method: 'get',
            path: `/api/network/summary${url_encoded_params}`,
            headers: {'Content-Type': 'application/x-www-form-urlencoded'}
        });
    }

    async get_wireguard_public_private_keys() {
        return this.call({
            method: 'get',
            path: `/api/wireguard/public-private-keys`,
        });
    }

    async get_wireguard_pre_shared_key() {
        return this.call({
            method: 'get',
            path: `/api/wireguard/pre-shared-key`,
        });
    }

    async patch_network_config(change_sum) {
        return this.call({
            method: 'patch',
            path: `/api/network/config`,
            body: change_sum
        });
    }

    async get_network_lease_id_address() {
        return this.call({
            method: 'get',
            path: `/api/network/lease/id-address`,
        });
    }

    async post_wireguard_server_status(body) {
        return this.call({
            method: 'post',
            path: '/api/wireguard/server/status',
            headers: {"Content-Type": "application/json"},
            body: body
        });
    }

}