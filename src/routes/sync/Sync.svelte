<script lang="ts">
	import { untrack } from 'svelte';
	import { btStore } from '@/state';
	import { matchAllDevices, initSelections, type SyncSelections } from '@/components/bluetooth/syncing/matching';
	import DeviceMatchList from '@/components/bluetooth/syncing/DeviceMatchList.svelte';
	import SyncActionBar from '@/components/bluetooth/syncing/SyncActionBar.svelte';

	let matchResult = $derived(matchAllDevices(btStore.state.windows, btStore.state.linux));

	let selections: SyncSelections = $state(new Map());

	// Re-initialize selections when match data changes, preserving existing user choices
	$effect(() => {
		// Track matchResult (the dependency we care about)
		const fresh = initSelections(matchResult);

		// Read current selections without creating a dependency
		const current = untrack(() => selections);
		const merged: SyncSelections = new Map();

		for (const [key, freshSel] of fresh) {
			const existing = current.get(key);
			if (existing) {
				merged.set(key, existing);
			} else {
				merged.set(key, freshSel);
			}
		}

		selections = merged;
	});
</script>

<div class="p-4 max-w-4xl mx-auto">
	<DeviceMatchList {matchResult} bind:selections />
</div>

<SyncActionBar {matchResult} {selections} />
