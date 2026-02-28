<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	let { children } = $props();

	let isHome = $derived(page.url.pathname === '/');
	let isSyncPage = $derived(page.url.pathname.startsWith('/sync'));
</script>
<ModeWatcher defaultMode="dark" />
<div class="app-shell">
	{#if !isHome && !isSyncPage}
		<div class="app-header">
			<button class="back-btn" onclick={() => goto('/')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
				Back to Setup
			</button>
		</div>
	{/if}
	{@render children?.()}
</div>

<style>
	.app-shell {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		background: #09090b;
		color: #fafafa;
		font-family: 'Inter', system-ui, -apple-system, sans-serif;
	}

	.app-header {
		padding: 12px 20px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
	}
</style>
