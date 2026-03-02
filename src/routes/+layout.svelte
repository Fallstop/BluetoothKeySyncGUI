<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	import { enableSmoothScroll } from '$lib/smoothScroll';
	import UpdateBanner from '$lib/components/UpdateBanner.svelte';
	let { children } = $props();

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
