import { describe, expect, it } from "vitest";
import { cargoVersion, lockVersion, withCargoVersion, withLockVersion } from "./version.mjs";

const cargo = `[workspace]
members = ["app"]

[workspace.package]
version = "0.1.0"

[dependencies]
serde = { version = "1" }

[dev-dependencies]
version = "9.9.9"
`;

const lock = `[[package]]
name = "other"
version = "1.2.3"

[[package]]
name = "deadlock-plus"
version = "0.1.0"
dependencies = []
`;

describe("cargo manifest version", () => {
    it("reads the workspace package version, not a dependency's", () => {
        expect(cargoVersion(cargo)).toBe("0.1.0");
    });

    it("rewrites only the workspace package version", () => {
        const next = withCargoVersion(cargo, "0.2.0");
        expect(cargoVersion(next)).toBe("0.2.0");
        expect(next).toContain('version = "9.9.9"');
        expect(next).toContain('serde = { version = "1" }');
    });
});

describe("lockfile version", () => {
    it("reads and rewrites the entry for the app only", () => {
        expect(lockVersion(lock)).toBe("0.1.0");
        const next = withLockVersion(lock, "0.2.0");
        expect(lockVersion(next)).toBe("0.2.0");
        expect(next).toContain('name = "other"\nversion = "1.2.3"');
    });
});
