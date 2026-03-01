<script lang="ts">
	import type {
		MatchedDevicePair,
		ManualMatch,
		SyncDirection
	} from './matching';
	import { describeSyncChanges } from './matching';
	import { Check, X } from 'lucide-svelte';
	import { osColor } from './os-theme';
	import PairSideMiniCard from './PairSideMiniCard.svelte';
	import DeviceDetailsDialog from './DeviceDetailsDialog.svelte';

	let {
		pair,
		direction = null as SyncDirection | null,
		readonly = false,
		isManual = false,
		ondirectionchange,
		onunlink
	}: {
		pair: MatchedDevicePair | ManualMatch;
		direction?: SyncDirection | null;
		readonly?: boolean;
		isManual?: boolean;
		ondirectionchange?: (direction: SyncDirection) => void;
		onunlink?: () => void;
	} = $props();

	let detailsOpen = $state(false);
	let rowEl: HTMLDivElement;

	let isSynced = $derived(pair.comparison.overallStatus === 'synced');
	let canDrag = $derived(!readonly && !isSynced);

	let syncChanges = $derived(describeSyncChanges(pair, direction));

	// Border color based on state
	let borderColor = $derived(
		isSynced
			? 'rgba(34, 197, 94, 0.3)'
			: direction !== null
				? 'rgba(167, 139, 250, 0.3)'
				: isManual
					? 'rgba(255, 255, 255, 0.1)'
					: 'rgba(255, 255, 255, 0.06)'
	);

	let borderStyle = $derived(isManual && !isSynced && direction === null ? 'dashed' : 'solid');

	// Direction arrow color matches the source OS
	let dirArrowColor = $derived(
		direction === 'win_to_linux'
			? osColor('Windows').textColor
			: direction === 'linux_to_win'
				? osColor('Linux').textColor
				: null
	);

	let dirBorderColor = $derived(
		direction === 'win_to_linux'
			? osColor('Windows').ringColor
			: direction === 'linux_to_win'
				? osColor('Linux').ringColor
				: null
	);

	let dirBgColor = $derived(
		direction === 'win_to_linux'
			? osColor('Windows').badgeBg
			: direction === 'linux_to_win'
				? osColor('Linux').badgeBg
				: null
	);

	// --- Drag-to-set-direction ---
	let draggingFromSide: 'win' | 'lin' | null = $state(null);
	let dragHoverSide: 'win' | 'lin' | null = $state(null);

	$effect(() => {
		if (draggingFromSide) {
			document.body.style.cursor = 'grabbing';
			document.body.style.userSelect = 'none';
			return () => {
				document.body.style.cursor = '';
				document.body.style.userSelect = '';
			};
		}
	});

	function handleSidePointerDown(side: 'win' | 'lin', e: PointerEvent) {
		if (!canDrag || e.button !== 0) return;
		draggingFromSide = side;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		e.preventDefault();
	}

	function handleSidePointerMove(e: PointerEvent) {
		if (!draggingFromSide) return;
		const el = document.elementFromPoint(e.clientX, e.clientY);
		const target = el?.closest('[data-pair-side]') as HTMLElement | null;
		if (target && rowEl?.contains(target)) {
			dragHoverSide = (target.dataset.pairSide as 'win' | 'lin') ?? null;
		} else {
			dragHoverSide = null;
		}
	}

	function handleSidePointerUp(e: PointerEvent) {
		if (!draggingFromSide) return;
		const el = document.elementFromPoint(e.clientX, e.clientY);
		const target = el?.closest('[data-pair-side]') as HTMLElement | null;
		if (target && rowEl?.contains(target)) {
			const targetSide = target.dataset.pairSide;
			if (draggingFromSide === 'win' && targetSide === 'lin') {
				ondirectionchange?.('win_to_linux');
			} else if (draggingFromSide === 'lin' && targetSide === 'win') {
				ondirectionchange?.('linux_to_win');
			}
		}
		draggingFromSide = null;
		dragHoverSide = null;
	}

	function handleSidePointerCancel() {
		draggingFromSide = null;
		dragHoverSide = null;
	}

	let winIsDragSource = $derived(draggingFromSide === 'win');
	let winIsDragTarget = $derived(dragHoverSide === 'win' && draggingFromSide === 'lin');
	let linIsDragSource = $derived(draggingFromSide === 'lin');
	let linIsDragTarget = $derived(dragHoverSide === 'lin' && draggingFromSide === 'win');
</script>

<DeviceDetailsDialog
	bind:open={detailsOpen}
	data={{ mode: 'pair', pair, direction }}
/>

