import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import worker from "../src/index";
import fileTranslated from "./fixtures/file-translated.json";

const env = {
    DISCORD_WEBHOOK_URL: "https://discord.com/api/webhooks/123/token",
    PATH_SECRET: "s3cret",
};

function post(path: string, body: unknown, init: RequestInit = {}): Request {
    return new Request(`https://relay.example${path}`, {
        method: "POST",
        body: typeof body === "string" ? body : JSON.stringify(body),
        ...init,
    });
}

const fetchMock = vi.fn<typeof fetch>();

beforeEach(() => {
    fetchMock.mockReset();
    fetchMock.mockResolvedValue(new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetchMock);
});

afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
});

describe("routing", () => {
    it("rejects the wrong path with 404 and does not call Discord", async () => {
        const res = await worker.fetch(post("/wrong", fileTranslated), env);
        expect(res.status).toBe(404);
        expect(fetchMock).not.toHaveBeenCalled();
    });

    it("rejects the root path with 404", async () => {
        const res = await worker.fetch(post("/", fileTranslated), env);
        expect(res.status).toBe(404);
    });

    it("rejects other methods on the right path with 405", async () => {
        const res = await worker.fetch(new Request("https://relay.example/s3cret"), env);
        expect(res.status).toBe(405);
        expect(res.headers.get("allow")).toBe("POST");
        expect(fetchMock).not.toHaveBeenCalled();
    });

    it("reports 404 before 405 on the wrong path", async () => {
        const res = await worker.fetch(new Request("https://relay.example/wrong"), env);
        expect(res.status).toBe(404);
    });

    it("answers 500 when the path secret is not configured", async () => {
        const res = await worker.fetch(post("/", fileTranslated), { ...env, PATH_SECRET: "" });
        expect(res.status).toBe(500);
    });

    it("answers 500 and logs when the webhook url is not a valid url", async () => {
        const error = vi.spyOn(console, "error").mockImplementation(() => {});
        const res = await worker.fetch(post("/s3cret", fileTranslated), { ...env, DISCORD_WEBHOOK_URL: '"nope"' });
        expect(res.status).toBe(500);
        expect(error).toHaveBeenCalledWith("DISCORD_WEBHOOK_URL is not a valid URL");
        expect(fetchMock).not.toHaveBeenCalled();
    });

    it("answers 500 when the webhook url is not configured", async () => {
        const res = await worker.fetch(post("/s3cret", fileTranslated), { ...env, DISCORD_WEBHOOK_URL: "" });
        expect(res.status).toBe(500);
    });
});

describe("body", () => {
    it("rejects invalid JSON with 400", async () => {
        const res = await worker.fetch(post("/s3cret", "{nope"), env);
        expect(res.status).toBe(400);
        expect(fetchMock).not.toHaveBeenCalled();
    });

    it.each([["null"], ["42"], ['"text"'], ["[]"]])("rejects the non-object body %s with 400", async (body) => {
        const res = await worker.fetch(post("/s3cret", body), env);
        expect(res.status).toBe(400);
        expect(fetchMock).not.toHaveBeenCalled();
    });
});

describe("forwarding", () => {
    it("posts the rendered message to Discord with with_components=true", async () => {
        const res = await worker.fetch(post("/s3cret", fileTranslated), env);
        expect(res.status).toBe(204);
        expect(fetchMock).toHaveBeenCalledTimes(1);

        const [url, init] = fetchMock.mock.calls[0]!;
        const target = new URL(String(url));
        expect(target.origin + target.pathname).toBe("https://discord.com/api/webhooks/123/token");
        expect(target.searchParams.get("with_components")).toBe("true");
        expect(init?.method).toBe("POST");
        expect(new Headers(init?.headers).get("content-type")).toBe("application/json");

        const sent = JSON.parse(String(init?.body));
        expect(sent.flags).toBe(32768);
        expect(sent.username).toBe("Crowdin");
    });

    it("keeps query parameters already on the webhook url", async () => {
        await worker.fetch(post("/s3cret", fileTranslated), {
            ...env,
            DISCORD_WEBHOOK_URL: "https://discord.com/api/webhooks/123/token?thread_id=9",
        });
        const target = new URL(String(fetchMock.mock.calls[0]![0]));
        expect(target.searchParams.get("thread_id")).toBe("9");
        expect(target.searchParams.get("with_components")).toBe("true");
    });

    it("answers 502 when Discord rejects the message", async () => {
        fetchMock.mockResolvedValue(new Response('{"message":"bad"}', { status: 400 }));
        vi.spyOn(console, "error").mockImplementation(() => {});
        const res = await worker.fetch(post("/s3cret", fileTranslated), env);
        expect(res.status).toBe(502);
    });

    it("answers 502 when the request to Discord fails", async () => {
        fetchMock.mockRejectedValue(new Error("network down"));
        vi.spyOn(console, "error").mockImplementation(() => {});
        const res = await worker.fetch(post("/s3cret", fileTranslated), env);
        expect(res.status).toBe(502);
    });

    it("never logs the webhook url or the path secret", async () => {
        fetchMock.mockRejectedValue(new Error(`failed for ${env.DISCORD_WEBHOOK_URL}`));
        const error = vi.spyOn(console, "error").mockImplementation(() => {});
        const log = vi.spyOn(console, "log").mockImplementation(() => {});
        await worker.fetch(post("/s3cret", fileTranslated), env);
        const logged = JSON.stringify([...error.mock.calls, ...log.mock.calls]);
        expect(logged).not.toContain("token");
        expect(logged).not.toContain("s3cret");
    });
});

describe("batched bodies", () => {
    const batch = { events: [fileTranslated, { ...fileTranslated, event: "file.added" }] };

    it("posts one message per event", async () => {
        const res = await worker.fetch(post("/s3cret", batch), env);
        expect(res.status).toBe(204);
        expect(fetchMock).toHaveBeenCalledTimes(2);
        const headers = fetchMock.mock.calls.map(([, init]) =>
            JSON.stringify(JSON.parse(String(init?.body)).components[0].components[0]),
        );
        expect(headers[0]).toContain("File translated");
        expect(headers[1]).toContain("File added");
    });

    it("answers 502 when any message is rejected", async () => {
        fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));
        fetchMock.mockResolvedValueOnce(new Response(null, { status: 400 }));
        vi.spyOn(console, "error").mockImplementation(() => {});
        const res = await worker.fetch(post("/s3cret", batch), env);
        expect(res.status).toBe(502);
    });

    it("still posts a single message for an empty events array", async () => {
        const res = await worker.fetch(post("/s3cret", { events: [] }), env);
        expect(res.status).toBe(204);
        expect(fetchMock).toHaveBeenCalledTimes(1);
    });
});
