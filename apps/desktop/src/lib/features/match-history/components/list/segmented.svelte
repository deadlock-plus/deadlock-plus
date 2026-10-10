<script lang="ts" generics="V extends string">
    let {
        options,
        value,
        label,
        disabled = false,
        onselect,
    }: {
        options: readonly { value: V; label: string }[];
        value: V;
        label: string;
        disabled?: boolean;
        onselect: (value: V) => void;
    } = $props();
</script>

<div
    class="inline-flex items-center gap-0.5 rounded-md border border-border bg-muted/50 p-0.5 {disabled
        ? 'opacity-50'
        : ''}"
    role="group"
    aria-label={label}
>
    {#each options as o (o.value)}
        <button
            type="button"
            {disabled}
            aria-pressed={value === o.value}
            class="h-7 rounded px-2.5 text-xs font-medium whitespace-nowrap transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:cursor-not-allowed {value ===
            o.value
                ? 'bg-primary text-primary-foreground shadow-sm'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
            onclick={() => onselect(o.value)}
        >
            {o.label}
        </button>
    {/each}
</div>
