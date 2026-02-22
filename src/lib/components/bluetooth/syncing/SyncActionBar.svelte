<script lang="ts">
	import type { MatchResult, SyncSelections, SyncRequest, SyncResult } from './matching';
	import type { Message } from '#root/bindings';
	import { buildSyncProposals, describeSyncChanges, pairKey } from './matching';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button';
	import { ArrowRight, Check, X, Loader2 } from 'lucide-svelte';
	import { rpc } from '$lib/api';

	let {
		matchResult,
		selections
	}: {
		matchResult: MatchResult;
		selections: SyncSelections;
	} = $props();

	let isApplying = $state(false);
	let applyResult = $state<{ success: boolean; message: string } | null>(null);
	let dialogOpen = $state(false);

	let proposals = $derived(buildSyncProposals(matchResult, selections));

	let selectedCount = $derived(
		Array.from(selections.values()).filter((s) => s.enabled).length
	);

	let selectedPairs = $derived(
		matchResult.needsSync.filter((pair) => {
			const sel = selections.get(pairKey(pair.controllerAddress, pair.windowsDevice.address));
			return sel?.enabled;
		})
	);

	async function applyChanges() {
		isApplying = true;
		applyResult = null;

		try {
			// The sync route exists in Rust but bindings.ts hasn't been regenerated to include it.
			// Once bindings are regenerated, this cast can be removed.
			const syncRpc = rpc as unknown as {
				sync: { apply_sync_proposals: (req: SyncRequest) => Promise<Message<SyncResult>> };
			};
			const result = await syncRpc.sync.apply_sync_proposals({ proposals });

			if (result.type === 'Success') {
				const data = result.data;
				if (data.success) {
					applyResult = {
						success: true,
						message: `Successfully synced ${data.applied_count} device(s).`
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

{#if matchResult.needsSync.length > 0}
	<div class="sticky bottom-0 w-full p-4 bg-background/80 backdrop-blur-sm border-t">
		<div class="flex items-center justify-between max-w-4xl mx-auto">
			<span class="text-sm text-muted-foreground">
				{selectedCount} device{selectedCount !== 1 ? 's' : ''} selected for sync
			</span>

			<Dialog.Root bind:open={dialogOpen}>
				<Dialog.Trigger>
					<Button disabled={selectedCount === 0}>
						<ArrowRight class="h-4 w-4" />
						Review & Apply
					</Button>
				</Dialog.Trigger>
				<Dialog.Content class="max-w-2xl">
					<Dialog.Header>
						<Dialog.Title>Review Sync Changes</Dialog.Title>
						<Dialog.Description>
							{selectedCount} device{selectedCount !== 1 ? 's' : ''} will be synced.
							Review the changes below before applying.
						</Dialog.Description>
					</Dialog.Header>

					<div class="space-y-3 max-h-96 overflow-y-auto">
						{#each selectedPairs as pair}
							{@const key = pairKey(pair.controllerAddress, pair.windowsDevice.address)}
							{@const sel = selections.get(key)}
							{@const dir = sel?.direction ?? pair.recommendedDirection}
							{@const changes = describeSyncChanges(pair, dir)}
							<div class="border rounded-lg p-3 space-y-1.5">
								<div class="flex items-center justify-between">
									<span class="font-medium text-sm">
										{pair.windowsDevice.name ?? pair.linuxDevice.name ?? 'Unknown'}
									</span>
									<div class="flex items-center gap-1.5 text-xs">
										<span class="px-1.5 py-0.5 rounded bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300">
											{dir === 'win_to_linux' ? 'Windows' : 'Linux'}
										</span>
										<ArrowRight class="h-3 w-3" />
										<span class="px-1.5 py-0.5 rounded bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300">
											{dir === 'win_to_linux' ? 'Linux' : 'Windows'}
										</span>
									</div>
								</div>
								<p class="text-xs text-muted-foreground font-mono">
									{pair.windowsDevice.address}
								</p>
								{#if changes.length > 0}
									<p class="text-xs text-muted-foreground">
										Keys: {changes.join(', ')}
									</p>
								{/if}
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
