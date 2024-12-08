'use strict';

export default class API {
    async call({method, path, body}) {
        const res = await fetch(`/api${path}`, {
            method,
            headers: {
                'Content-Type': 'application/json',
            },
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

    async getWireGuardStatus() {
        return this.call({
            method: 'get',
            path: '/server/status',
        });
    }
}