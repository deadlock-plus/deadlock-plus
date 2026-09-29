import { bucket, MIN_SAMPLE, scored, type Bucket, type Session } from "./sessions";
import type { Match } from "./stats";

export type Tone = "better" | "worse" | "same" | "unknown";

export interface Bar {
    label: string;
    games: number;
    winrate: number | null;
    tone: Tone;
    note?: string;
}

export interface Finding {
    id: string;
    title: string;
    headline: string;
    tone: Tone;
    bars: Bar[];
}

const MIN_GAP = 0.08;
const Z = 1.28;
const QUICK_REQUEUE_S = 10 * 60;
const STOP_FROM = 3;
const STOP_TO = 6;

// A gap counts only when it clears both a fixed floor and what chance alone would produce at
// this sample size, so a lucky handful of games never reads as a pattern.
export function judge(b: Bucket, baseline: number | null): Tone {
    if (baseline === null || b.winrate === null || b.games < MIN_SAMPLE) return "unknown";
    const diff = b.winrate - baseline;
    const need = Math.max(MIN_GAP, Z * Math.sqrt((baseline * (1 - baseline)) / b.games));
    if (diff >= need) return "better";
    if (diff <= -need) return "worse";
    return "same";
}

interface Game {
    match: Match;
    index: number;
    lossRun: number;
    prev: Match | null;
}

function gamesOf(sessions: Session[]): Game[] {
    const out: Game[] = [];
    for (const s of sessions) {
        let lossRun = 0;
        let prev: Match | null = null;
        scored(s).forEach((match, i) => {
            out.push({ match, index: i + 1, lossRun, prev });
            lossRun = match.outcome === "loss" ? lossRun + 1 : 0;
            prev = match;
        });
    }
    return out;
}

const bucketOf = (games: Game[]): Bucket => bucket(games.length, games.filter((g) => g.match.outcome === "win").length);

const pct = (v: number | null) => (v === null ? "-" : `${Math.round(v * 100)}%`);
const bar = (label: string, b: Bucket, tone: Tone, note?: string): Bar => ({
    label,
    games: b.games,
    winrate: b.winrate,
    tone,
    note,
});

function afterLosses(games: Game[]): Finding {
    const a = bucketOf(games.filter((g) => g.lossRun >= 2));
    const rest = bucketOf(games.filter((g) => g.lossRun < 2));
    const tone = judge(a, rest.winrate);
    const headline = {
        unknown: `Not enough games after two losses in a row yet (${a.games} of ${MIN_SAMPLE}).`,
        worse: `After two losses in a row you win ${pct(a.winrate)}, against ${pct(rest.winrate)} otherwise.`,
        better: `You do better after two losses in a row: ${pct(a.winrate)} against ${pct(rest.winrate)} otherwise.`,
        same: `Two losses in a row don't change how you play: ${pct(a.winrate)} against ${pct(rest.winrate)} otherwise.`,
    }[tone];
    return {
        id: "after-losses",
        title: "After losses",
        headline,
        tone,
        bars: [bar("After two losses in a row", a, tone), bar("Every other game", rest, "same")],
    };
}

function sessionLength(games: Game[], baseline: number | null): Finding {
    const bars: Bar[] = [];
    for (let n = 1; n <= STOP_TO; n++) {
        const at = games.filter((g) => (n === STOP_TO ? g.index >= n : g.index === n));
        const b = bucketOf(at);
        const deltas = at.map((g) => g.match.rankDelta).filter((d): d is number => d !== null);
        const avg = deltas.length ? Math.round(deltas.reduce((x, y) => x + y, 0) / deltas.length) : null;
        const note = avg === null ? undefined : `${avg > 0 ? "+" : ""}${avg} pts`;
        bars.push(bar(n === STOP_TO ? `Game ${n}+` : `Game ${n}`, b, judge(b, baseline), note));
    }

    let best: { k: number; head: Bucket; tail: Bucket; gap: number } | null = null;
    for (let k = STOP_FROM; k <= STOP_TO; k++) {
        const head = bucketOf(games.filter((g) => g.index < k));
        const tail = bucketOf(games.filter((g) => g.index >= k));
        if (head.games < MIN_SAMPLE || judge(tail, head.winrate) !== "worse") continue;
        const gap = head.winrate! - tail.winrate!;
        if (!best || gap > best.gap) best = { k, head, tail, gap };
    }
    if (best) {
        return {
            id: "session-length",
            title: "Where to stop",
            headline: `Your results drop after game ${best.k - 1}: ${pct(best.head.winrate)} up to then, ${pct(best.tail.winrate)} from game ${best.k}.`,
            tone: "worse",
            bars,
        };
    }
    const longGames = games.filter((g) => g.index >= 4).length;
    const enough = longGames >= MIN_SAMPLE;
    return {
        id: "session-length",
        title: "Where to stop",
        headline: enough
            ? "Your results hold up as a session gets longer."
            : `Not enough long sessions yet (${longGames} games past game 3, ${MIN_SAMPLE} needed).`,
        tone: enough ? "same" : "unknown",
        bars,
    };
}

function afterLossSplit(
    games: Game[],
    id: string,
    title: string,
    split: (g: Game) => boolean,
    labels: [string, string],
    text: { better: string; worse: string; same: string },
): Finding {
    const after = games.filter((g) => g.lossRun >= 1 && g.prev);
    const yes = bucketOf(after.filter(split));
    const no = bucketOf(after.filter((g) => !split(g)));
    const tone = no.games < MIN_SAMPLE ? "unknown" : judge(yes, no.winrate);
    const headline =
        tone === "unknown"
            ? `Not enough games after a loss yet (${yes.games} and ${no.games}, ${MIN_SAMPLE} of each needed).`
            : text[tone].replace("{yes}", pct(yes.winrate)).replace("{no}", pct(no.winrate));
    return { id, title, headline, tone, bars: [bar(labels[0], yes, tone), bar(labels[1], no, "same")] };
}

