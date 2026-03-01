<script lang="ts">
	import type { UnmatchedDevice } from './matching';
	import { Link, Trash2, Info } from 'lucide-svelte';
	import { osColor } from './os-theme';
	import KeyIndicators from './KeyIndicators.svelte';
	import DeviceDetailsDialog from './DeviceDetailsDialog.svelte';

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

	let detailsOpen = $state(false);

	function handleDeleteClick(e: MouseEvent | KeyboardEvent) {
		e.stopPropagation();
		ondelete?.();
	}

	function handleDetailsClick(e: MouseEvent | KeyboardEvent) {
		e.stopPropagation();
		detailsOpen = true;
	}

	function onKeyActivate(handler: (e: KeyboardEvent) => void) {
		return (e: KeyboardEvent) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				handler(e);
			}
		};
	}
</script>

<DeviceDetailsDialog
	bind:open={detailsOpen}
	data={{ mode: 'single', device: unmatchedDevice.device, os: unmatchedDevice.os, controllerAddress: unmatchedDevice.controllerAddress }}
/>

<button
	class="card"
	class:card-deleted={markedForDeletion}
	class:card-selected={!markedForDeletion && selected}
	class:card-target={!markedForDeletion && !selected && isTarget}
	class:card-drag-over={!markedForDeletion && isDragOver}
	class:card-dimmed={!markedForDeletion && !selected && !isTarget && selectionActive}
	style="--os-border: {osColor(unmatchedDevice.os).borderColor}; --os-gradient: {osColor(unmatchedDevice.os).accentGradient}"
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
		<!-- These use role="button" (not <button>) because they're nested inside the parent <button class="card">.
			 HTML forbids <button> inside <button>. -->
		<div class="card-actions">
			<div
				class="icon-btn details-btn"
				role="button"
				tabindex="-1"
				onclick={handleDetailsClick}
				onkeydown={onKeyActivate(handleDetailsClick)}
				title="View device details"
			>
				<Info class="h-3.5 w-3.5" />
			</div>
			{#if ondelete}
				<div
					class="icon-btn delete-btn"
					class:delete-active={markedForDeletion}
					role="button"
					tabindex="-1"
					onclick={handleDeleteClick}
					onkeydown={onKeyActivate(handleDeleteClick)}
					title={markedForDeletion ? 'Undo deletion' : 'Mark for deletion'}
				>
					<Trash2 class="h-3.5 w-3.5" />
				</div>
			{/if}
		</div>
	</div>
	{#if markedForDeletion}
		<div class="status-banner status-delete">
			Will be deleted - click to undo
		</div>
	{:else if isDragOver}
		<div class="status-banner status-pair">
			<Link class="h-3 w-3" />
			Drop to pair
		</div>
	{:else if selected}
		<div class="status-banner status-pair">
			Click a device on the other side to pair
		</div>
	{:else if isTarget}
		<div class="status-banner status-pair">
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
		border-left: 2.5px solid var(--os-border);
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		border-right: 1px solid rgba(255, 255, 255, 0.06);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
		color: #fafafa;
		padding: 12px 14px;
		cursor: grab;
		transition: all 0.2s;
		position: relative;
		font-family: inherit;
		overflow: hidden;
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
		background: linear-gradient(135deg, rgba(167, 139, 250, 0.06) 0%, transparent 60%);
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
		font-family: ui-monospace, monospace;
		color: rgba(250, 250, 250, 0.35);
		margin-top: 2px;
	}

	.device-mac.name-deleted {
		text-decoration: line-through;
		color: rgba(250, 250, 250, 0.2);
	}

	.indicator-row {
		margin-top: 6px;
	}

	.card-actions {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex-shrink: 0;
	}

	.icon-btn {
		padding: 4px;
		border-radius: 4px;
		transition: color 0.2s;
	}

	.details-btn {
		color: rgba(250, 250, 250, 0.12);
	}

	.card:hover .details-btn {
		color: rgba(250, 250, 250, 0.3);
	}

	.card:hover .details-btn:hover {
		color: rgba(250, 250, 250, 0.7);
	}

	.delete-btn {
		color: rgba(250, 250, 250, 0.12);
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

	.status-banner {
		margin: 8px -14px -12px -14px;
		padding: 6px 14px;
		font-size: 12px;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.status-delete {
		background: rgba(239, 68, 68, 0.08);
		color: #ef4444;
	}

	.status-pair {
		background: rgba(167, 139, 250, 0.08);
		color: #a78bfa;
	}
</style>
