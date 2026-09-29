export function bestPing(values: (number | null | undefined)[]): number | null {
    const replies = values.filter((v): v is number => v != null);
    return replies.length > 0 ? Math.min(...replies) : null;
}
