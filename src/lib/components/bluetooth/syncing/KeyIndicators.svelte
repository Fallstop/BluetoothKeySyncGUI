<script lang="ts">
	import type { BluetoothDevice } from '#root/bindings';
	import { Check, X } from 'lucide-svelte';

	let {
		device,
		class: className = ''
	}: {
		device: BluetoothDevice;
		class?: string;
	} = $props();
</script>

<div class="flex items-center gap-2 text-xs text-muted-foreground {className}">
	{#if device.link_key}
		<span class="text-green-600 dark:text-green-400">
			<Check class="inline h-3 w-3" /> Link
		</span>
	{:else}
		<span class="text-muted-foreground/50">
			<X class="inline h-3 w-3" /> Link
		</span>
	{/if}
	{#if device.le_data}
		<span class="text-green-600 dark:text-green-400">
			<Check class="inline h-3 w-3" /> LE
		</span>
	{:else}
		<span class="text-muted-foreground/50">
			<X class="inline h-3 w-3" /> LE
		</span>
	{/if}
</div>
