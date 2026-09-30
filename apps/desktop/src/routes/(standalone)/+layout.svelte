<script lang="ts">
    import Frame from "$lib/shell/frame.svelte";

    let { children } = $props();
</script>

<Frame statusbar={false}>
    {@render children?.()}
</Frame>
