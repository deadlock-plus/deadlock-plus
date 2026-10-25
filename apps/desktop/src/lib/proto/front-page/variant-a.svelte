<script lang="ts">
    import { formatDate, formatNumber } from "$lib/core/i18n.svelte";
    import { clock, compact, kda, ordinal, teamName, type FrontPage } from "./fixture";
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

    const [banner, lead, second, third, ...rest] = $derived(data.stories);
    const briefs = $derived(rest.slice(0, 6));
    const date = $derived(
        formatDate(data.startTime * 1000, { weekday: "long", year: "numeric", month: "long", day: "numeric" }),
    );
    const spark = $derived.by(() => {
        const pts = data.weather.points;
        const max = Math.max(...pts);
        const min = Math.min(...pts);
        return pts
            .map(
                (v, i) =>
                    `${((i / (pts.length - 1)) * 100).toFixed(1)},${(22 - ((v - min) / (max - min || 1)) * 20).toFixed(1)}`,
            )
            .join(" ");
    });
</script>

<Takeover width={1280} height={830} angle={-1.2} {tossAt} {still} {onopen} {onclose}>
    {#snippet paper()}
        <article class="a">
            <header class="mast">
                <div class="ear">
                    <b>Match Edition</b>
                    <span>No. {data.matchId}</span>
                    <span>Vol. {new Date(data.startTime * 1000).getFullYear()}</span>
                </div>
                <h1>Deadlock+ Dispatch</h1>
                <div class="ear right">
                    <b>Weather</b>
                    <span>{data.weather.forecast}, {data.weather.avg} ms</span>
                    <span class="mock">mock ping data</span>
                </div>
            </header>
            <div class="dateline">
                <span>{date}</span>
                <span
                    >{data.winner ? `${teamName(data.winner)} win` : "No result"} · {data.mode} · {clock(
                        data.durationS,
                    )}</span
                >
                <span>Free with Deadlock+</span>
            </div>

            {#if banner}
                <section class="banner">
                    <p class="kicker">{banner.kicker}</p>
                    <h2>{banner.headline}</h2>
                    <p class="deck">{banner.body}</p>
                </section>
            {/if}

            <div class="grid">
                <section class="col lead">
                    <figure>
                        <Halftone src={data.art} alt={data.artHero} class="photo" dot={4.5} />
                        <figcaption>
                            {data.artHero}{data.own ? `, ${kda(data.own)}` : ""}. Hero art: Valve.
                        </figcaption>
                    </figure>
                    {#if lead}
                        <p class="kicker">{lead.kicker}</p>
                        <h3>{lead.headline}</h3>
                        <p class="copy">{lead.body}</p>
                    {/if}
                </section>

                <section class="col">
                    {#each [second, third] as story (story?.kind)}
                        {#if story}
                            <article class="story">
                                <p class="kicker">{story.kicker}</p>
                                <h3>{story.headline}</h3>
                                <p class="copy">{story.body}</p>
                            </article>
                        {/if}
                    {/each}
                    <h4 class="section">In brief</h4>
                    <ul class="briefs">
                        {#each briefs as b (b.kind)}
                            <li><b>{b.kicker}.</b> {b.body}</li>
                        {/each}
                    </ul>
                </section>

                <section class="col">
                    <h4 class="section">The critics</h4>
                    <ol class="critics">
                        {#each data.ratings as r (r.slot)}
                            <li class:own={r.own}>
                                <span class="place">{ordinal(r.grade?.place ?? 0)}</span>
                                <span class="letter">{r.grade?.letter}</span>
                                <span class="who"><b>{r.hero}</b> {r.name}</span>
                            </li>
                        {/each}
                    </ol>
                    <p class="note">Deadlock+ grades, 1st to {data.ratings.length}th.</p>

                    <div class="weather">
                        <h4 class="section">Weather</h4>
                        <p class="forecast">{data.weather.forecast}</p>
                        <svg viewBox="0 0 100 24" preserveAspectRatio="none" aria-hidden="true">
                            <polyline points={spark} />
                        </svg>
                        <dl>
                            <div>
                                <dt>Avg</dt>
                                <dd>{data.weather.avg} ms</dd>
                            </div>
                            <div>
                                <dt>Worst</dt>
                                <dd>{data.weather.worst} ms</dd>
                            </div>
                            <div>
                                <dt>Jitter</dt>
                                <dd>{data.weather.jitter} ms</dd>
                            </div>
                        </dl>
                        <p class="mock">Mock ping series: the demo has no recording.</p>
                    </div>
                </section>

                <section class="col">
                    <h4 class="section">Box score</h4>
                    <table class="box">
                        <thead>
                            <tr
                                ><th class="hero">Hero</th><th>K</th><th>D</th><th>A</th><th>NW</th><th>Dmg</th><th
                                    >Heal</th
                                ></tr
                            >
                        </thead>
                        {#each data.teams as team (team.team)}
                            <tbody class={team.team}>
                                <tr class="team">
                                    <td colspan="7">
                                        {team.name}
                                        <span
                                            >{team.kills} kills · {compact(team.souls)} NW{team.won
                                                ? " · Win"
                                                : ""}</span
                                        >
                                    </td>
                                </tr>
                                {#each team.lines as l (l.slot)}
                                    <tr class:own={l.own}>
                                        <td class="hero">{l.hero}</td>
                                        <td>{l.kills}</td>
                                        <td>{l.deaths}</td>
                                        <td>{l.assists}</td>
                                        <td>{compact(l.souls)}</td>
                                        <td>{compact(l.damage)}</td>
                                        <td>{compact(l.healing)}</td>
                                    </tr>
                                {/each}
                            </tbody>
                        {/each}
                    </table>

                    <h4 class="section">Lane report</h4>
                    <ul class="lanes">
                        {#each data.lanes as lane (lane.street)}
                            <li style="--lane: {lane.ink}">
                                <b>{lane.street}</b>
                                <span class="colour">{lane.colour}</span>
                                <span>
                                    {lane.leadAt10 === 0
                                        ? "Even at 10:00"
                                        : `${teamName(lane.leadAt10 > 0 ? "hidden-king" : "archmother")} +${formatNumber(Math.abs(lane.leadAt10))} souls at 10:00`}{#each lane.guardianFalls as g (g.team)}.
                                        {teamName(g.team)} Guardian fell {clock(g.timeS)}{/each}.
                                </span>
                            </li>
                        {/each}
                    </ul>
                </section>
            </div>

            <Objectives marks={data.objectives} durationS={data.durationS} />
        </article>
    {/snippet}
</Takeover>

<style>
    .a {
        position: absolute;
        inset: 0;
        padding: 22px 34px 18px;
        display: flex;
        flex-direction: column;
        font-family: "Retail Demo", sans-serif;
        font-size: 12.5px;
        line-height: 1.3;
        hyphens: auto;
    }

    .mast {
        display: grid;
        grid-template-columns: 200px 1fr 200px;
        align-items: center;
        border-bottom: 1px solid var(--rule);
        padding-bottom: 4px;
    }

    .mast h1 {
        margin: 0;
        text-align: center;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 78px;
        line-height: 1;
        letter-spacing: -0.01em;
        color: var(--ink);
    }

    .ear {
        display: flex;
        flex-direction: column;
        gap: 1px;
        border: 1px solid var(--rule);
        padding: 6px 8px;
        font-family: "Radiance", sans-serif;
        font-size: 11px;
        line-height: 1.25;
    }

    .ear b {
        font-family: "Valve Pulp", sans-serif;
        font-size: 12px;
        letter-spacing: 0.04em;
        text-transform: uppercase;
    }

    .ear.right {
        text-align: right;
    }

    .mock {
        color: var(--spot);
        font-family: "Radiance", sans-serif;
        font-style: italic;
        font-size: 10px;
    }

    .dateline {
        display: flex;
        justify-content: space-between;
        border-bottom: 3px double var(--rule);
        padding: 4px 0;
        font-family: "Radiance", sans-serif;
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
    }

    .banner {
        padding: 8px 0 8px;
        border-bottom: 1px solid var(--rule);
        text-align: center;
    }

    .banner h2 {
        margin: 0;
        font-family: "Valve Pulp", sans-serif;
        font-size: 50px;
        line-height: 1;
        letter-spacing: -0.01em;
        color: var(--ink);
    }

    .banner .deck {
        margin: 6px auto 0;
        max-width: 860px;
        font-family: "Valve Oracle", serif;
        font-style: italic;
        font-weight: 500;
        font-size: 17px;
        line-height: 1.2;
        color: var(--ink-soft);
    }

    .kicker {
        margin: 0 0 2px;
        font-family: "Radiance", sans-serif;
        font-weight: 700;
        font-size: 10px;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--spot);
    }

    .grid {
        flex: 1;
        min-height: 0;
        display: grid;
        grid-template-columns: 300px 260px 250px 1fr;
        padding-top: 10px;
    }

    .col {
        min-width: 0;
        padding: 0 14px;
        border-left: 1px solid color-mix(in oklch, var(--rule) 55%, transparent);
    }

    .col:first-child {
        padding-left: 0;
        border-left: 0;
    }

    .col:last-child {
        padding-right: 0;
    }

    figure {
        margin: 0 0 8px;
    }

    .lead :global(.photo) {
        width: 100%;
        height: 250px;
        border: 1px solid var(--rule);
    }

    figcaption {
        margin-top: 3px;
        font-family: "Radiance", sans-serif;
        font-style: italic;
        font-size: 10.5px;
        color: var(--ink-soft);
    }

    h3 {
        margin: 0 0 4px;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 21px;
        line-height: 1.05;
        color: var(--ink);
    }

    .lead h3 {
        font-size: 25px;
    }

    .copy {
        margin: 0;
        text-align: justify;
    }

    .copy::first-letter {
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 2.6em;
        float: left;
        line-height: 0.82;
        margin: 2px 2px 0 0;
    }

    .story + .story {
        margin-top: 10px;
        padding-top: 8px;
        border-top: 1px solid color-mix(in oklch, var(--rule) 55%, transparent);
    }

    .section {
        margin: 10px 0 5px;
        padding: 2px 0;
        border-top: 2px solid var(--rule);
        border-bottom: 1px solid var(--rule);
        font-family: "Valve Pulp", sans-serif;
        font-size: 13px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--ink);
    }

    .col > .section:first-child {
        margin-top: 0;
    }

    .briefs {
        margin: 0;
        padding: 0;
        list-style: none;
        font-size: 11.5px;
        line-height: 1.25;
    }

    .briefs li + li {
        margin-top: 4px;
    }

    .briefs b {
        font-family: "Radiance", sans-serif;
        font-weight: 700;
    }

    .critics {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .critics li {
        display: grid;
        grid-template-columns: 30px 34px 1fr;
        align-items: baseline;
        padding: 1px 0;
        border-bottom: 1px dotted color-mix(in oklch, var(--rule) 45%, transparent);
    }

    .critics .place {
        font-family: "Radiance", sans-serif;
        font-size: 10.5px;
        color: var(--ink-soft);
    }

    .critics .letter {
        font-family: "Valve Pulp", sans-serif;
        font-size: 16px;
        line-height: 1.15;
    }

    .critics .who {
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        font-size: 11.5px;
    }

    .critics .who b {
        font-family: "Radiance", sans-serif;
        font-weight: 700;
    }

    .critics li.own,
    .box tr.own td {
        background: color-mix(in oklch, var(--spot) 12%, transparent);
    }

    .note {
        margin: 3px 0 0;
        font-family: "Radiance", sans-serif;
        font-style: italic;
        font-size: 10px;
        color: var(--ink-soft);
    }

    .weather {
        margin-top: 4px;
    }

    .forecast {
        margin: 0;
        font-family: "Valve Oracle", serif;
        font-weight: 600;
        font-size: 20px;
    }

    .weather svg {
        display: block;
        width: 100%;
        height: 26px;
        margin: 2px 0;
        border-bottom: 1px solid var(--rule);
    }

    .weather polyline {
        fill: none;
        stroke: var(--ink);
        stroke-width: 1.2;
        vector-effect: non-scaling-stroke;
    }

    .weather dl {
        display: flex;
        justify-content: space-between;
        margin: 2px 0 0;
        font-family: "Radiance", sans-serif;
        font-size: 11px;
    }

    .weather dt {
        font-size: 9.5px;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--ink-soft);
    }

    .weather dd {
        margin: 0;
        font-weight: 700;
    }

    .box {
        width: 100%;
        border-collapse: collapse;
        font-family: "Radiance", sans-serif;
        font-size: 11px;
        font-variant-numeric: tabular-nums;
    }

    .box th {
        padding: 0 0 2px;
        font-size: 9.5px;
        font-weight: 700;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        text-align: right;
        color: var(--ink-soft);
    }

    .box td {
        padding: 1px 0;
        text-align: right;
    }

    .box .hero {
        text-align: left;
        padding-left: 3px;
    }

    .box tr.own .hero {
        font-weight: 700;
    }

    .box tr.team td {
        padding: 4px 0 1px;
        text-align: left;
        border-bottom: 1.5px solid var(--team);
        font-family: "Valve Pulp", sans-serif;
        font-size: 11.5px;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--team);
    }

    .box tr.team span {
        float: right;
        font-family: "Radiance", sans-serif;
        font-weight: 600;
        font-size: 10px;
        letter-spacing: 0.04em;
    }

    tbody.hidden-king {
        --team: var(--hk-ink);
    }

    tbody.archmother {
        --team: var(--am-ink);
    }

    .lanes {
        margin: 0;
        padding: 0;
        list-style: none;
        font-size: 11.5px;
        line-height: 1.25;
    }

    .lanes li {
        padding-left: 9px;
        border-left: 3px solid var(--lane);
    }

    .lanes li + li {
        margin-top: 5px;
    }

    .lanes b {
        font-family: "Valve Pulp", sans-serif;
        font-size: 11.5px;
        letter-spacing: 0.04em;
        text-transform: uppercase;
    }

    .lanes .colour {
        font-family: "Radiance", sans-serif;
        font-style: italic;
        font-size: 10.5px;
        color: var(--lane);
        margin-right: 4px;
    }
</style>
