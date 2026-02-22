<script lang="ts">
	import type { MatchedDevicePair, SyncDirection, KeyFieldComparison } from './matching';
	import { describeSyncChanges } from './matching';
	import { Check, X, AlertTriangle, ArrowRight, Minus } from 'lucide-svelte';

	let {
		pair,
		enabled = true,
		direction = 'win_to_linux' as SyncDirection,
		readonly = false,
		onenabledchange,
		ondirectionchange
	}: {
		pair: MatchedDevicePair;
		enabled?: boolean;
		direction?: SyncDirection;
		readonly?: boolean;
		onenabledchange?: (enabled: boolean) => void;
		ondirectionchange?: (direction: SyncDirection) => void;
	} = $props();

	let expanded = $state(false);

	let deviceName = $derived(
		pair.windowsDevice.name ?? pair.linuxDevice.name ?? 'Unknown Device'
	);

	let syncChanges = $derived(describeSyncChanges(pair, direction));

	let borderColor = $derived(
		pair.comparison.overallStatus === 'synced'
			? 'border-l-green-500'
			: pair.comparison.overallStatus === 'needs_sync'
				? 'border-l-amber-500'
				: 'border-l-muted-foreground'
	);

	function statusIcon(status: KeyFieldComparison['status']) {
		switch (status) {
			case 'match':
				return { icon: Check, class: 'text-green-500' };
			case 'mismatch':
				return { icon: X, class: 'text-red-500' };
			case 'source_only':
			case 'target_only':
				return { icon: AlertTriangle, class: 'text-amber-500' };
			case 'both_missing':
				return { icon: Minus, class: 'text-muted-foreground' };
		}
	}

	function truncateKey(value: string | null): string {
		if (!value) return '--';
		if (value.length <= 16) return value;
		return value.slice(0, 8) + '...' + value.slice(-8);
	}
</script>

<div class="border rounded-lg border-l-4 {borderColor} bg-card text-card-foreground">
	<div class="p-3 flex items-start gap-3">
		{#if !readonly}
			<label class="flex items-center pt-0.5">
				<input
					type="checkbox"
					checked={enabled}
					onchange={(e) => onenabledchange?.(e.currentTarget.checked)}
					class="rounded border-input h-4 w-4 accent-primary"
				/>
			</label>
		{/if}

		<div class="flex-1 min-w-0">
			<!-- Device info row -->
			<div class="flex items-center gap-2 flex-wrap">
				<span class="font-medium text-sm">{deviceName}</span>
				<span class="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground font-mono">
					{pair.windowsDevice.address}
				</span>
				<span class="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
					{pair.windowsDevice.device_type}
				</span>
			</div>

			<!-- Sync summary -->
			{#if pair.comparison.overallStatus === 'synced'}
				<p class="text-xs text-green-600 dark:text-green-400 mt-1">All keys match</p>
			{:else if syncChanges.length > 0}
				<p class="text-xs text-muted-foreground mt-1">
					Will copy: {syncChanges.join(', ')}
					<span class="font-medium">
						{direction === 'win_to_linux' ? 'Windows' : 'Linux'}
					</span>
					<ArrowRight class="inline h-3 w-3 mx-0.5" />
					<span class="font-medium">
						{direction === 'win_to_linux' ? 'Linux' : 'Windows'}
					</span>
				</p>
			{:else}
				<p class="text-xs text-muted-foreground mt-1">No keys to copy</p>
			{/if}

			<!-- Direction toggle -->
			{#if !readonly && pair.comparison.overallStatus !== 'synced'}
				<div class="flex items-center gap-1 mt-2">
					<div class="inline-flex rounded-md border border-input text-xs">
						<button
							class="px-2 py-1 rounded-l-md transition-colors {direction === 'win_to_linux'
								? 'bg-primary text-primary-foreground'
								: 'hover:bg-muted'}"
							onclick={() => ondirectionchange?.('win_to_linux')}
						>
							Win <ArrowRight class="inline h-3 w-3" /> Lin
						</button>
						<button
							class="px-2 py-1 rounded-r-md transition-colors {direction === 'linux_to_win'
								? 'bg-primary text-primary-foreground'
								: 'hover:bg-muted'}"
							onclick={() => ondirectionchange?.('linux_to_win')}
						>
							Lin <ArrowRight class="inline h-3 w-3" /> Win
						</button>
					</div>
					<button
						class="ml-auto text-xs text-muted-foreground hover:text-foreground transition-colors"
						onclick={() => (expanded = !expanded)}
					>
						{expanded ? 'Hide' : 'Show'} details
					</button>
				</div>
			{:else}
				<button
					class="mt-2 text-xs text-muted-foreground hover:text-foreground transition-colors"
					onclick={() => (expanded = !expanded)}
				>
					{expanded ? 'Hide' : 'Show'} details
				</button>
			{/if}
		</div>
	</div>

	<!-- Expanded key comparison table -->
	{#if expanded}
		<div class="border-t px-3 py-2">
			<table class="w-full text-xs">
				<thead>
					<tr class="text-muted-foreground">
						<th class="text-left font-medium pb-1 w-1/4">Field</th>
						<th class="text-left font-medium pb-1">Windows</th>
						<th class="text-left font-medium pb-1">Linux</th>
						<th class="text-center font-medium pb-1 w-12">Status</th>
					</tr>
				</thead>
				<tbody>
					{#each [...pair.comparison.linkKey, ...pair.comparison.leData] as field}
						{@const si = statusIcon(field.status)}
						<tr class="border-t border-border/50">
							<td class="py-1 text-muted-foreground">{field.label}</td>
							<td class="py-1 font-mono break-all">{truncateKey(field.windowsValue)}</td>
							<td class="py-1 font-mono break-all">{truncateKey(field.linuxValue)}</td>
							<td class="py-1 text-center">
								<si.icon class="inline h-3.5 w-3.5 {si.class}" />
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
