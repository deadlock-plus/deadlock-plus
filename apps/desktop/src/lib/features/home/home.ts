export type Period = "lateNight" | "earlyMorning" | "morning" | "midday" | "afternoon" | "evening" | "night";

// Every template keeps the name as ", {name}" so it can be dropped without leaving stray punctuation.
export const GREETINGS: Record<Period, string[]> = {
    lateNight: [
        "Late night, {name}?",
        "Still up, {name}?",
        "Burning the midnight oil, {name}?",
        "The grind never ends, huh {name}?",
        "One more match, {name}?",
    ],
    earlyMorning: [
        "Early start, {name}?",
        "Up with the sun, {name}?",
        "Morning, {name}. Coffee first?",
        "Rise and grind, {name}.",
    ],
    morning: [
        "Good morning, {name}",
        "Morning, {name}",
        "Fresh day, fresh queue, {name}?",
        "Hello there, {name}",
        "Ready when you are, {name}",
    ],
    midday: [
        "Good afternoon, {name}",
        "Lunch break queue, {name}?",
        "Time for a quick midday match, {name}?",
        "Hello again, {name}",
    ],
    afternoon: [
        "Good afternoon, {name}",
        "Afternoon, {name}",
        "How's the day going, {name}?",
        "Ready for a match, {name}?",
        "Welcome back, {name}",
    ],
    evening: [
        "Good evening, {name}",
        "Evening, {name}",
        "Prime queue time, {name}?",
        "Welcome back, {name}",
        "Winding down or warming up, {name}?",
    ],
    night: [
        "Good evening, {name}",
        "Night session, {name}?",
        "One more before bed, {name}?",
        "Evening, {name}. Let's get going",
        "Still going, {name}?",
    ],
};

export function periodFor(hour: number): Period {
    if (hour < 5) return "lateNight";
    if (hour < 8) return "earlyMorning";
    if (hour < 12) return "morning";
    if (hour < 14) return "midday";
    if (hour < 18) return "afternoon";
    if (hour < 22) return "evening";
    return "night";
}

export function greeting(hour: number, name: string | null, seed: number): string {
    const options = GREETINGS[periodFor(hour)];
    const template = options[seed % options.length];
    return name ? template.replace("{name}", name) : template.replace(", {name}", "").replace("{name}", "");
}

/** Chosen once per app session so the greeting does not change on every visit to Home. */
export const sessionSeed = Math.floor(Math.random() * 1_000_000);

/** Calendar-day label in local time, so a match at 23:50 last night is "Yesterday", not "Today". */
export function relativeDay(thenS: number, nowS: number): string {
    const startOfDay = (s: number) => {
        const d = new Date(s * 1000);
        return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
    };
    const days = Math.round((startOfDay(nowS) - startOfDay(thenS)) / 86_400_000);
    if (days <= 0) return "Today";
    if (days === 1) return "Yesterday";
    return `${days} days ago`;
}

export function isActivePath(pathname: string, href: string): boolean {
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
}
