<script lang="ts">
	import type { MatchResult, SyncSelections, SyncDirection } from './matching';
	import { pairKey } from './matching';
	import DevicePairRow from './DevicePairRow.svelte';
	import UnmatchedDeviceCard from './UnmatchedDevice.svelte';
	import { slide } from 'svelte/transition';
	import { ChevronDown, AlertCircle, CheckCircle, HelpCircle } from 'lucide-svelte';

	let {
		matchResult,
		selections = $bindable()
	}: {
		matchResult: MatchResult;
		selections: SyncSelections;
	} = $props();

	let needsSyncOpen = $state(true);
	let syncedOpen = $state(false);
	let unmatchedOpen = $state(false);

	let allNeedsSyncEnabled = $derived(
		matchResult.needsSync.length > 0 &&
			matchResult.needsSync.every((pair) => {
				const sel = selections.get(pairKey(pair.controllerAddress, pair.windowsDevice.address));
				return sel?.enabled ?? false;
			})
	);

	function toggleAllNeedsSync() {
		const newValue = !allNeedsSyncEnabled;
		for (const pair of matchResult.needsSync) {
			const key = pairKey(pair.controllerAddress, pair.windowsDevice.address);
			const existing = selections.get(key);
			if (existing) {
				selections.set(key, { ...existing, enabled: newValue });
			}
		}
		selections = new Map(selections);
	}

	function updateEnabled(key: string, enabled: boolean) {
		const existing = selections.get(key);
		if (existing) {
			selections.set(key, { ...existing, enabled });
			selections = new Map(selections);
		}
	}

	function updateDirection(key: string, direction: SyncDirection) {
		const existing = selections.get(key);
		if (existing) {
			selections.set(key, { ...existing, direction });
			selections = new Map(selections);
		}
	}
</script>

<div class="space-y-3">
	<!-- Needs Sync Section -->
	{#if matchResult.needsSync.length > 0}
		<div class="rounded-lg border bg-card">
			<button
				class="w-full flex items-center gap-2 p-3 text-left hover:bg-muted/50 transition-colors rounded-t-lg"
				onclick={() => (needsSyncOpen = !needsSyncOpen)}
			>
				<ChevronDown
					class="h-4 w-4 transition-transform {needsSyncOpen ? '' : '-rotate-90'}"
				/>
				<AlertCircle class="h-4 w-4 text-amber-500" />
				<span class="font-medium text-sm flex-1">
					Needs Sync ({matchResult.needsSync.length})
				</span>
				{#if needsSyncOpen}
					<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
					<label
						class="text-xs text-muted-foreground flex items-center gap-1"
						onclick={(e) => e.stopPropagation()}
						onkeydown={(e) => { if (e.key === ' ') e.stopPropagation(); }}
					>
						<input
							type="checkbox"
							checked={allNeedsSyncEnabled}
							onchange={toggleAllNeedsSync}
							class="rounded border-input h-3.5 w-3.5 accent-primary"
						/>
						Select all
					</label>
				{/if}
			</button>
			{#if needsSyncOpen}
				<div class="px-3 pb-3 space-y-2" transition:slide={{ duration: 200 }}>
					{#each matchResult.needsSync as pair (pairKey(pair.controllerAddress, pair.windowsDevice.address))}
						{@const key = pairKey(pair.controllerAddress, pair.windowsDevice.address)}
						{@const sel = selections.get(key)}
						<DevicePairRow
							{pair}
							enabled={sel?.enabled ?? true}
							direction={sel?.direction ?? pair.recommendedDirection}
							onenabledchange={(v) => updateEnabled(key, v)}
							ondirectionchange={(v) => updateDirection(key, v)}
						/>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Already Synced Section -->
	{#if matchResult.alreadySynced.length > 0}
		<div class="rounded-lg border bg-card">
			<button
				class="w-full flex items-center gap-2 p-3 text-left hover:bg-muted/50 transition-colors rounded-t-lg"
				onclick={() => (syncedOpen = !syncedOpen)}
			>
				<ChevronDown
					class="h-4 w-4 transition-transform {syncedOpen ? '' : '-rotate-90'}"
				/>
				<CheckCircle class="h-4 w-4 text-green-500" />
				<span class="font-medium text-sm flex-1">
					Already Synced ({matchResult.alreadySynced.length})
				</span>
			</button>
			{#if syncedOpen}
				<div class="px-3 pb-3 space-y-2" transition:slide={{ duration: 200 }}>
					{#each matchResult.alreadySynced as pair (pairKey(pair.controllerAddress, pair.windowsDevice.address))}
						<DevicePairRow {pair} readonly />
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Unmatched Devices Section -->
	{#if matchResult.unmatched.length > 0}
		<div class="rounded-lg border bg-card">
			<button
				class="w-full flex items-center gap-2 p-3 text-left hover:bg-muted/50 transition-colors rounded-t-lg"
				onclick={() => (unmatchedOpen = !unmatchedOpen)}
			>
				<ChevronDown
					class="h-4 w-4 transition-transform {unmatchedOpen ? '' : '-rotate-90'}"
				/>
				<HelpCircle class="h-4 w-4 text-muted-foreground" />
				<span class="font-medium text-sm flex-1">
					Unmatched ({matchResult.unmatched.length})
				</span>
			</button>
			{#if unmatchedOpen}
				<div class="px-3 pb-3 space-y-2" transition:slide={{ duration: 200 }}>
					{#each matchResult.unmatched as device (device.os + '/' + device.controllerAddress + '/' + device.device.address)}
						<UnmatchedDeviceCard {device} />
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Empty state -->
	{#if matchResult.needsSync.length === 0 && matchResult.alreadySynced.length === 0 && matchResult.unmatched.length === 0}
		<div class="text-center py-12 text-muted-foreground">
			<p class="text-sm">No devices found. Load both Windows and Linux Bluetooth data first.</p>
		</div>
	{/if}
</div>
