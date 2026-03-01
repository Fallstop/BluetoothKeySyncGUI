<script lang="ts">
	import type {
		MatchResult,
		MatchedDevicePair,
		ManualMatch,
		SyncSelections,
		UnmatchedDevice
	} from './matching';
	import type { Message, SyncRequest, SyncResult } from '#root/bindings';
	import {
		buildSyncProposals,
		buildDeleteProposals,
		describeSyncChanges,
		pairKey,
		manualPairKey,
		deviceKey
	} from './matching';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { ArrowRight, Check, X, Loader2, AlertTriangle, Trash2, CheckCircle } from 'lucide-svelte';
	import { rpc } from '$lib/api';
	import { osColor } from './os-theme';
	import { windowsState, btStore } from '@/state';

	let {
		matchResult,
		manualMatches,
		selections,
		deletions,
		unpairedDevices,
		onsynccomplete
	}: {
		matchResult: MatchResult;
		manualMatches: ManualMatch[];
		selections: SyncSelections;
		deletions: Set<string>;
		unpairedDevices: UnmatchedDevice[];
		onsynccomplete?: () => void;
	} = $props();

	let isApplying = $state(false);
	let applyResult = $state<{ success: boolean; message: string } | null>(null);
	let dialogOpen = $state(false);

	// Reset stale result when reopening the dialog
	$effect(() => {
		if (dialogOpen) {
			applyResult = null;
		}
	});

	let copyProposals = $derived(buildSyncProposals(matchResult, selections, manualMatches));
	let deleteProposals = $derived(buildDeleteProposals(deletions, unpairedDevices));
	let allProposals = $derived([...copyProposals, ...deleteProposals]);

	// Count pairs with a direction chosen (ready)
	let readyCount = $derived(
		Array.from(selections.values()).filter((s) => s.direction !== null).length
	);

	let deleteCount = $derived(deletions.size);

	// Build list of pairs that have a direction selected for review
	let selectedPairs = $derived.by(() => {
		const pairs: Array<{
			pair: MatchedDevicePair | ManualMatch;
			direction: NonNullable<(typeof selections extends Map<string, infer V> ? V : never)['direction']>;
		}> = [];

		for (const p of matchResult.needsSync) {
			const key = pairKey(p.controllerAddress, p.windowsDevice.address);
			const sel = selections.get(key);
			if (sel?.direction) {
				pairs.push({ pair: p, direction: sel.direction });
			}
		}

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
	let deletionDevices = $derived.by(() => {
		return unpairedDevices.filter((d) =>
			deletions.has(deviceKey(d.os, d.controllerAddress, d.device.address))
		);
	});

	let hasAnyActionable = $derived(matchResult.needsSync.length > 0 || manualMatches.length > 0 || deleteCount > 0);
	let deleteOnly = $derived(readyCount === 0 && deleteCount > 0);

	async function applyChanges() {
		isApplying = true;
		applyResult = null;

		try {
			const result = await rpc.sync.apply_sync_proposals({
				proposals: allProposals,
				windows_hive_path: windowsState.state.lastWindowsHiveFile ?? null
			});

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

				// Refresh btStore with updated data so the UI reflects synced state
				if (data.refreshed_linux) {
					btStore.state.linux = data.refreshed_linux;
				}
				if (data.refreshed_windows) {
					btStore.state.windows = data.refreshed_windows;
				}

				// Clear stale manual matches, deletions, and dismissed pairs
				onsynccomplete?.();
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
	<div class="action-bar">
		<div class="action-bar-inner">
			<div class="status-group">
				{#if readyCount > 0}
					<span class="status-chip status-chip-ready">
						{readyCount} sync{readyCount !== 1 ? 's' : ''} ready
					</span>
				{/if}
				{#if deleteCount > 0}
					<span class="status-chip status-chip-delete">
						<Trash2 class="h-3 w-3" />
						{deleteCount} to delete
					</span>
				{/if}
			</div>

			{#if deleteOnly}
				<!-- Delete-only: apply directly, no review dialog needed -->
				<button
					class="gf-btn destructive"
					onclick={applyChanges}
					disabled={isApplying}
				>
					{#if isApplying}
						<Loader2 class="h-4 w-4 animate-spin" />
						Deleting...
					{:else}
						<Trash2 class="h-4 w-4" />
						Delete
					{/if}
				</button>
			{:else}
				<Dialog.Root bind:open={dialogOpen}>
					<Dialog.Trigger
						class="gf-btn primary"
						disabled={readyCount === 0 && deleteCount === 0}
					>
						<ArrowRight class="h-4 w-4" />
						Review & Apply
					</Dialog.Trigger>
					<Dialog.Content class="glass-dialog">
						<Dialog.Header>
							<Dialog.Title class="glass-dialog-title">Review Changes</Dialog.Title>
							<Dialog.Description class="glass-dialog-desc">
								{#if readyCount > 0 && deleteCount > 0}
									{readyCount} sync{readyCount !== 1 ? 's' : ''} and {deleteCount} deletion{deleteCount !== 1 ? 's' : ''}.
								{:else}
									{readyCount} device{readyCount !== 1 ? 's' : ''} will be synced.
								{/if}
							</Dialog.Description>
						</Dialog.Header>

						<div class="review-list">
							<!-- Sync proposals -->
							{#each selectedPairs as { pair, direction }}
								{@const changes = describeSyncChanges(pair, direction)}
								{@const sourceOs = direction === 'win_to_linux' ? 'Windows' : 'Linux'}
								{@const targetOs = direction === 'win_to_linux' ? 'Linux' : 'Windows'}
								<div class="review-card">
									<div class="review-card-header">
										<div class="review-name-group">
											<span class="review-device-name">
												{pair.windowsDevice.name ?? pair.linuxDevice.name ?? 'Unknown'}
											</span>
											{#if pair.windowsDevice.device_type}
												<span class="review-type-badge">{pair.windowsDevice.device_type}</span>
											{/if}
										</div>
										<div class="review-direction">
											<span class="os-pill" style="background: {osColor(sourceOs).pillBg}; border-color: {osColor(sourceOs).pillBorder}; color: {osColor(sourceOs).pillText}">
												{sourceOs}
											</span>
											<ArrowRight class="h-3 w-3" style="color: rgba(250,250,250,0.3)" />
											<span class="os-pill" style="background: {osColor(targetOs).pillBg}; border-color: {osColor(targetOs).pillBorder}; color: {osColor(targetOs).pillText}">
												{targetOs}
											</span>
										</div>
									</div>
									<div class="review-addresses">
										<span class="addr-row">
											<span class="os-dot" style="background: {osColor('Windows').hex}"></span>
											{pair.windowsDevice.address}
										</span>
										<span class="addr-row">
											<span class="os-dot" style="background: {osColor('Linux').hex}"></span>
											{pair.linuxDevice.address}
										</span>
									</div>
									{#if changes.length > 0}
										<div class="review-tags">
											{#each changes as change}
												<span class="key-tag">{change}</span>
											{/each}
										</div>
									{/if}
								</div>
							{/each}

							<!-- Deletion proposals -->
							{#each deletionDevices as device}
								<div class="review-card review-card-delete">
									<div class="review-card-header">
										<div class="review-delete-label">
											<Trash2 class="h-3.5 w-3.5" style="color: #ef4444" />
											<span class="review-device-name">
												{device.device.name ?? 'Unknown Device'}
											</span>
										</div>
										<span class="delete-badge">delete</span>
									</div>
									<div class="review-addresses">
										<span class="addr-row">
											<span class="os-dot" style="background: {osColor(device.os).hex}"></span>
											{device.os} &middot; {device.device.address}
										</span>
									</div>
								</div>
							{/each}
						</div>

						{#if applyResult}
							<div
								class="result-alert"
								class:result-success={applyResult.success}
								class:result-error={!applyResult.success}
							>
								{#if applyResult.success}
									<CheckCircle class="h-4 w-4 flex-shrink-0" />
								{:else}
									<AlertTriangle class="h-4 w-4 flex-shrink-0" />
								{/if}
								<span>{applyResult.message}</span>
							</div>
						{/if}

						<Dialog.Footer class="glass-dialog-footer">
							<Dialog.Close class="gf-btn ghost">
								<X class="h-4 w-4" />
								{applyResult?.success ? 'Done' : 'Cancel'}
							</Dialog.Close>
							{#if !applyResult?.success}
								<button
									class="gf-btn {deleteCount > 0 ? 'destructive' : 'primary'}"
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
								</button>
							{/if}
						</Dialog.Footer>
					</Dialog.Content>
				</Dialog.Root>
			{/if}
		</div>
	</div>
{/if}

<style lang="css">
	.action-bar {
		position: sticky;
		bottom: 0;
		width: 100%;
		padding: 16px;
		background: linear-gradient(to top, rgba(9, 9, 11, 0.95), rgba(9, 9, 11, 0.85));
		backdrop-filter: blur(12px);
		border-top: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 16px 16px 0 0;
		box-shadow: 0 -4px 24px rgba(0, 0, 0, 0.3);
		z-index: 10;
	}

	.action-bar-inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		max-width: 60rem;
		margin: 0 auto;
	}

	.status-group {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.status-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 12px;
		font-weight: 500;
		padding: 4px 10px;
		border-radius: 8px;
	}

	.status-chip-ready {
		background: rgba(34, 197, 94, 0.1);
		color: #4ade80;
		border: 1px solid rgba(34, 197, 94, 0.2);
	}

	.status-chip-delete {
		background: rgba(239, 68, 68, 0.1);
		color: #ef4444;
		border: 1px solid rgba(239, 68, 68, 0.2);
	}

	/* Review list */
	.review-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: 384px;
		overflow-y: auto;
	}

	.review-card {
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: 10px;
		padding: 12px;
		background: rgba(255, 255, 255, 0.02);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.review-card-delete {
		border-color: rgba(239, 68, 68, 0.2);
	}

	.review-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.review-name-group {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.review-device-name {
		font-weight: 500;
		font-size: 14px;
		color: rgba(250, 250, 250, 0.85);
	}

	.review-type-badge {
		font-size: 10px;
		font-weight: 500;
		padding: 1px 6px;
		border-radius: 4px;
		background: rgba(255, 255, 255, 0.06);
		color: rgba(250, 250, 250, 0.4);
		white-space: nowrap;
	}

	.review-direction {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.os-pill {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 6px;
		border: 1px solid;
		font-size: 11px;
		font-weight: 600;
	}

	.review-delete-label {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.delete-badge {
		font-size: 11px;
		font-weight: 500;
		padding: 2px 8px;
		border-radius: 6px;
		background: rgba(239, 68, 68, 0.1);
		color: #ef4444;
		border: 1px solid rgba(239, 68, 68, 0.2);
	}

	.review-addresses {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
		font-family: ui-monospace, monospace;
	}

	.addr-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.os-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.review-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.key-tag {
		font-size: 10px;
		font-weight: 500;
		padding: 2px 7px;
		border-radius: 5px;
		background: rgba(167, 139, 250, 0.08);
		color: rgba(167, 139, 250, 0.7);
		border: 1px solid rgba(167, 139, 250, 0.12);
	}

	/* Result alert */
	.result-alert {
		border-radius: 8px;
		padding: 12px;
		font-size: 14px;
		margin-top: 8px;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.result-success {
		background: rgba(34, 197, 94, 0.1);
		border: 1px solid rgba(34, 197, 94, 0.2);
		color: #4ade80;
	}

	.result-error {
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.2);
		color: #fca5a5;
	}
</style>
