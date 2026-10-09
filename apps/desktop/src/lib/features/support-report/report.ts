const pad = (n: number) => String(n).padStart(2, "0");

export function reportFileName(now: Date): string {
    return `deadlock-plus-support-${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}.txt`;
}