<div class="pair-row" style="--border-color: {borderColor}; --border-style: {borderStyle}" bind:this={rowEl}>
	<!-- Main row: Win card - direction - Lin card -->
	<div class="row-main">
		<!-- Windows device mini-card -->
		<PairSideMiniCard
			device={pair.windowsDevice}
			os="Windows"
			side="left"
			isDragSource={winIsDragSource}
			isDragTarget={winIsDragTarget}
			{canDrag}
			onpointerdown={(e) => handleSidePointerDown('win', e)}
			onpointermove={handleSidePointerMove}
			onpointerup={handleSidePointerUp}
			onpointercancel={handleSidePointerCancel}
		/>

		<!-- Center: Direction toggle or synced indicator -->
		<div class="direction-center">
			{#if isSynced}
				<div class="dir-toggle dir-synced" title="Keys are already in sync">
					<Check class="h-4 w-4" />
				</div>
			{:else if !readonly}
				<button
					class="dir-toggle"
					class:dir-active={direction !== null}
					style={dirArrowColor ? `color: ${dirArrowColor}; border-color: ${dirBorderColor}; background: ${dirBgColor}` : ''}
					onclick={() => {
						if (direction === null || direction === 'linux_to_win') {
							ondirectionchange?.('win_to_linux');
						} else {
							ondirectionchange?.('linux_to_win');
						}
					}}
					title={direction === 'win_to_linux'
						? 'Copying Windows → Linux (click to reverse)'
						: direction === 'linux_to_win'
							? 'Copying Linux → Windows (click to reverse)'
							: 'Click to set sync direction'}
				>
					{#if direction === null}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
							<path d="M8 3L4 7l4 4" />
							<path d="M4 7h16" />
							<path d="M16 21l4-4-4-4" />
							<path d="M20 17H4" />
						</svg>
					{:else if direction === 'win_to_linux'}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M5 12h14" />
							<path d="M13 6l6 6-6 6" />
						</svg>
					{:else}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M19 12H5" />
							<path d="M11 18l-6-6 6-6" />
						</svg>
					{/if}
				</button>
			{/if}
		</div>

		<!-- Linux device mini-card -->
		<PairSideMiniCard
			device={pair.linuxDevice}
			os="Linux"
			side="right"
			isDragSource={linIsDragSource}
			isDragTarget={linIsDragTarget}
			{canDrag}
			onpointerdown={(e) => handleSidePointerDown('lin', e)}
			onpointermove={handleSidePointerMove}
			onpointerup={handleSidePointerUp}
			onpointercancel={handleSidePointerCancel}
		/>
	</div>

	<!-- Bottom bar: sync info + details toggle -->
	<div class="row-footer">
		<div class="footer-info">
			{#if direction && syncChanges.length > 0 && !isSynced}
				<span class="keys-badge">
					{syncChanges.length} key{syncChanges.length !== 1 ? 's' : ''} to copy
				</span>
			{/if}
		</div>
		<div class="footer-actions">
			{#if onunlink}
				<button
					class="action-btn"
					onclick={() => onunlink?.()}
					title="Unmatch these devices"
				>
					<X class="h-3 w-3" />
					Unmatch
				</button>
			{/if}
			<button
				class="action-btn"
				onclick={() => (detailsOpen = true)}
			>
				Details
			</button>
		</div>
	</div>
</div>

<style lang="css">
	.pair-row {
		border-radius: 12px;
		border-left: 2.5px var(--border-style) var(--border-color);
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		border-right: 1px solid rgba(255, 255, 255, 0.06);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
		padding: 14px;
		transition: all 0.2s;
	}

	.row-main {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	/* Direction picker */
	.direction-center {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
	}

	.dir-toggle {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 1px solid rgba(255, 255, 255, 0.1);
		background: rgba(255, 255, 255, 0.03);
		color: rgba(250, 250, 250, 0.3);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 0.2s;
		font-family: inherit;
		padding: 0;
	}

	.dir-toggle:hover {
		filter: brightness(1.2);
	}

	.dir-toggle.dir-synced {
		border-color: rgba(34, 197, 94, 0.3);
		background: rgba(34, 197, 94, 0.1);
		color: #4ade80;
		cursor: default;
	}

	/* Footer */
	.row-footer {
		padding-top: 10px;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.action-btn {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.12);
		border: none;
		background: none;
		cursor: pointer;
		padding: 0;
		font-family: inherit;
		transition: color 0.2s;
		display: flex;
		align-items: center;
		gap: 3px;
	}

	.pair-row:hover .action-btn {
		color: rgba(250, 250, 250, 0.3);
	}

	.pair-row:hover .action-btn:hover {
		color: rgba(250, 250, 250, 0.7);
	}

	.keys-badge {
		font-size: 11px;
		font-weight: 500;
		padding: 2px 8px;
		border-radius: 6px;
		background: rgba(167, 139, 250, 0.1);
		color: #c4b5fd;
		border: 1px solid rgba(167, 139, 250, 0.15);
	}
</style>
