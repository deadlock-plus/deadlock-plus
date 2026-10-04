import { render } from "./render";

export interface Env {
    DISCORD_WEBHOOK_URL: string;
    PATH_SECRET: string;
}

function status(code: number, headers?: HeadersInit): Response {
    return new Response(null, { status: code, headers });
}

export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        if (!env.PATH_SECRET || !env.DISCORD_WEBHOOK_URL) return status(500);
        if (new URL(request.url).pathname !== `/${env.PATH_SECRET}`) return status(404);
        if (request.method !== "POST") return status(405, { allow: "POST" });

        let payload: unknown;
        try {
            payload = await request.json();
        } catch {
            return status(400);
        }
        if (typeof payload !== "object" || payload === null || Array.isArray(payload)) return status(400);

        let target: URL;
        try {
            target = new URL(env.DISCORD_WEBHOOK_URL);
        } catch {
            console.error("DISCORD_WEBHOOK_URL is not a valid URL");
            return status(500);
        }
        target.searchParams.set("with_components", "true");

        try {
            const res = await fetch(target, {
                method: "POST",
                headers: { "content-type": "application/json" },
                body: JSON.stringify(render(payload)),
            });
            if (!res.ok) {
                console.error(`Discord rejected the message: ${res.status}`);
                return status(502);
            }
        } catch {
            console.error("Request to Discord failed");
            return status(502);
        }
        return status(204);
    },
};
