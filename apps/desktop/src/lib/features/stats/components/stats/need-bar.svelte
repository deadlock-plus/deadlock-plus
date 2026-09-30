<script lang="ts">
    let { label, value, need }: { label: string; value: number; need: number } = $props();
</script>

<div>
    <div class="flex justify-between text-sm">
        <span class="text-muted-foreground">{label}</span>
        <span class={value >= need ? "text-primary" : ""}>{value} / {need}</span>
    </div>
    <div class="mt-1 h-2 overflow-hidden rounded-full bg-muted">
        <div class="h-full rounded-full bg-primary" style:width="{Math.min(100, (value / need) * 100)}%"></div>
    </div>
</div>
