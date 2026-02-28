<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
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
	import { Download, RefreshCw } from 'lucide-svelte';
	import { rpc } from '@/api';

	let isRefreshing = $state(false);

	async function refreshLinuxData() {
		if (isRefreshing) return;
		isRefreshing = true;
		try {
			const [response] = await Promise.all([
				rpc.linux.parse_local_config(),
				new Promise((r) => setTimeout(r, 500))
			]);
			if (response.type === 'Success') {
				btStore.state.linux = response.data;
			}
		} finally {
			isRefreshing = false;
		}
	}

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

	// Unpaired = unmatched devices, minus those in manualMatches
	let unpairedDevices = $derived.by(() => {
		const manualWinAddrs = new Set(
			manualMatches.map((m) => `${m.windowsControllerAddr}/${m.windowsDevice.address}`)
		);
		const manualLinAddrs = new Set(
			manualMatches.map((m) => `${m.linuxControllerAddr}/${m.linuxDevice.address}`)
		);

		return matchResult.unmatched.filter((d) => {
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

	function handleSyncComplete() {
		// Clear stale state — the refreshed btStore data will re-derive
		// matchResult correctly, so manual matches, deletions, and dismissed
		// pairs from the old state would be duplicates or dangling references
		manualMatches = [];
		deletions = new Set();
		dismissedAutoMatches = new Set();
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

<div class="gf-root">
	<div class="gradient-mesh"></div>

	<div class="gf-content">
		<!-- Header -->
		<div class="sync-header">
			<button class="back-btn mb-4" onclick={() => goto('/')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
				Back
			</button>
			<div class="sync-title-row">
				<h1 class="sync-title">Device Sync</h1>
				<button class="gf-btn ghost small refresh-btn" onclick={refreshLinuxData} disabled={isRefreshing} title="Refresh Linux Bluetooth data">
					<RefreshCw class="h-4 w-4 {isRefreshing ? 'animate-spin' : ''}" />
				</button>
			</div>
			<p class="sync-tagline">Match and sync Bluetooth pairing keys between systems</p>
		</div>

		<!-- Sync content -->
		{#if isRefreshing}
			<div class="refresh-panel">
				<div class="refresh-bar"><div class="refresh-fill"></div></div>
				<p class="refresh-text">Refreshing Linux Bluetooth data...</p>
			</div>
		{:else}
			<DeviceSyncLayout
				{matchResult}
				{manualMatches}
				unpairedDevices={unpairedDevices}
				{deletions}
				bind:selections
				onmanualmatch={handleManualMatch}
				onunlink={handleUnlink}
				onautounlink={handleAutoUnlink}
				ontoggledelete={handleToggleDelete}
			/>
		{/if}

		{#if btStore.state.windows || btStore.state.linux}
			<div class="debug-row">
				<button class="gf-btn ghost small" onclick={exportDebugData}>
					<Download class="h-3 w-3" />
					Export debug data
				</button>
			</div>
		{/if}
	</div>
</div>

<SyncActionBar {matchResult} {manualMatches} {selections} {deletions} unpairedDevices={unpairedDevices} onsynccomplete={handleSyncComplete} />

<style lang="css">
	/* Header */
	.sync-header {
		margin-bottom: 2rem;
	}

	.sync-title-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.refresh-btn {
		opacity: 0.6;
		transition: opacity 0.15s;
	}

	.refresh-btn:hover:not(:disabled) {
		opacity: 1;
	}

	.sync-title {
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		margin: 0;
		background: linear-gradient(135deg, #fafafa 0%, #a78bfa 50%, #60a5fa 100%);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}

	.sync-tagline {
		color: rgba(250, 250, 250, 0.4);
		font-size: 14px;
		font-weight: 400;
		margin: 0.35rem 0 0;
	}

	/* Refresh loading state */
	.refresh-panel {
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: 12px;
		background: rgba(255, 255, 255, 0.015);
		padding: 48px 16px;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.refresh-bar {
		width: min(280px, 80%);
		height: 3px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 2px;
		overflow: hidden;
		margin-bottom: 12px;
	}

	.refresh-fill {
		width: 30%;
		height: 100%;
		background: linear-gradient(90deg, #a78bfa, #60a5fa);
		border-radius: 2px;
		animation: refresh-sweep 1.5s ease-in-out infinite;
	}

	@keyframes refresh-sweep {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(400%); }
	}

	.refresh-text {
		font-size: 13px;
		color: rgba(250, 250, 250, 0.4);
		margin: 0;
	}

	/* Debug row */
	.debug-row {
		margin-top: 24px;
		display: flex;
		justify-content: flex-end;
	}

</style>
