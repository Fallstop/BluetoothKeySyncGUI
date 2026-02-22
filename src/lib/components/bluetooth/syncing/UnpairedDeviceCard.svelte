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

	let borderClass = $derived(
		markedForDeletion
			? 'border-destructive/50 opacity-60'
			: isDragOver
				? 'border-primary ring-2 ring-primary/30 scale-[1.02]'
				: selected
					? 'border-primary ring-2 ring-primary/20'
					: isTarget
						? 'border-primary/50 hover:border-primary hover:ring-2 hover:ring-primary/20'
						: selectionActive
							? 'border-border opacity-50'
							: 'border-border hover:border-foreground/30'
	);

	let osColorClass = $derived(osColor(unmatchedDevice.os).borderL);

	function handleDeleteClick(e: MouseEvent) {
		e.stopPropagation();
		ondelete?.();
	}
</script>

<button
	class="w-full text-left rounded-lg border-l-4 border bg-card text-card-foreground p-3 transition-all {markedForDeletion ? 'cursor-default' : 'cursor-grab active:cursor-grabbing'} {borderClass} {osColorClass} relative group"
	data-unpaired-device
	data-unpaired-os={unmatchedDevice.os}
	data-unpaired-key="{unmatchedDevice.controllerAddress}/{unmatchedDevice.device.address}"
	onclick={markedForDeletion ? handleDeleteClick : onclick}
	onpointerdown={markedForDeletion ? undefined : onpointerdown}
>
	<div class="flex items-start justify-between gap-1">
		<div class="min-w-0 flex-1">
			<div class="font-medium text-sm truncate {markedForDeletion ? 'line-through text-muted-foreground' : ''}">
				{unmatchedDevice.device.name ?? 'Unknown Device'}
			</div>
			<KeyIndicators device={unmatchedDevice.device} class="mt-0.5" />
		</div>
		{#if ondelete}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="shrink-0 p-1 rounded {markedForDeletion
					? 'text-destructive hover:text-destructive/80'
					: 'text-muted-foreground/0 group-hover:text-muted-foreground/40 hover:!text-destructive'} transition-colors"
				onclick={handleDeleteClick}
				title={markedForDeletion ? 'Undo deletion' : 'Mark for deletion'}
			>
				<Trash2 class="h-3.5 w-3.5" />
			</div>
		{/if}
	</div>
	{#if markedForDeletion}
		<div class="mt-1.5 text-xs text-destructive font-medium">
			Will be deleted — click to undo
		</div>
	{:else if isDragOver}
		<div class="mt-1.5 text-xs text-primary font-medium flex items-center gap-1">
			<Link class="h-3 w-3" />
			Drop to pair
		</div>
	{:else if selected}
		<div class="mt-1.5 text-xs text-primary font-medium">
			Click a device on the other side to pair
		</div>
	{:else if isTarget}
		<div class="mt-1.5 text-xs text-primary font-medium flex items-center gap-1">
			<Link class="h-3 w-3" />
			Click to pair
		</div>
	{/if}
</button>
