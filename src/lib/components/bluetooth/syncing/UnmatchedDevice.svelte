<script lang="ts">
	import type { UnmatchedDevice } from './matching';
	import Device from '../Device.svelte';

	let { device: unmatchedDevice }: { device: UnmatchedDevice } = $props();

	let osBadgeClass = $derived(
		unmatchedDevice.os === 'Windows'
			? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
			: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
	);

	let otherOs = $derived(unmatchedDevice.os === 'Windows' ? 'Linux' : 'Windows');
</script>

<div class="border rounded-lg border-l-4 border-l-muted-foreground/30 bg-card text-card-foreground p-3">
	<div class="flex items-center gap-2 mb-1">
		<span class="text-xs px-1.5 py-0.5 rounded font-medium {osBadgeClass}">
			{unmatchedDevice.os}
		</span>
		<span class="text-xs text-muted-foreground">No match on {otherOs}</span>
	</div>
	<Device device={unmatchedDevice.device} />
</div>
