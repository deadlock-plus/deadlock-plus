<script lang="ts">
    let {
        src,
        alt,
        dot = 5,
        position = "50% 20%",
        class: className = "",
    }: { src: string | null; alt: string; dot?: number; position?: string; class?: string } = $props();
</script>

<div class="halftone {className}" style="--dot: {dot}px;" role="img" aria-label={alt}>
    {#if src}
        <img {src} alt="" style="object-position: {position};" />
    {/if}
</div>

<style>
    /* Grey dots multiplied over a blurred greyscale picture, then a hard contrast turns them into ink dots
       sized by how dark the picture is; multiply drops the white onto the paper. */
    .halftone {
        position: relative;
        overflow: hidden;
        background: #fff;
        filter: grayscale(1) contrast(12);
        mix-blend-mode: multiply;
    }

    .halftone img {
        display: block;
        width: 100%;
        height: 100%;
        object-fit: cover;
        filter: blur(0.7px) brightness(1.12) contrast(1.15);
    }

    .halftone::after {
        content: "";
        position: absolute;
        inset: -50%;
        background: radial-gradient(closest-side, #5e5e5e, #fff) 0 0 / var(--dot) var(--dot);
        mix-blend-mode: multiply;
        transform: rotate(45deg);
    }
</style>
