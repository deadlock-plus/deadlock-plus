<script lang="ts">
    import { formatDate, formatNumber } from "$lib/core/i18n.svelte";
    import { clock, compact, kda, ordinal, type FrontPage } from "./fixture";
    import Halftone from "./halftone.svelte";
    import Objectives from "./objectives.svelte";
    import Takeover from "./takeover.svelte";

    let {
        data,
        tossAt,
        still,
        onopen,
        onclose,
    }: { data: FrontPage; tossAt: number | null; still: boolean; onopen: () => void; onclose: () => void } = $props();

    const [banner, lead, ...rest] = $derived(data.stories);
    const briefs = $derived(rest.slice(0, 2));
    const date = $derived(
        formatDate(data.startTime * 1000, { weekday: "short", month: "short", day: "numeric", year: "numeric" }),
    );
    const result = $derived(
        data.own && data.winner
            ? data.own.team === data.winner
                ? "Win"
                : "Loss"
            : data.winner
              ? "Final"
              : "No result",
    );
    const critics = $derived.by(() => {
        const top3 = data.ratings.slice(0, 3);
        const own = data.ratings.find((r) => r.own);
        return own && !top3.includes(own) ? [...top3, own] : top3;
    });
</script>

<Takeover width={720} height={960} angle={1.6} {tossAt} {still} {onopen} {onclose}>
    {#snippet paper()}
        <article class="b">
            <header class="mast">
                <h1>Deadlock+ Dispatch</h1>
                <div class="ear">
                    <b>Weather</b>
                    <span>{data.weather.forecast}</span>
                    <span>{data.weather.avg} ms avg · {data.weather.worst} worst</span>
                    <span class="mock">mock ping data</span>
                </div>
            </header>
            <div class="dateline">
                <span>{date}</span>
                <span>Match Edition · No. {data.matchId}</span>
                <span>{data.mode} · {clock(data.durationS)}</span>
            </div>

            {#if banner}
                <section class="banner">
                    <p class="tag">{banner.kicker}</p>
                    <h2>{banner.headline}</h2>
                </section>
            {/if}

            <div class="lead">
                <figure>
                    <Halftone src={data.art} alt={data.artHero} class="photo" dot={6} />
                    <span class="stamp">{result}</span>
                    <figcaption>{data.artHero}{data.own ? `, ${kda(data.own)}` : ""}. Hero art: Valve.</figcaption>
                </figure>
                <div class="story">
                    {#if banner}
                        <p class="deck">{banner.body}</p>
                    {/if}
                    {#if lead}
                        <h3>{lead.headline}</h3>
                        <p class="copy">{lead.body}</p>
                    {/if}
                    <div class="score">
                        {#each data.teams as team, i (team.team)}
                            {#if i === 1}<span class="dash">–</span>{/if}
                            <div class={team.team}>
                                <b>{team.kills}</b>
                                <span>{team.name}</span>
                            </div>
                        {/each}
                    </div>
                    <p class="score-note">Kills</p>
                </div>
            </div>

            <section class="box">
                {#each data.teams as team (team.team)}
                    <table class={team.team}>
                        <caption>{team.name}{team.won ? " · Win" : ""}</caption>
                        <tbody>
                            {#each team.lines as l (l.slot)}
                                <tr class:own={l.own}>
                                    <td class="hero">{l.hero}</td>
                                    <td>{kda(l)}</td>
                                    <td class="nw">{compact(l.souls)}</td>
                                    <td class="grade">{l.grade?.letter ?? ""}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {/each}
            </section>

            <div class="bottom">
                <section>
                    <h4>The critics</h4>
                    <ol class="critics">
                        {#each critics as r (r.slot)}
                            <li class:own={r.own}>
                                <span class="letter">{r.grade?.letter}</span>
                                <span><b>{r.hero}</b><br />{ordinal(r.grade?.place ?? 0)} of {data.ratings.length}</span
                                >
                            </li>
                        {/each}
                    </ol>
                </section>
                <section>
                    <h4>Lane report</h4>
                    <ul class="lanes">
                        {#each data.lanes as lane (lane.street)}
                            <li style="--lane: {lane.ink}">
                                <b>{lane.street}</b>
                                {lane.leadAt10 === 0
                                    ? "even at 10:00"
                                    : `${lane.leadAt10 > 0 ? "Hidden King" : "Archmother"} +${formatNumber(Math.abs(lane.leadAt10))} at 10:00`}
                            </li>
                        {/each}
                    </ul>
                </section>
                <section>
                    <h4>In brief</h4>
                    <ul class="briefs">
                        {#each briefs as b (b.kind)}
                            <li><b>{b.headline}.</b> {b.body}</li>
                        {/each}
                    </ul>
                </section>
            </div>

            <Objectives marks={data.objectives} durationS={data.durationS} compact />
        </article>
    {/snippet}
</Takeover>

<style>
    .b {
        position: absolute;
        inset: 0;
        padding: 20px 26px 14px;
        display: flex;
        flex-direction: column;
        font-family: "Radiance", sans-serif;
        font-size: 14px;
        line-height: 1.3;
        --obj-head: "Valve Oracle", serif;
        --obj-text: "Retail Demo", sans-serif;
    }

    .mast {
        display: grid;
        grid-template-columns: 1fr 150px;
        gap: 10px;
        align-items: stretch;
    }

    .mast h1 {
        margin: 0;
        padding: 8px 12px 6px;
        background: var(--spot);
        color: var(--paper);
        font-family: "Valve Pulp", sans-serif;
        font-size: 45px;
        line-height: 1;
        letter-spacing: -0.01em;
        white-space: nowrap;
        min-width: 0;
    }

    .ear {
        display: flex;
        flex-direction: column;
        justify-content: center;
        border: 2px solid var(--ink);
        padding: 4px 8px;
        font-size: 11px;
        line-height: 1.2;
    }

    .ear b {
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 14px;
    }

    .mock {
        color: var(--spot);
        font-style: italic;
        font-size: 10px;
    }

    .dateline {
        display: flex;
        justify-content: space-between;
        margin-top: 8px;
        padding: 3px 0;
        border-top: 2px solid var(--ink);
        border-bottom: 2px solid var(--ink);
        font-family: "Retail Demo", sans-serif;
        font-weight: 600;
        font-size: 12px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
    }

    .banner {
        padding: 10px 0 6px;
    }

    .tag {
        display: inline-block;
        margin: 0 0 6px;
        padding: 2px 8px;
        background: var(--ink);
        color: var(--paper);
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 15px;
        letter-spacing: 0.04em;
        text-transform: uppercase;
    }

    .banner h2 {
        margin: 0;
        font-family: "Retail Demo", sans-serif;
        font-weight: 700;
        font-size: 64px;
        line-height: 0.9;
        letter-spacing: -0.025em;
        text-transform: uppercase;
        color: var(--ink);
    }

    .lead {
        display: grid;
        grid-template-columns: 290px 1fr;
        gap: 16px;
        padding: 8px 0 10px;
        border-top: 1px solid var(--ink);
    }

    figure {
        position: relative;
        margin: 0;
    }

    .lead :global(.photo) {
        width: 100%;
        height: 250px;
        border: 2px solid var(--ink);
    }

    .stamp {
        position: absolute;
        right: -12px;
        top: -10px;
        z-index: 1;
        padding: 2px 10px;
        border: 3px solid var(--spot);
        border-radius: 4px;
        color: var(--spot);
        font-family: "Valve Pulp", sans-serif;
        font-size: 30px;
        line-height: 1;
        text-transform: uppercase;
        transform: rotate(-9deg);
        mix-blend-mode: multiply;
        opacity: 0.88;
    }

    figcaption {
        margin-top: 3px;
        font-size: 11px;
        font-style: italic;
        color: var(--ink-soft);
    }

    .deck {
        margin: 0 0 8px;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-style: italic;
        font-size: 18px;
        line-height: 1.15;
    }

    h3 {
        margin: 0 0 3px;
        font-family: "Retail Demo", sans-serif;
        font-weight: 700;
        font-size: 18px;
        line-height: 1.05;
        text-transform: uppercase;
        color: var(--ink);
    }

    .copy {
        margin: 0;
    }

    .score {
        display: flex;
        align-items: flex-end;
        gap: 10px;
        margin-top: 12px;
        padding-top: 8px;
        border-top: 3px solid var(--ink);
    }

    .score div {
        display: flex;
        flex-direction: column;
    }

    .score b {
        font-family: "Valve Pulp", sans-serif;
        font-size: 52px;
        line-height: 0.9;
    }

    .score span {
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 13px;
    }

    .score .dash {
        font-family: "Valve Pulp", sans-serif;
        font-size: 40px;
        line-height: 1.3;
        align-self: flex-start;
    }

    .score .hidden-king {
        color: var(--hk-ink);
    }

    .score .archmother {
        color: var(--am-ink);
    }

    .score-note {
        margin: 0;
        font-size: 10px;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--ink-soft);
    }

    .box {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 18px;
        padding: 6px 0 8px;
        border-top: 3px solid var(--ink);
    }

    .box table {
        width: 100%;
        border-collapse: collapse;
        font-family: "Retail Demo", sans-serif;
        font-size: 12.5px;
        font-variant-numeric: tabular-nums;
    }

    .box caption {
        text-align: left;
        padding-bottom: 2px;
        border-bottom: 2px solid var(--team);
        color: var(--team);
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 14px;
        text-transform: uppercase;
    }

    .box table.hidden-king {
        --team: var(--hk-ink);
    }

    .box table.archmother {
        --team: var(--am-ink);
    }

    .box td {
        padding: 1px 0;
        border-bottom: 1px dotted color-mix(in oklch, var(--ink) 40%, transparent);
        text-align: right;
    }

    .box td.hero {
        text-align: left;
    }

    .box td.nw {
        width: 52px;
        color: var(--ink-soft);
    }

    .box td.grade {
        width: 30px;
        font-family: "Valve Pulp", sans-serif;
    }

    .box tr.own td {
        font-weight: 700;
        background: color-mix(in oklch, var(--spot) 13%, transparent);
    }

    .bottom {
        display: grid;
        grid-template-columns: 1fr 1fr 1.25fr;
        gap: 16px;
        padding-top: 6px;
        border-top: 3px solid var(--ink);
    }

    h4 {
        margin: 0 0 4px;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 16px;
        text-transform: uppercase;
        color: var(--spot);
    }

    .critics,
    .lanes,
    .briefs {
        margin: 0;
        padding: 0;
        list-style: none;
        font-size: 12px;
        line-height: 1.2;
    }

    .critics li {
        display: grid;
        grid-template-columns: 40px 1fr;
        align-items: center;
        padding: 2px 0;
    }

    .critics li.own {
        background: color-mix(in oklch, var(--spot) 13%, transparent);
    }

    .critics .letter {
        font-family: "Valve Pulp", sans-serif;
        font-size: 24px;
        line-height: 1;
    }

    .lanes li {
        padding-left: 7px;
        border-left: 4px solid var(--lane);
    }

    .lanes li + li,
    .briefs li + li {
        margin-top: 6px;
    }

    .lanes b {
        display: block;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 13px;
        text-transform: uppercase;
    }

    .briefs b {
        font-family: "Retail Demo", sans-serif;
        font-weight: 700;
    }
</style>
