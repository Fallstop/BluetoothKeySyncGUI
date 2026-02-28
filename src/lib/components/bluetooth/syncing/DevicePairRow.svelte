<script lang="ts">
	import type {
		MatchedDevicePair,
		ManualMatch,
		SyncDirection,
		KeyFieldComparison
	} from './matching';
	import { describeSyncChanges } from './matching';
	import { Check, X, AlertTriangle, Minus, EyeOff } from 'lucide-svelte';
	import { osColor } from './os-theme';
	import PairSideMiniCard from './PairSideMiniCard.svelte';

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

	let expanded = $state(false);
	let rowEl: HTMLDivElement;

	let isSynced = $derived(pair.comparison.overallStatus === 'synced');
	let canDrag = $derived(!readonly && !isSynced);

	let syncChanges = $derived(describeSyncChanges(pair, direction));

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

	// --- Helpers ---
	function statusIcon(status: KeyFieldComparison['status']) {
		switch (status) {
			case 'match':
				return { icon: Check, color: '#22c55e' };
			case 'mismatch':
				return { icon: X, color: '#ef4444' };
			case 'source_only':
			case 'target_only':
				return { icon: AlertTriangle, color: '#f59e0b' };
			case 'os_not_available':
				return { icon: Minus, color: 'rgba(250,250,250,0.2)' };
			case 'both_missing':
				return { icon: Minus, color: 'rgba(250,250,250,0.3)' };
		}
	}

	function truncateKey(value: string | null, status?: KeyFieldComparison['status']): string {
		if (!value) {
			return status === 'os_not_available' ? 'N/A' : '--';
		}
		if (value.length <= 16) return value;
		return value.slice(0, 8) + '...' + value.slice(-8);
	}
</script>

<div class="pair-row" bind:this={rowEl}>
	<!-- Main row: Win card — direction — Lin card -->
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
				<span class="dir-label dir-label-synced">synced</span>
			{:else if !readonly}
				<button
					class="dir-toggle"
					class:dir-active={direction !== null}
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
						<!-- Swap icon (no direction chosen) -->
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
							<path d="M8 3L4 7l4 4" />
							<path d="M4 7h16" />
							<path d="M16 21l4-4-4-4" />
							<path d="M20 17H4" />
						</svg>
					{:else if direction === 'win_to_linux'}
						<!-- Right arrow (Win → Lin) -->
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M5 12h14" />
							<path d="M13 6l6 6-6 6" />
						</svg>
					{:else}
						<!-- Left arrow (Lin → Win) -->
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M19 12H5" />
							<path d="M11 18l-6-6 6-6" />
						</svg>
					{/if}
				</button>
				<span class="dir-label" class:dir-label-active={direction !== null}>
					{#if direction === 'win_to_linux'}
						W → L
					{:else if direction === 'linux_to_win'}
						L → W
					{:else}
						sync
					{/if}
				</span>
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
				<span class="keys-to-copy">
					{syncChanges.length} key{syncChanges.length !== 1 ? 's' : ''} to copy
				</span>
			{/if}
		</div>
		<div class="footer-actions">
			{#if onunlink}
				<button
					class="unlink-btn"
					onclick={() => onunlink?.()}
					title="Unmatch these devices"
				>
					<X class="h-3 w-3" />
					Unmatch
				</button>
			{/if}
			<button
				class="details-toggle"
				onclick={() => (expanded = !expanded)}
			>
				{expanded ? 'Hide' : 'Show'} details
			</button>
		</div>
	</div>

	<!-- Expanded key comparison table -->
	{#if expanded}
		<div class="details-panel">
			<table class="key-table">
				<thead>
					<tr>
						<th class="col-field">Field</th>
						<th class="col-value">Windows</th>
						<th class="col-value">Linux</th>
						<th class="col-status">Status</th>
					</tr>
				</thead>
				<tbody>
					{#each [...pair.comparison.linkKey, ...pair.comparison.leData].filter((f) => f.status !== 'both_missing') as field}
						{@const si = statusIcon(field.status)}
						<tr class:row-na={field.status === 'os_not_available'}>
							<td class="cell-field">{field.label}</td>
							<td class="cell-value">{truncateKey(field.windowsValue, field.status)}</td>
							<td class="cell-value">{truncateKey(field.linuxValue, field.status)}</td>
							<td class="cell-status">
								<si.icon class="inline h-3.5 w-3.5" style="color: {si.color}" />
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.pair-row {
		border-radius: 10px;
		border: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
	}

	.row-main {
		padding: 12px;
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
		width: 36px;
		height: 36px;
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
		border-color: rgba(255, 255, 255, 0.2);
		background: rgba(255, 255, 255, 0.06);
		color: rgba(250, 250, 250, 0.6);
	}

	.dir-toggle.dir-active {
		border-color: rgba(124, 58, 237, 0.4);
		background: linear-gradient(135deg, rgba(124, 58, 237, 0.2), rgba(59, 130, 246, 0.2));
		color: #e0d4ff;
	}

	.dir-toggle.dir-active:hover {
		border-color: rgba(124, 58, 237, 0.6);
		background: linear-gradient(135deg, rgba(124, 58, 237, 0.3), rgba(59, 130, 246, 0.3));
	}

	.dir-toggle.dir-synced {
		border-color: rgba(34, 197, 94, 0.3);
		background: rgba(34, 197, 94, 0.1);
		color: #4ade80;
		cursor: default;
	}

	.dir-label.dir-label-synced {
		color: rgba(74, 222, 128, 0.6);
	}

	.dir-label {
		font-size: 10px;
		font-weight: 500;
		color: rgba(250, 250, 250, 0.25);
		letter-spacing: 0.02em;
	}

	.dir-label.dir-label-active {
		color: rgba(224, 212, 255, 0.6);
	}

	/* Footer */
	.row-footer {
		padding: 0 12px 8px;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.unlink-btn {
		font-size: 12px;
		color: rgba(250, 250, 250, 0);
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

	.pair-row:hover .unlink-btn {
		color: rgba(250, 250, 250, 0.25);
	}

	.pair-row:hover .unlink-btn:hover {
		color: #ef4444;
	}

	.keys-to-copy {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
	}

	.details-toggle {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
		border: none;
		background: none;
		cursor: pointer;
		padding: 0;
		font-family: inherit;
		transition: color 0.2s;
	}

	.details-toggle:hover {
		color: rgba(250, 250, 250, 0.7);
	}

	/* Details panel */
	.details-panel {
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		padding: 8px 12px;
	}

	.key-table {
		width: 100%;
		font-size: 12px;
		border-collapse: collapse;
	}

	.key-table th {
		text-align: left;
		font-weight: 500;
		padding-bottom: 4px;
		color: rgba(250, 250, 250, 0.35);
	}

	.col-field {
		width: 25%;
	}

	.col-status {
		width: 48px;
		text-align: center !important;
	}

	.key-table tbody tr {
		border-top: 1px solid rgba(255, 255, 255, 0.04);
	}

	.cell-field {
		padding: 4px 0;
		color: rgba(250, 250, 250, 0.4);
	}

	.cell-value {
		padding: 4px 0;
		font-family: ui-monospace, monospace;
		word-break: break-all;
		color: rgba(250, 250, 250, 0.7);
	}

	.cell-status {
		padding: 4px 0;
		text-align: center;
	}

	.row-na {
		opacity: 0.45;
	}

</style>
