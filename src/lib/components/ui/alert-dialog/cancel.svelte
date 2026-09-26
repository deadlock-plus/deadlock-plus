<script lang="ts">
    import { AlertDialog } from "bits-ui";
    import { cn } from "$lib/utils";
    import { buttonVariants } from "$lib/components/ui/button.svelte";

    let { class: className, ...rest }: AlertDialog.CancelProps = $props();
</script>

<AlertDialog.Cancel class={cn(buttonVariants({ variant: "outline" }), className)} {...rest} />
