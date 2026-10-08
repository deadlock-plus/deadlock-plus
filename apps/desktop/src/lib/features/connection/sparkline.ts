export type SparkGeometry = { width: number; height: number; pad: number; min: number; max: number };

export function seriesBounds(series: ReadonlyArray<ReadonlyArray<number | null>>): { min: number; max: number } {
    let lo = Infinity;
    let hi = -Infinity;
    for (const values of series) {
        for (const v of values) {
            if (v === null) continue;
            if (v < lo) lo = v;
            if (v > hi) hi = v;
        }
    }
    if (lo === Infinity) return { min: 0, max: 1 };
    const min = Math.floor(lo - 5);
    const max = Math.ceil(hi + 5);
    return { min: Math.max(0, min), max: Math.max(max, min + 10) };
}

export function sparkPath(values: ReadonlyArray<number | null>, g: SparkGeometry): string {
    const xStep = (g.width - g.pad * 2) / Math.max(values.length - 1, 1);
    const yScale = (g.height - g.pad * 2) / (g.max - g.min);
    const parts: string[] = [];
    let pen = false;
    for (let i = 0; i < values.length; i++) {
        const v = values[i];
        if (v === null) {
            pen = false;
            continue;
        }
        const x = g.pad + i * xStep;
        const y = g.pad + (g.max - v) * yScale;
        parts.push(`${pen ? "L" : "M"}${x.toFixed(1)} ${y.toFixed(1)}`);
        pen = true;
    }
    return parts.join(" ");
}
