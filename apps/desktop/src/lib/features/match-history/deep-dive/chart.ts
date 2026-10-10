import { allPlayers, type MatchDetail, type MatchPlayer } from "../detail";

export type Metric = "souls" | "playerDamage";

export interface LinePoint {
    t: number;
    v: number;
}

export interface ChartBox {
    width: number;
    height: number;
    padLeft: number;
    padRight: number;
    padTop: number;
    padBottom: number;
}

export interface ChartDomain {
    tMax: number;
    vMin: number;
    vMax: number;
}

/** Series values are cumulative, so the line starts from zero at the first sample's left. */
export function playerLine(player: MatchPlayer, metric: Metric): LinePoint[] {
    if (player.series.length === 0) return [];
    return [{ t: 0, v: 0 }, ...player.series.map((s) => ({ t: s.timeS, v: s[metric] }))];
}

/** Linear between samples, held at the last value after the final sample. */
export function valueAt(line: readonly LinePoint[], t: number): number {
    if (line.length === 0) return 0;
    if (t <= line[0].t) return line[0].v;
    for (let i = 1; i < line.length; i++) {
        const b = line[i];
        if (t <= b.t) {
            const a = line[i - 1];
            const span = b.t - a.t;
            return span <= 0 ? b.v : a.v + ((b.v - a.v) * (t - a.t)) / span;
        }
    }
    return line[line.length - 1].v;
}

/** Team 0 minus team 1 at every sample time any player has. */
export function teamLeadLine(detail: MatchDetail, metric: Metric): LinePoint[] {
    const lines = detail.teams.map((team) => team.players.map((p) => playerLine(p, metric)));
    const times = new Set<number>();
    for (const team of lines) for (const line of team) for (const p of line) times.add(p.t);
    if (times.size === 0) return [];
    const total = (team: LinePoint[][], t: number) => team.reduce((sum, line) => sum + valueAt(line, t), 0);
    return [...times].sort((a, b) => a - b).map((t) => ({ t, v: total(lines[0], t) - total(lines[1], t) }));
}

export function chartDomain(lines: ReadonlyArray<readonly LinePoint[]>): ChartDomain {
    let tMax = 0;
    let vMin = 0;
    let vMax = 0;
    for (const line of lines) {
        for (const p of line) {
            if (p.t > tMax) tMax = p.t;
            if (p.v < vMin) vMin = p.v;
            if (p.v > vMax) vMax = p.v;
        }
    }
    return { tMax: tMax > 0 ? tMax : 1, vMin, vMax: vMax > vMin ? vMax : vMin + 1 };
}

export function xOf(t: number, dom: ChartDomain, box: ChartBox): number {
    return box.padLeft + (t / dom.tMax) * (box.width - box.padLeft - box.padRight);
}

export function yOf(v: number, dom: ChartDomain, box: ChartBox): number {
    const h = box.height - box.padTop - box.padBottom;
    return box.padTop + ((dom.vMax - v) / (dom.vMax - dom.vMin)) * h;
}

export function linePath(line: readonly LinePoint[], dom: ChartDomain, box: ChartBox): string {
    return line
        .map((p, i) => `${i === 0 ? "M" : "L"}${xOf(p.t, dom, box).toFixed(1)} ${yOf(p.v, dom, box).toFixed(1)}`)
        .join(" ");
}

export function everyPlayerLine(detail: MatchDetail, metric: Metric): Map<number, LinePoint[]> {
    return new Map(allPlayers(detail).map((p) => [p.slot, playerLine(p, metric)]));
}
