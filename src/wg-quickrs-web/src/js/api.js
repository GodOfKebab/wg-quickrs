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
            method: method.toUpperCase(),
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

        if (!res.ok) {
            throw new Error(`${method} ${path}: ${res.status} ${res.statusText}\n${await res.text()}`);
        }

        const json = await res.json();

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

    async get_network_summary(url_encoded_params) {
        return this.call({
            method: 'get',
            path: `/api/network/summary${url_encoded_params}`,
            headers: {'Content-Type': 'application/x-www-form-urlencoded'}
        });
    }

    async patch_network_config(change_sum) {
        return this.call({
            method: 'patch',
            path: `/api/network/config`,
            body: change_sum
        });
    }

    async post_network_reserve_address() {
        return this.call({
            method: 'post',
            path: `/api/network/reserve/address`,
        });
    }

    async post_wireguard_status(body) {
        return this.call({
            method: 'post',
            path: '/api/wireguard/status',
            headers: {"Content-Type": "application/json"},
            body: body
        });
    }

}
