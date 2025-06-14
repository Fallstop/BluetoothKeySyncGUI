<script lang="ts">
    let { children } = $props();
</script>

<code class="border-accent border-1 font-bold text-sm bg-accent-2 p-0.5 mx-0.5 rounded-sm">
    {@render children?.()}
</code>
