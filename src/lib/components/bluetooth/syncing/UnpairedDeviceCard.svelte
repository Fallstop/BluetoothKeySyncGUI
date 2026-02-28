<script lang="ts">
	import type { UnmatchedDevice } from './matching';
	import { Link, Trash2 } from 'lucide-svelte';
	import { osColor } from './os-theme';
	import KeyIndicators from './KeyIndicators.svelte';

	let {
		device: unmatchedDevice,
		selected = false,
		selectionActive = false,
		isTarget = false,
		isDragOver = false,
		markedForDeletion = false,
		onclick,
		onpointerdown,
		ondelete
	}: {
		device: UnmatchedDevice;
		selected?: boolean;
		selectionActive?: boolean;
		isTarget?: boolean;
		isDragOver?: boolean;
		markedForDeletion?: boolean;
		onclick?: () => void;
		onpointerdown?: (e: PointerEvent) => void;
		ondelete?: () => void;
	} = $props();

	function handleDeleteClick(e: MouseEvent) {
		e.stopPropagation();
		ondelete?.();
	}
</script>

<button
	class="card"
	class:card-deleted={markedForDeletion}
	class:card-selected={!markedForDeletion && selected}
	class:card-target={!markedForDeletion && !selected && isTarget}
	class:card-drag-over={!markedForDeletion && isDragOver}
	class:card-dimmed={!markedForDeletion && !selected && !isTarget && selectionActive}
	style="--os-border: {osColor(unmatchedDevice.os).borderColor}"
	data-unpaired-device
	data-unpaired-os={unmatchedDevice.os}
	data-unpaired-key="{unmatchedDevice.controllerAddress}/{unmatchedDevice.device.address}"
	onclick={markedForDeletion ? handleDeleteClick : onclick}
	onpointerdown={markedForDeletion ? undefined : onpointerdown}
>
	<div class="card-inner">
		<div class="card-content">
			<div class="device-name" class:name-deleted={markedForDeletion}>
				{unmatchedDevice.device.name ?? 'Unknown Device'}
			</div>
			<div class="device-mac" class:name-deleted={markedForDeletion}>
				{unmatchedDevice.device.address}
			</div>
			<div class="indicator-row">
				<KeyIndicators device={unmatchedDevice.device} />
			</div>
		</div>
		{#if ondelete}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="delete-btn"
				class:delete-active={markedForDeletion}
				onclick={handleDeleteClick}
				title={markedForDeletion ? 'Undo deletion' : 'Mark for deletion'}
			>
				<Trash2 class="h-3.5 w-3.5" />
			</div>
		{/if}
	</div>
	{#if markedForDeletion}
		<div class="status-text status-delete">
			Will be deleted — click to undo
		</div>
	{:else if isDragOver}
		<div class="status-text status-pair">
			<Link class="h-3 w-3" />
			Drop to pair
		</div>
	{:else if selected}
		<div class="status-text status-pair">
			Click a device on the other side to pair
		</div>
	{:else if isTarget}
		<div class="status-text status-pair">
			<Link class="h-3 w-3" />
			Click to pair
		</div>
	{/if}
</button>

<style lang="css">
	.card {
		width: 100%;
		text-align: left;
		border-radius: 10px;
		border-left: 3px solid var(--os-border);
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		border-right: 1px solid rgba(255, 255, 255, 0.06);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
		color: #fafafa;
		padding: 10px 12px;
		cursor: grab;
		transition: all 0.2s;
		position: relative;
		font-family: inherit;
	}

	.card:active {
		cursor: grabbing;
	}

	.card:hover {
		border-color: rgba(255, 255, 255, 0.12);
		border-left-color: var(--os-border);
	}

	.card.card-deleted {
		opacity: 0.6;
		cursor: default;
		border-color: rgba(239, 68, 68, 0.3);
		border-left-color: rgba(239, 68, 68, 0.5);
	}

	.card.card-selected {
		border-color: rgba(167, 139, 250, 0.4);
		border-left-color: var(--os-border);
		box-shadow: 0 0 0 2px rgba(167, 139, 250, 0.2);
	}

	.card.card-target {
		border-color: rgba(167, 139, 250, 0.25);
		border-left-color: var(--os-border);
	}

	.card.card-target:hover {
		border-color: rgba(167, 139, 250, 0.4);
		border-left-color: var(--os-border);
		box-shadow: 0 0 0 2px rgba(167, 139, 250, 0.2);
	}

	.card.card-drag-over {
		border-color: rgba(167, 139, 250, 0.5);
		border-left-color: var(--os-border);
		box-shadow: 0 0 0 2px rgba(167, 139, 250, 0.3);
		transform: scale(1.02);
	}

	.card.card-dimmed {
		opacity: 0.5;
	}

	.card-inner {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 4px;
	}

	.card-content {
		min-width: 0;
		flex: 1;
	}

	.device-name {
		font-weight: 500;
		font-size: 14px;
		color: rgba(250, 250, 250, 0.85);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.device-name.name-deleted {
		text-decoration: line-through;
		color: rgba(250, 250, 250, 0.4);
	}

	.device-mac {
		font-size: 11px;
		font-family: monospace;
		color: rgba(250, 250, 250, 0.35);
		margin-top: 1px;
	}

	.device-mac.name-deleted {
		text-decoration: line-through;
		color: rgba(250, 250, 250, 0.2);
	}

	.indicator-row {
		margin-top: 2px;
	}

	.delete-btn {
		flex-shrink: 0;
		padding: 4px;
		border-radius: 4px;
		color: rgba(250, 250, 250, 0);
		transition: color 0.2s;
	}

	.card:hover .delete-btn {
		color: rgba(250, 250, 250, 0.3);
	}

	.card:hover .delete-btn:hover {
		color: #ef4444;
	}

	.delete-btn.delete-active {
		color: #ef4444;
	}

	.delete-btn.delete-active:hover {
		color: rgba(239, 68, 68, 0.7);
	}

	.status-text {
		margin-top: 6px;
		font-size: 12px;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.status-delete {
		color: #ef4444;
	}

	.status-pair {
		color: #a78bfa;
	}
</style>
