<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLAttributes } from "svelte/elements";
    import { cn } from "$lib/core/utils";

    type Props = Omit<HTMLAttributes<HTMLElement>, "title"> & {
        title: string;
        subtitle?: string | Snippet;
        actions?: Snippet;
        size?: "md" | "lg";
        class?: string;
    };

    let { title, subtitle, actions, size = "md", class: className, ...rest }: Props = $props();
</script>

<header
    class={cn("flex items-start justify-between", size === "lg" ? "flex-wrap gap-3" : "gap-4", className)}
    {...rest}
>
    <div>
        <h1 class={size === "lg" ? "text-3xl" : "text-2xl"}>{title}</h1>
        {#if typeof subtitle === "function"}
            <p class={cn("text-muted-foreground", size === "lg" ? "mt-1 text-base" : "text-sm")}>
                {@render subtitle()}
            </p>
        {:else if subtitle}
            <p class={cn("text-muted-foreground", size === "lg" ? "mt-1 text-base" : "text-sm")}>{subtitle}</p>
        {/if}
    </div>
    {#if actions}
        <div class="flex shrink-0 items-center gap-2">
            {@render actions()}
        </div>
    {/if}
</header>
