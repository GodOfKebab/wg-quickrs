'use strict';

export default class API {
    async call({method, path, headers, body}) {
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

    async get_summary(url_encoded_params) {
        return this.call({
            method: 'get',
            path: `/summary${url_encoded_params}`,
            headers: {'Content-Type': 'application/x-www-form-urlencoded'}
        });
    }
}