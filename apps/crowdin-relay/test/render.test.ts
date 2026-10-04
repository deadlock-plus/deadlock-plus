import { describe, expect, it } from "vitest";
import { render } from "../src/render";
import fileTranslated from "./fixtures/file-translated.json";
import suggestionAdded from "./fixtures/suggestion-added.json";
import suggestionApproved from "./fixtures/suggestion-approved.json";

interface Component {
    type: number;
    content?: string;
    url?: string;
    components?: Component[];
    accessory?: Component;
}

function container(message: ReturnType<typeof render>): Component {
    return message.components[0] as Component;
}

function texts(message: ReturnType<typeof render>): string[] {
    const out: string[] = [];
    const walk = (components: Component[]) => {
        for (const c of components) {
            if (c.type === 10 && c.content !== undefined) out.push(c.content);
            if (c.components) walk(c.components);
        }
    };
    walk(container(message).components ?? []);
    return out;
}

function links(message: ReturnType<typeof render>): string[] {
    const out: string[] = [];
    for (const c of container(message).components ?? []) {
        if (c.accessory?.url) out.push(c.accessory.url);
    }
    return out;
}

function clone<T>(value: T): T {
    return structuredClone(value);
}

describe("message envelope", () => {
    const message = render(fileTranslated);

    it("uses the Components V2 flag and no content or embeds", () => {
        expect(message.flags).toBe(32768);
        expect(message).not.toHaveProperty("content");
        expect(message).not.toHaveProperty("embeds");
    });

    it("blocks all mentions", () => {
        expect(message.allowed_mentions).toEqual({ parse: [] });
    });

    it("sets the username and no avatar", () => {
        expect(message.username).toBe("Crowdin");
        expect(message).not.toHaveProperty("avatar_url");
    });

    it("wraps everything in one container", () => {
        expect(message.components).toHaveLength(1);
        expect(container(message).type).toBe(17);
    });
});

describe("file events", () => {
    it("renders header, project, language and file path", () => {
        const all = texts(render(fileTranslated));
        expect(all[0]).toBe("### File translated");
        expect(all).toContain("**Project**\nDeadlock+");
        expect(all).toContain("**Language**\nSpanish (`es-ES`)");
        expect(all).toContain("**File**\n`/locales/en.json`");
    });

    it("links the header to the editor", () => {
        expect(links(render(fileTranslated))).toEqual(["https://crowdin.com/editor/deadlock-plus/all/en-es"]);
    });

    it("omits the editor link when the language editor code is missing", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        delete event.targetLanguage.editorCode;
        expect(links(render(event))).toEqual([]);
    });

    it("omits the editor link when the project identifier is missing", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        delete event.file.project.identifier;
        expect(links(render(event))).toEqual([]);
    });

    it("skips the language when the event has none", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        event.event = "file.added";
        delete event.targetLanguage;
        const all = texts(render(event));
        expect(all.some((t) => t.startsWith("**Language**"))).toBe(false);
        expect(all[0]).toBe("### File added");
    });

    it("never sends blank text", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        event.file.path = "   ";
        event.file.project.name = "";
        const all = texts(render(event));
        expect(all.every((t) => t.trim() !== "")).toBe(true);
        expect(all.some((t) => t.startsWith("**File**"))).toBe(false);
        expect(all.some((t) => t.startsWith("**Project**"))).toBe(false);
    });

    it("clips every text value to 1000 characters", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        event.file.path = "/" + "a".repeat(5000);
        const file = texts(render(event)).find((t) => t.startsWith("**File**"))!;
        const value = file.slice("**File**\n".length);
        expect(value.length).toBeLessThanOrEqual(1000);
        expect(value.endsWith("…`")).toBe(true);
    });

    it("keeps the closing code fence when the path is clipped", () => {
        const event = clone(fileTranslated) as Record<string, any>;
        event.file.path = "/" + "a".repeat(5000);
        const file = texts(render(event)).find((t) => t.startsWith("**File**"))!;
        expect(file.endsWith("`")).toBe(true);
    });
});

