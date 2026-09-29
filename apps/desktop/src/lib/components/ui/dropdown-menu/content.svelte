<script lang="ts">
    import { DropdownMenu } from "bits-ui";
    import { cn } from "$lib/utils";

    let { class: className, sideOffset = 4, align = "end", ...rest }: DropdownMenu.ContentProps = $props();
</script>

<DropdownMenu.Portal>
    <DropdownMenu.Content
        {sideOffset}
        {align}
        class={cn(
            "z-50 min-w-44 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0",
            className,
        )}
        {...rest}
    />
</DropdownMenu.Portal>
