<script lang="ts">
	import { untrack } from 'svelte';
	import { btStore } from '@/state';
	import {
		matchAllDevices,
		initSelections,
		createManualMatch,
		manualPairKey,
		pairKey,
		deviceKey,
		type SyncSelections,
		type SyncDirection,
		type ManualMatch,
		type UnmatchedDevice
	} from '@/components/bluetooth/syncing/matching';
	import DeviceSyncLayout from '@/components/bluetooth/syncing/DeviceSyncLayout.svelte';
	import SyncActionBar from '@/components/bluetooth/syncing/SyncActionBar.svelte';
	import { Download } from 'lucide-svelte';

	let rawMatchResult = $derived(matchAllDevices(btStore.state.windows, btStore.state.linux));

	let dismissedAutoMatches = $state(new Set<string>());
	let manualMatches: ManualMatch[] = $state([]);
	let selections: SyncSelections = $state(new Map());
	let deletions = $state(new Set<string>());

	// Effective match result: auto-matches minus dismissed pairs
	let matchResult = $derived.by(() => {
		const dismissed = dismissedAutoMatches;

		const needsSync = rawMatchResult.needsSync.filter(
			(p) => !dismissed.has(pairKey(p.controllerAddress, p.windowsDevice.address))
		);
		const alreadySynced = rawMatchResult.alreadySynced.filter(
			(p) => !dismissed.has(pairKey(p.controllerAddress, p.windowsDevice.address))
		);

		// Dismissed pairs' devices become unmatched
		const dismissedUnmatched: UnmatchedDevice[] = [];
		for (const p of rawMatchResult.needsSync) {
			if (dismissed.has(pairKey(p.controllerAddress, p.windowsDevice.address))) {
				dismissedUnmatched.push(
					{ device: p.windowsDevice, os: 'Windows', controllerAddress: p.controllerAddress },
					{ device: p.linuxDevice, os: 'Linux', controllerAddress: p.controllerAddress }
				);
			}
		}
		for (const p of rawMatchResult.alreadySynced) {
			if (dismissed.has(pairKey(p.controllerAddress, p.windowsDevice.address))) {
				dismissedUnmatched.push(
					{ device: p.windowsDevice, os: 'Windows', controllerAddress: p.controllerAddress },
					{ device: p.linuxDevice, os: 'Linux', controllerAddress: p.controllerAddress }
				);
			}
		}

		return {
			...rawMatchResult,
			needsSync,
			alreadySynced,
			unmatched: [...rawMatchResult.unmatched, ...dismissedUnmatched]
		};
	});

	// Unpaired = unmatched + needsSync devices, minus those in manualMatches
	// needsSync devices go into the unpaired pool so users manually pick what to sync
	let unpairedDevices = $derived(() => {
		const manualWinAddrs = new Set(
			manualMatches.map((m) => `${m.windowsControllerAddr}/${m.windowsDevice.address}`)
		);
		const manualLinAddrs = new Set(
			manualMatches.map((m) => `${m.linuxControllerAddr}/${m.linuxDevice.address}`)
		);

		const allDevices: UnmatchedDevice[] = [
			...matchResult.unmatched,
			...matchResult.needsSync.flatMap((p) => [
				{ device: p.windowsDevice, os: 'Windows' as const, controllerAddress: p.controllerAddress },
				{ device: p.linuxDevice, os: 'Linux' as const, controllerAddress: p.controllerAddress }
			])
		];

		return allDevices.filter((d) => {
			const key = `${d.controllerAddress}/${d.device.address}`;
			if (d.os === 'Windows') return !manualWinAddrs.has(key);
			return !manualLinAddrs.has(key);
		});
	});

	// Re-initialize selections when match data changes, preserving existing user choices
	$effect(() => {
		const fresh = initSelections(matchResult);
		const current = untrack(() => selections);
		const merged: SyncSelections = new Map();

		// Preserve existing auto-match selections
		for (const [key, freshSel] of fresh) {
			const existing = current.get(key);
			if (existing) {
				merged.set(key, existing);
			} else {
				merged.set(key, freshSel);
			}
		}

		// Preserve manual match selections
		for (const [key, sel] of current) {
			if (key.startsWith('manual/')) {
				merged.set(key, sel);
			}
		}

		selections = merged;
	});

	function handleManualMatch(
		winDevice: UnmatchedDevice,
		linDevice: UnmatchedDevice,
		direction?: SyncDirection
	) {
		const match = createManualMatch(
			winDevice.device,
			linDevice.device,
			winDevice.controllerAddress,
			linDevice.controllerAddress
		);
		manualMatches = [...manualMatches, match];
		selections.set(manualPairKey(match.id), { direction: direction ?? null });
		selections = new Map(selections);
	}

	function handleUnlink(matchId: string) {
		manualMatches = manualMatches.filter((m) => m.id !== matchId);
		const key = manualPairKey(matchId);
		selections.delete(key);
		selections = new Map(selections);
	}

	function handleAutoUnlink(key: string) {
		dismissedAutoMatches = new Set([...dismissedAutoMatches, key]);
		selections.delete(key);
		selections = new Map(selections);
	}

	function handleToggleDelete(device: UnmatchedDevice) {
		const key = deviceKey(device.os, device.controllerAddress, device.device.address);
		const next = new Set(deletions);
		if (next.has(key)) {
			next.delete(key);
		} else {
			next.add(key);
		}
		deletions = next;
	}

	// --- Debug export ---
	function exportDebugData() {
		const debugData = {
			exportedAt: new Date().toISOString(),
			windows: btStore.state.windows,
			linux: btStore.state.linux,
			manualMatches,
			dismissedAutoMatches: [...dismissedAutoMatches],
			selections: Object.fromEntries(selections)
		};

		const json = JSON.stringify(debugData, null, 2);
		const blob = new Blob([json], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `bt-sync-debug-${new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-')}.json`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
</script>

<div class="p-4 max-w-4xl mx-auto">
	<DeviceSyncLayout
		{matchResult}
		{manualMatches}
		unpairedDevices={unpairedDevices()}
		{deletions}
		bind:selections
		onmanualmatch={handleManualMatch}
		onunlink={handleUnlink}
		onautounlink={handleAutoUnlink}
		ontoggledelete={handleToggleDelete}
	/>

	{#if btStore.state.windows || btStore.state.linux}
		<div class="mt-6 flex justify-end">
			<button
				class="text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors flex items-center gap-1.5"
				onclick={exportDebugData}
			>
				<Download class="h-3 w-3" />
				Export debug data
			</button>
		</div>
	{/if}
</div>

<SyncActionBar {matchResult} {manualMatches} {selections} {deletions} unpairedDevices={unpairedDevices()} />
