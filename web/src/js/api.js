'use strict';

export default class API {
    static async call({method, path, headers, body}) {
        const res = await fetch(`/api${path}`, {
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


    static async get_summary(url_encoded_params) {
        return API.call({
            method: 'get',
            path: `/summary${url_encoded_params}`,
            headers: {'Content-Type': 'application/x-www-form-urlencoded'}
        });
    }

    static async get_public_private_key() {
        return API.call({
            method: 'get',
            path: `/public_private_key`,
            headers: {}
        });
    }

    static async get_pre_shared_key() {
        return API.call({
            method: 'get',
            path: `/pre_shared_key`,
            headers: {}
        });
    }
}