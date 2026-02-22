<script lang="ts">
	import type {
		MatchResult,
		ManualMatch,
		SyncSelections,
		SyncRequest,
		SyncResult,
		UnmatchedDevice
	} from './matching';
	import type { Message } from '#root/bindings';
	import {
		buildSyncProposals,
		buildDeleteProposals,
		describeSyncChanges,
		manualPairKey,
		deviceKey
	} from './matching';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button';
	import { ArrowRight, Check, X, Loader2, AlertTriangle, Trash2 } from 'lucide-svelte';
	import { rpc } from '$lib/api';
	import { osColor } from './os-theme';

	let {
		matchResult,
		manualMatches,
		selections,
		deletions,
		unpairedDevices
	}: {
		matchResult: MatchResult;
		manualMatches: ManualMatch[];
		selections: SyncSelections;
		deletions: Set<string>;
		unpairedDevices: UnmatchedDevice[];
	} = $props();

	let isApplying = $state(false);
	let applyResult = $state<{ success: boolean; message: string } | null>(null);
	let dialogOpen = $state(false);

	let copyProposals = $derived(buildSyncProposals(matchResult, selections, manualMatches));
	let deleteProposals = $derived(buildDeleteProposals(deletions, unpairedDevices));
	let allProposals = $derived([...copyProposals, ...deleteProposals]);

	// Count pairs with a direction chosen (ready)
	let readyCount = $derived(
		Array.from(selections.values()).filter((s) => s.direction !== null).length
	);

	let deleteCount = $derived(deletions.size);

	// Count manual matches that still need a direction picked
	let actionableNeedsDirection = $derived(() => {
		let count = 0;
		for (const match of manualMatches) {
			const key = manualPairKey(match.id);
			const sel = selections.get(key);
			if (sel && sel.direction === null) count++;
		}
		return count;
	});

	// Build list of pairs that have a direction selected for review
	let selectedPairs = $derived(() => {
		const pairs: Array<{
			pair: ManualMatch;
			direction: NonNullable<(typeof selections extends Map<string, infer V> ? V : never)['direction']>;
		}> = [];

		for (const m of manualMatches) {
			const key = manualPairKey(m.id);
			const sel = selections.get(key);
			if (sel?.direction) {
				pairs.push({ pair: m, direction: sel.direction });
			}
		}

		return pairs;
	});

	// Devices marked for deletion with their info
	let deletionDevices = $derived(() => {
		return unpairedDevices.filter((d) =>
			deletions.has(deviceKey(d.os, d.controllerAddress, d.device.address))
		);
	});

	let hasAnyActionable = $derived(manualMatches.length > 0 || deleteCount > 0);

	async function applyChanges() {
		isApplying = true;
		applyResult = null;

		try {
			const syncRpc = rpc as unknown as {
				sync: { apply_sync_proposals: (req: SyncRequest) => Promise<Message<SyncResult>> };
			};
			const result = await syncRpc.sync.apply_sync_proposals({ proposals: allProposals });

			if (result.type === 'Success') {
				const data = result.data;
				if (data.success) {
					const total = data.applied_count;
					applyResult = {
						success: true,
						message: `Successfully applied ${total} change${total !== 1 ? 's' : ''}.`
					};
				} else {
					applyResult = {
						success: false,
						message: `${data.applied_count} succeeded, ${data.failed_count} failed. ${data.errors.join('; ')}`
					};
				}
			} else {
				applyResult = { success: false, message: result.data };
			}
		} catch (error) {
			applyResult = {
				success: false,
				message: `Unexpected error: ${error instanceof Error ? error.message : String(error)}`
			};
		} finally {
			isApplying = false;
		}
	}
</script>

