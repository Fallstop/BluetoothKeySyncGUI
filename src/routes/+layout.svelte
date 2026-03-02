<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { enableSmoothScroll } from '$lib/smoothScroll';
	import UpdateBanner from '$lib/components/UpdateBanner.svelte';
	let { children } = $props();

	let isHome = $derived(page.url.pathname === '/' || page.url.pathname === '/index.html');
	let isSyncPage = $derived(page.url.pathname.startsWith('/sync'));

	$effect(() => {
		return enableSmoothScroll();
	});
</script>
<ModeWatcher defaultMode="dark" />
<UpdateBanner />
<div class="app-shell">
	<svg class="noise-texture" aria-hidden="true">
		<filter id="noise">
			<feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch" />
		</filter>
		<rect width="100%" height="100%" filter="url(#noise)" />
	</svg>
	{#if !isHome && !isSyncPage}
		<div class="app-header">
			<button class="back-btn" onclick={() => goto('/')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
				Back to Setup
			</button>
		</div>
	{/if}
	<div class="page-enter">
		{@render children?.()}
	</div>
</div>

<style lang="css">
	.app-shell {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		background: #09090b;
		color: #fafafa;
		font-family: 'Sora Variable', system-ui, -apple-system, sans-serif;
		position: relative;
	}

	.noise-texture {
		position: fixed;
		inset: 0;
		width: 100%;
		height: 100%;
		opacity: 0.035;
		pointer-events: none;
		z-index: 0;
	}

	.app-header {
		position: relative;
		z-index: 1;
		padding: 12px 20px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
	}

	.page-enter {
		position: relative;
		z-index: 1;
		display: flex;
		flex-direction: column;
		flex: 1;
		animation: page-fade-in 0.2s ease-out;
	}

	@keyframes page-fade-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
