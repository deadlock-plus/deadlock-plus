<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLAttributes } from "svelte/elements";
    import { cn } from "$lib/core/utils";

    type Props = HTMLAttributes<HTMLDivElement> & {
        size?: "md" | "lg";
        class?: string;
        children?: Snippet;
    };

    let { size = "md", class: className, children, ...rest }: Props = $props();
</script>

<div
    class={cn(
        "mx-auto flex min-h-full flex-col",
        size === "lg" ? "max-w-5xl gap-5 px-8 pb-12 pt-8" : "max-w-4xl gap-4 px-6 pb-10 pt-6",
        className,
    )}
    {...rest}
>
    {@render children?.()}
</div>