{#if hasAnyActionable}
	<div class="sticky bottom-0 w-full p-4 bg-background/80 backdrop-blur-sm border-t">
		<div class="flex items-center justify-between max-w-4xl mx-auto">
			<div class="flex items-center gap-3 text-sm">
				{#if readyCount > 0}
					<span class="text-muted-foreground">
						{readyCount} sync{readyCount !== 1 ? 's' : ''} ready
					</span>
				{/if}
				{#if deleteCount > 0}
					<span class="text-destructive flex items-center gap-1">
						<Trash2 class="h-3.5 w-3.5" />
						{deleteCount} to delete
					</span>
				{/if}
				{#if actionableNeedsDirection() > 0}
					<span
						class="text-amber-600 dark:text-amber-400 flex items-center gap-1"
					>
						<AlertTriangle class="h-3.5 w-3.5" />
						{actionableNeedsDirection()} need{actionableNeedsDirection() !== 1 ? '' : 's'} direction
					</span>
				{/if}
			</div>

			<Dialog.Root bind:open={dialogOpen}>
				<Dialog.Trigger>
					<Button disabled={readyCount === 0 && deleteCount === 0}>
						<ArrowRight class="h-4 w-4" />
						Review & Apply
					</Button>
				</Dialog.Trigger>
				<Dialog.Content class="max-w-2xl">
					<Dialog.Header>
						<Dialog.Title>Review Changes</Dialog.Title>
						<Dialog.Description>
							{#if readyCount > 0 && deleteCount > 0}
								{readyCount} sync{readyCount !== 1 ? 's' : ''} and {deleteCount} deletion{deleteCount !== 1 ? 's' : ''}.
							{:else if readyCount > 0}
								{readyCount} device{readyCount !== 1 ? 's' : ''} will be synced.
							{:else}
								{deleteCount} device{deleteCount !== 1 ? 's' : ''} will be deleted.
							{/if}
							Review the changes below before applying.
						</Dialog.Description>
					</Dialog.Header>

					<div class="space-y-3 max-h-96 overflow-y-auto">
						<!-- Sync proposals -->
						{#each selectedPairs() as { pair, direction }}
							{@const changes = describeSyncChanges(pair, direction)}
							{@const sourceOs = direction === 'win_to_linux' ? 'Windows' : 'Linux'}
							{@const targetOs = direction === 'win_to_linux' ? 'Linux' : 'Windows'}
							<div class="border rounded-lg p-3 space-y-1.5">
								<div class="flex items-center justify-between">
									<span class="font-medium text-sm">
										{pair.windowsDevice.name ?? pair.linuxDevice.name ?? 'Unknown'}
									</span>
									<div class="flex items-center gap-1.5 text-xs">
										<span class="px-1.5 py-0.5 rounded {osColor(sourceOs).badge}">
											{sourceOs}
										</span>
										<ArrowRight class="h-3 w-3" />
										<span class="px-1.5 py-0.5 rounded {osColor(targetOs).badge}">
											{targetOs}
										</span>
									</div>
								</div>
								<div class="flex gap-3 text-xs text-muted-foreground font-mono">
									<span>Win: {pair.windowsDevice.address}</span>
									<span>Lin: {pair.linuxDevice.address}</span>
								</div>
								{#if changes.length > 0}
									<p class="text-xs text-muted-foreground">
										Keys: {changes.join(', ')}
									</p>
								{/if}
							</div>
						{/each}

						<!-- Deletion proposals -->
						{#each deletionDevices() as device}
							<div class="border border-destructive/30 rounded-lg p-3 space-y-1">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Trash2 class="h-3.5 w-3.5 text-destructive" />
										<span class="font-medium text-sm">
											{device.device.name ?? 'Unknown Device'}
										</span>
									</div>
									<span class="text-xs px-1.5 py-0.5 rounded bg-destructive/10 text-destructive">
										delete
									</span>
								</div>
								<div class="text-xs text-muted-foreground font-mono">
									{device.os} &middot; {device.device.address}
								</div>
							</div>
						{/each}
					</div>

					{#if applyResult}
						<div
							class="rounded-lg p-3 text-sm {applyResult.success
								? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
								: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'}"
						>
							{applyResult.message}
						</div>
					{/if}

					<Dialog.Footer>
						<Dialog.Close>
							<Button variant="outline">
								<X class="h-4 w-4" />
								{applyResult?.success ? 'Done' : 'Cancel'}
							</Button>
						</Dialog.Close>
						{#if !applyResult?.success}
							<Button
								onclick={applyChanges}
								disabled={isApplying}
								variant={deleteCount > 0 && readyCount === 0 ? 'destructive' : 'default'}
							>
								{#if isApplying}
									<Loader2 class="h-4 w-4 animate-spin" />
									Applying...
								{:else}
									<Check class="h-4 w-4" />
									Apply Changes
								{/if}
							</Button>
						{/if}
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>
		</div>
	</div>
{/if}
