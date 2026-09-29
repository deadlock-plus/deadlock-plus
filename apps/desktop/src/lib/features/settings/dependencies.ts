export interface DependencyPackage {
    name: string;
    version: string;
    source: "rust" | "npm";
    url?: string;
}

export interface LicenseGroup {
    title: string;
    text: string | null;
    packages: DependencyPackage[];
}

export interface DependencyLicenses {
    groups: LicenseGroup[];
}

export function summarize(data: DependencyLicenses) {
    const seen = new Map<string, DependencyPackage["source"]>();
    for (const g of data.groups) {
        for (const p of g.packages) seen.set(`${p.source}:${p.name}@${p.version}`, p.source);
    }
    const sources = [...seen.values()];
    return {
        packages: seen.size,
        rust: sources.filter((s) => s === "rust").length,
        npm: sources.filter((s) => s === "npm").length,
        groups: data.groups.length,
    };
}