describe("suggestion events", () => {
    it("renders language, key, source and translation", () => {
        const all = texts(render(suggestionAdded));
        expect(all[0]).toBe("### Suggestion added");
        expect(all).toContain("**Project**\nDeadlock+");
        expect(all).toContain("**Language**\nSpanish (`es-ES`)");
        expect(all).toContain("**Key**\n`home.greeting`");
        expect(all).toContain("**Source**\nNot all videos are shown to users.");
        expect(all).toContain("**Translation**\nLos videos no se muestran a todos los usuarios.");
        expect(all).toContain("**Author**\nJohn Smith");
    });

    it("links to the string in the editor", () => {
        expect(links(render(suggestionAdded))).toEqual(["https://crowdin.com/translate/deadlock-plus/44/en-es#2814"]);
    });

    it("falls back to a built editor link when the string has no url", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        delete event.translation.string.url;
        expect(links(render(event))).toEqual(["https://crowdin.com/editor/deadlock-plus/all/en-es"]);
    });

    it("omits the link when there is neither a url nor identifier and editor code", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        delete event.translation.string.url;
        delete event.translation.targetLanguage.editorCode;
        expect(links(render(event))).toEqual([]);
    });

    it("falls back to the identifier when the key is missing", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        delete event.translation.string.key;
        event.translation.string.identifier = "fallback.id";
        expect(texts(render(event))).toContain("**Key**\n`fallback.id`");
    });

    it("falls back to the username when there is no full name", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        event.translation.user.fullName = "";
        expect(texts(render(event))).toContain("**Author**\njohn_smith");
    });

    it("adds the reviewer for approved suggestions", () => {
        const all = texts(render(suggestionApproved));
        expect(all[0]).toBe("### Suggestion approved");
        expect(all).toContain("**Reviewer**\nRae Viewer");
    });

    it.each([
        ["suggestion.added", "### Suggestion added"],
        ["suggestion.updated", "### Suggestion updated"],
        ["suggestion.deleted", "### Suggestion deleted"],
        ["suggestion.approved", "### Suggestion approved"],
        ["suggestion.disapproved", "### Suggestion disapproved"],
    ])("headlines %s", (name, header) => {
        const event = clone(suggestionApproved) as Record<string, any>;
        event.event = name;
        expect(texts(render(event))[0]).toBe(header);
    });

    it("clips source and translation separately", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        event.translation.text = "x".repeat(3000);
        event.translation.string.text = "y".repeat(3000);
        const all = texts(render(event));
        const translation = all.find((t) => t.startsWith("**Translation**"))!;
        const source = all.find((t) => t.startsWith("**Source**"))!;
        expect(translation.slice("**Translation**\n".length).length).toBe(1000);
        expect(source.slice("**Source**\n".length).length).toBe(1000);
    });

    it("never splits a surrogate pair when clipping", () => {
        const event = clone(suggestionAdded) as Record<string, any>;
        event.translation.text = "😀".repeat(1500);
        const translation = texts(render(event)).find((t) => t.startsWith("**Translation**"))!;
        const value = translation.slice("**Translation**\n".length);
        expect(value.length).toBeLessThanOrEqual(1000);
        expect(() => encodeURIComponent(value)).not.toThrow();
    });
});

describe("humanised event names", () => {
    it.each([
        ["file.approved", "### File approved"],
        ["task.status_changed", "### Task status changed"],
        ["stringComment.created", "### String comment created"],
        ["preTranslation.completed", "### Pre translation completed"],
    ])("%s", (name, header) => {
        expect(texts(render({ event: name }))[0]).toBe(header);
    });
});

describe("unknown events", () => {
    it("renders a header with the event name and the project", () => {
        const all = texts(render({ event: "project.built", project: { name: "Deadlock+" } }));
        expect(all).toEqual(["### Project built", "**Project**\nDeadlock+"]);
    });

    it("renders just the header when there is no project", () => {
        expect(texts(render({ event: "task.added" }))).toEqual(["### Task added"]);
    });

    it("survives a payload with no event name", () => {
        expect(texts(render({}))).toEqual(["### Crowdin event"]);
    });

    it("survives non-object input", () => {
        expect(texts(render(null))).toEqual(["### Crowdin event"]);
        expect(texts(render("nope"))).toEqual(["### Crowdin event"]);
    });

    it("has no link", () => {
        expect(links(render({ event: "project.built", project: { name: "x", identifier: "x" } }))).toEqual([]);
    });
});

describe("structure", () => {
    it("puts a divider after the header and uses a section only with a link", () => {
        const withLink = container(render(fileTranslated)).components!;
        expect(withLink[0]!.type).toBe(9);
        expect(withLink[0]!.accessory).toMatchObject({ type: 2, style: 5, label: "Open in editor" });
        expect(withLink[1]!.type).toBe(14);

        const without = container(render({ event: "task.added" })).components!;
        expect(without[0]).toEqual({ type: 10, content: "### Task added" });
    });
});
