import { describe, expect, it } from "vitest";
import { notificationText, type AppNotification } from "./notifications";

const base: AppNotification = {
    id: "a",
    kind: "maintenance",
    key: null,
    params: {},
    title: null,
    body: null,
    timestamp: 0,
    read: false,
    link: null,
};

describe("notificationText", () => {
    it("renders a keyed item from the catalog", () => {
        const text = notificationText({ ...base, key: "notifications.maintenance_soon", params: { minutes: "30" } });
        expect(text.title).toBe("Steam maintenance soon");
        expect(text.body).toContain("in about 30 min");
    });

    it("renders feed text passed as a param", () => {
        const text = notificationText({
            ...base,
            key: "notifications.alert_new_more",
            params: { title: "Patch", count: "2" },
        });
        expect(text.body).toBe("Patch (+2 more)");
    });

    it("falls back to raw text on items stored before keys existed", () => {
        expect(notificationText({ ...base, title: "Old", body: "Old body" })).toEqual({
            title: "Old",
            body: "Old body",
        });
    });

    it("never throws on a missing title or body", () => {
        expect(notificationText(base)).toEqual({ title: "", body: "" });
    });
});
