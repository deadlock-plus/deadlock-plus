<script lang="ts">
    import type { HTMLSelectAttributes } from "svelte/elements";
    import { cn } from "$lib/core/utils";

    type Props = HTMLSelectAttributes & {
        class?: string;
    };

    let { class: className, children, ...rest }: Props = $props();
</script>

<!-- The open list is drawn by the OS from the option styles, so options need explicit colours. -->
<select
    class={cn(
        "h-9 rounded-md border border-input bg-card px-2 text-sm text-foreground shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 [&_option]:bg-popover [&_option]:text-popover-foreground",
        className,
    )}
    {...rest}
>
    {@render children?.()}
</select>
