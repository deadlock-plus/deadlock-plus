export interface GameStatus {
    found: boolean;
    path: string | null;
}

export function needsOnboarding(stored: boolean | null | undefined): boolean {
    return stored !== true;
}

export function gameStatus(gameDir: string | null | undefined): GameStatus {
    return gameDir ? { found: true, path: gameDir } : { found: false, path: null };
}