const BLOCKS: { label: string; phrase: string; from: number; to: number }[] = [
    { label: "Morning", phrase: "in the morning", from: 6, to: 11 },
    { label: "Afternoon", phrase: "in the afternoon", from: 12, to: 17 },
    { label: "Evening", phrase: "in the evening", from: 18, to: 23 },
    { label: "Late night", phrase: "during the late night", from: 0, to: 5 },
];

const localOf = (m: Match, tzOffsetMin: number) => m.startTime + tzOffsetMin * 60;
const mod = (n: number, d: number) => ((n % d) + d) % d;

function timeOfDay(games: Game[], baseline: Bucket, tzOffsetMin: number): Finding {
    const rows = BLOCKS.map((blk) => {
        const b = bucketOf(
            games.filter((g) => {
                const hour = Math.floor(mod(localOf(g.match, tzOffsetMin), 86_400) / 3600);
                return hour >= blk.from && hour <= blk.to;
            }),
        );
        return { blk, b, tone: judge(b, baseline.winrate) };
    });
    const gapOf = (r: (typeof rows)[number]) => Math.abs((r.b.winrate ?? 0) - (baseline.winrate ?? 0));
    const best = rows.filter((r) => r.tone === "better").sort((a, b) => gapOf(b) - gapOf(a))[0];
    const worst = rows.filter((r) => r.tone === "worse").sort((a, b) => gapOf(b) - gapOf(a))[0];
    const known = rows.some((r) => r.tone !== "unknown");

    let headline: string;
    if (best && worst) {
        headline = `You do best ${best.blk.phrase} (${pct(best.b.winrate)}) and worst ${worst.blk.phrase} (${pct(worst.b.winrate)}), against ${pct(baseline.winrate)} overall.`;
    } else if (best) {
        headline = `You do best ${best.blk.phrase}: ${pct(best.b.winrate)} against ${pct(baseline.winrate)} overall.`;
    } else if (worst) {
        headline = `You do worst ${worst.blk.phrase}: ${pct(worst.b.winrate)} against ${pct(baseline.winrate)} overall.`;
    } else {
        headline = known
            ? "No part of the day clearly stands out for you."
            : `Not enough games in each part of the day yet (${MIN_SAMPLE} each needed).`;
    }
    return {
        id: "time-of-day",
        title: "Time of day",
        headline,
        tone: best && worst ? "same" : worst ? "worse" : best ? "better" : known ? "same" : "unknown",
        bars: rows.map((r) => bar(r.blk.label, r.b, r.tone)),
    };
}

function dayType(games: Game[], tzOffsetMin: number): Finding {
    // 1970-01-01 was a Thursday, so day 0 maps to weekday index 4 (Sunday = 0).
    const isWeekend = (g: Game) => {
        const dow = mod(Math.floor(localOf(g.match, tzOffsetMin) / 86_400) + 4, 7);
        return dow === 0 || dow === 6;
    };
    const weekend = bucketOf(games.filter(isWeekend));
    const weekday = bucketOf(games.filter((g) => !isWeekend(g)));
    const tone = weekday.games < MIN_SAMPLE ? "unknown" : judge(weekend, weekday.winrate);
    const headline = {
        unknown: `Not enough games on weekdays and weekends yet (${weekday.games} and ${weekend.games}, ${MIN_SAMPLE} of each needed).`,
        better: `You do better on weekends: ${pct(weekend.winrate)} against ${pct(weekday.winrate)} on weekdays.`,
        worse: `You do better on weekdays: ${pct(weekday.winrate)} against ${pct(weekend.winrate)} on weekends.`,
        same: `Weekdays and weekends make no clear difference: ${pct(weekday.winrate)} and ${pct(weekend.winrate)}.`,
    }[tone];
    return {
        id: "day-type",
        title: "Weekdays and weekends",
        headline,
        tone,
        bars: [bar("Weekdays", weekday, "same"), bar("Weekends", weekend, tone)],
    };
}

export function buildFindings(sessions: Session[], tzOffsetMin: number): { baseline: Bucket; findings: Finding[] } {
    const games = gamesOf(sessions);
    const baseline = bucketOf(games);
    return {
        baseline,
        findings: [
            afterLosses(games),
            sessionLength(games, baseline.winrate),
            afterLossSplit(
                games,
                "swap-hero",
                "Switch or stay",
                (g) => g.prev!.heroId !== g.match.heroId,
                ["Switched hero", "Same hero"],
                {
                    better: "After a loss, switching heroes works better for you: {yes} against {no} when you stay on the same hero.",
                    worse: "After a loss, staying on the same hero works better for you: {no} against {yes} when you switch.",
                    same: "Switching heroes after a loss or staying makes no clear difference: {yes} and {no}.",
                },
            ),
            afterLossSplit(
                games,
                "requeue",
                "Requeue speed",
                (g) => g.match.startTime - (g.prev!.startTime + g.prev!.durationS) < QUICK_REQUEUE_S,
                ["Queued within 10 minutes", "Waited longer"],
                {
                    better: "Queueing straight back up after a loss goes better for you: {yes} against {no} when you wait longer.",
                    worse: "Waiting a bit after a loss goes better for you: {no} against {yes} when you queue straight back up.",
                    same: "Queueing straight back up or waiting after a loss makes no clear difference: {yes} and {no}.",
                },
            ),
            timeOfDay(games, baseline, tzOffsetMin),
            dayType(games, tzOffsetMin),
        ],
    };
}
