const DEBOUNCE_MS = 250;

/** Debounced search where only the newest request may write results. */
export class PatchSearch<T> {
    query = $state("");
    results = $state<T[]>([]);
    searching = $state(false);

    private token = 0;
    private timer: ReturnType<typeof setTimeout> | undefined;

    constructor(
        private search: (query: string) => Promise<T[]>,
        private onError: (error: unknown) => void,
    ) {}

    input() {
        clearTimeout(this.timer);
        if (!this.query.trim()) {
            this.results = [];
            this.searching = false;
            return;
        }
        this.searching = true;
        this.timer = setTimeout(() => void this.run(), DEBOUNCE_MS);
    }

    clear() {
        clearTimeout(this.timer);
        this.query = "";
        this.results = [];
        this.searching = false;
    }

    private async run() {
        const token = ++this.token;
        try {
            const found = await this.search(this.query);
            if (token === this.token) this.results = found;
        } catch (e) {
            if (token === this.token) this.onError(e);
        } finally {
            if (token === this.token) this.searching = false;
        }
    }
}
