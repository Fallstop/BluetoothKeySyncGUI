<script lang="ts">
  import ThemeToggle from '@/components/ThemeToggle.svelte';
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	let { children } = $props();
</script>
<ModeWatcher />
<div class="min-h-screen">
	<div class="header border-b-2 flex justify-between items-center p-4">
		<h1 class="text-2xl font-bold">Bluetooth Key Sync</h1>
		<div class="flex items-center gap-2">
			<ThemeToggle />
		</div>

	</div>
	{@render children?.()}
</div>