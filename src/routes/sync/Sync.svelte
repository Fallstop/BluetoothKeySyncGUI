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
	import { Download, RefreshCw, CheckCircle } from 'lucide-svelte';
	import { rpc } from '@/api';
	import { osColor } from '@/components/bluetooth/syncing/os-theme';

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

	// Device counts for summary strip
	function countDevices(data: typeof btStore.state.windows) {
		if (!data) return 0;
		return data.controllers.reduce((sum, c) => sum + c.devices.length, 0);
	}

	let winDeviceCount = $derived(countDevices(btStore.state.windows));
	let linDeviceCount = $derived(countDevices(btStore.state.linux));
	let matchedCount = $derived(matchResult.alreadySynced.length + matchResult.needsSync.length + manualMatches.length);
	let readyCount = $derived(Array.from(selections.values()).filter((s) => s.direction !== null).length);

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
		selections.set(manualPairKey(match.id), { direction: direction ?? 'win_to_linux' });
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
		manualMatches = [];
		deletions = new Set();
		dismissedAutoMatches = new Set();
	}

	// --- Toast ---
	let toastMessage = $state<string | null>(null);
	let toastTimer: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		return () => {
			if (toastTimer) clearTimeout(toastTimer);
		};
	});

	function showToast(message: string, duration = 2500) {
		toastMessage = message;
		if (toastTimer) clearTimeout(toastTimer);
		toastTimer = setTimeout(() => {
			toastMessage = null;
			toastTimer = null;
		}, duration);
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

		showToast('Debug data exported');
	}
</script>

<div class="gf-root">
	<div class="gradient-mesh"></div>

	<div class="gf-content">
		<!-- Header -->
		<div class="sync-header">
			<div class="header-top-row">
				<button class="back-btn" onclick={() => goto('/')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
					Back
				</button>
				<div class="toolbar">
					<button class="gf-btn ghost small" onclick={refreshLinuxData} disabled={isRefreshing} title="Refresh Linux Bluetooth data">
						<RefreshCw class="h-3.5 w-3.5 {isRefreshing ? 'animate-spin' : ''}" />
						Refresh
					</button>
					{#if btStore.state.windows || btStore.state.linux}
						<button class="gf-btn ghost small" onclick={exportDebugData}>
							<Download class="h-3 w-3" />
							Export
						</button>
					{/if}
				</div>
			</div>
			<h1 class="sync-title">Device Sync</h1>
			<p class="sync-tagline">Match and sync Bluetooth pairing keys between systems</p>
		</div>

		<!-- Summary strip -->
		{#if btStore.state.windows || btStore.state.linux}
			<div class="summary-strip">
				<div class="summary-stat">
					<span class="stat-value" style="color: {osColor('Windows').textColor}">{winDeviceCount}</span>
					<span class="stat-label">Windows</span>
				</div>
				<div class="summary-divider"></div>
				<div class="summary-stat">
					<span class="stat-value" style="color: {osColor('Linux').textColor}">{linDeviceCount}</span>
					<span class="stat-label">Linux</span>
				</div>
				<div class="summary-divider"></div>
				<div class="summary-stat">
					<span class="stat-value">{matchedCount}</span>
					<span class="stat-label">Matched</span>
				</div>
				<div class="summary-divider"></div>
				<div class="summary-stat">
					<span class="stat-value" style="color: #4ade80">{readyCount}</span>
					<span class="stat-label">Ready</span>
				</div>
			</div>
		{/if}

		<!-- Sync content -->
		<div class="sync-content" class:content-refreshing={isRefreshing}>
			{#if isRefreshing}
				<div class="refresh-bar-container">
					<div class="refresh-bar"><div class="refresh-fill"></div></div>
				</div>
			{/if}
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
		</div>
	</div>

	<!-- Toast notification -->
	{#if toastMessage}
		<div class="toast">
			<CheckCircle class="h-4 w-4" />
			<span>{toastMessage}</span>
		</div>
	{/if}
</div>

<SyncActionBar {matchResult} {manualMatches} {selections} {deletions} unpairedDevices={unpairedDevices} onsynccomplete={handleSyncComplete} />

<style lang="css">
	/* Header */
	.sync-header {
		margin-bottom: 1.5rem;
	}

	.header-top-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 6px;
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

	/* Summary strip */
	.summary-strip {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 10px 16px;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.06);
		margin-bottom: 1.25rem;
	}

	.summary-stat {
		display: flex;
		align-items: baseline;
		gap: 6px;
	}

	.stat-value {
		font-size: 16px;
		font-weight: 700;
		color: rgba(250, 250, 250, 0.85);
	}

	.stat-label {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
	}

	.summary-divider {
		width: 1px;
		height: 16px;
		background: rgba(255, 255, 255, 0.08);
	}

	/* Non-blocking refresh */
	.sync-content {
		position: relative;
		transition: opacity 0.2s;
	}

	.sync-content.content-refreshing {
		opacity: 0.6;
		pointer-events: none;
	}

	.refresh-bar-container {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		z-index: 5;
	}

	.refresh-bar {
		width: 100%;
		height: 2px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 1px;
		overflow: hidden;
	}

	.refresh-fill {
		width: 30%;
		height: 100%;
		background: linear-gradient(90deg, #a78bfa, #60a5fa);
		border-radius: 1px;
		animation: refresh-sweep 1.5s ease-in-out infinite;
	}

	@keyframes refresh-sweep {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(400%); }
	}

	/* Toast */
	.toast {
		position: fixed;
		bottom: 80px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 18px;
		border-radius: 10px;
		background: rgba(34, 197, 94, 0.12);
		border: 1px solid rgba(34, 197, 94, 0.25);
		color: #4ade80;
		font-size: 13px;
		font-weight: 500;
		backdrop-filter: blur(12px);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
		z-index: 50;
		animation: toast-in 0.2s ease-out;
	}

	@keyframes toast-in {
		from {
			opacity: 0;
			transform: translateX(-50%) translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) translateY(0);
		}
	}
</style>
