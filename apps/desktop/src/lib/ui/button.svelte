<script lang="ts" module>
    import { tv, type VariantProps } from "tailwind-variants";

    export const buttonVariants = tv({
        base: "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-semibold transition-colors disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
        variants: {
            variant: {
                default: "bg-primary text-primary-foreground shadow-sm hover:opacity-90",
                destructive: "bg-destructive text-destructive-foreground shadow-sm hover:opacity-90",
                outline: "border border-input bg-transparent shadow-sm hover:bg-accent hover:text-accent-foreground",
                secondary: "bg-secondary text-secondary-foreground shadow-sm hover:opacity-90",
                ghost: "hover:bg-accent hover:text-accent-foreground",
                link: "text-primary underline-offset-4 hover:underline",
            },
            size: {
                default: "h-9 px-4 py-2 [&_svg]:size-4",
                sm: "h-8 rounded-md px-3 text-xs [&_svg]:size-3.5",
                lg: "h-10 rounded-md px-6 [&_svg]:size-4",
                icon: "h-9 w-9 [&_svg]:size-4",
            },
        },
        defaultVariants: {
            variant: "default",
            size: "default",
        },
    });

    // Only the focus ring and disabled state: for clickable cards and rows that carry their own layout and colours.
    export const unstyledButtonClass =
        "text-left transition-colors disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50";

    export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
    export type ButtonSize = VariantProps<typeof buttonVariants>["size"];
</script>

<script lang="ts">
    import type { HTMLButtonAttributes } from "svelte/elements";
    import { cn } from "$lib/core/utils";

    type Props = HTMLButtonAttributes & {
        variant?: ButtonVariant | "unstyled";
        size?: ButtonSize;
        class?: string;
    };

    let { variant = "default", size = "default", class: className, children, ...rest }: Props = $props();
</script>

<button
    class={cn(variant === "unstyled" ? unstyledButtonClass : buttonVariants({ variant, size }), className)}
    {...rest}
>
    {@render children?.()}
</button>
