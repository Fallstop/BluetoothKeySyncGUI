<script lang="ts">
	import type {
		MatchedDevicePair,
		ManualMatch,
		SyncDirection,
		KeyFieldComparison
	} from './matching';
	import { describeSyncChanges } from './matching';
	import { Check, X, AlertTriangle, Minus, ArrowRight } from 'lucide-svelte';
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

	let winDragClass = $derived(
		draggingFromSide === 'win'
			? `ring-2 ${osColor('Windows').ringActive}`
			: dragHoverSide === 'win' && draggingFromSide === 'lin'
				? `ring-2 ${osColor('Windows').ringHover}`
				: ''
	);

	let linDragClass = $derived(
		draggingFromSide === 'lin'
			? `ring-2 ${osColor('Linux').ringActive}`
			: dragHoverSide === 'lin' && draggingFromSide === 'win'
				? `ring-2 ${osColor('Linux').ringHover}`
				: ''
	);

	// --- Helpers ---
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

<div class="rounded-lg border bg-card text-card-foreground" bind:this={rowEl}>
	<!-- Main row: Win card — direction — Lin card -->
	<div class="p-3 flex items-center gap-3">
		<!-- Windows device mini-card -->
		<PairSideMiniCard
			device={pair.windowsDevice}
			os="Windows"
			side="left"
			dragClass={winDragClass}
			{canDrag}
			onpointerdown={(e) => handleSidePointerDown('win', e)}
			onpointermove={handleSidePointerMove}
			onpointerup={handleSidePointerUp}
			onpointercancel={handleSidePointerCancel}
		/>

		<!-- Center: Direction picker or synced indicator -->
		<div class="flex flex-col items-center gap-1 shrink-0">
			{#if isSynced}
				<div
					class="flex items-center gap-1 text-xs text-green-600 dark:text-green-400 font-medium"
				>
					<Check class="h-4 w-4" />
					<span>synced</span>
				</div>
			{:else if !readonly}
				<div class="inline-flex rounded-md border border-input text-xs">
					<button
						class="px-2 py-1.5 rounded-l-md transition-colors {direction === 'win_to_linux'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted text-muted-foreground'}"
						onclick={() => ondirectionchange?.('win_to_linux')}
						title="Copy Windows keys to Linux"
					>
						<ArrowRight class="h-3 w-3" />
					</button>
					<button
						class="px-2 py-1.5 rounded-r-md transition-colors {direction === 'linux_to_win'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted text-muted-foreground'}"
						onclick={() => ondirectionchange?.('linux_to_win')}
						title="Copy Linux keys to Windows"
					>
						<ArrowRight class="h-3 w-3 rotate-180" />
					</button>
				</div>
				{#if direction === null}
					<span class="text-[10px] text-muted-foreground/60">pick direction</span>
				{/if}
			{/if}
		</div>

		<!-- Linux device mini-card -->
		<PairSideMiniCard
			device={pair.linuxDevice}
			os="Linux"
			side="right"
			dragClass={linDragClass}
			{canDrag}
			onpointerdown={(e) => handleSidePointerDown('lin', e)}
			onpointermove={handleSidePointerMove}
			onpointerup={handleSidePointerUp}
			onpointercancel={handleSidePointerCancel}
		/>

		<!-- Unmatch button -->
		{#if onunlink}
			<button
				class="shrink-0 p-1.5 rounded-md text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-colors"
				onclick={() => onunlink?.()}
				title="Unmatch these devices"
			>
				<X class="h-4 w-4" />
			</button>
		{/if}
	</div>

	<!-- Bottom bar: sync info + details toggle -->
	<div class="px-3 pb-2 flex items-center justify-between">
		<div class="flex items-center gap-2">
			{#if direction && syncChanges.length > 0 && !isSynced}
				<span class="text-xs text-muted-foreground">
					{syncChanges.length} key{syncChanges.length !== 1 ? 's' : ''} to copy
				</span>
			{/if}
		</div>
		<button
			class="text-xs text-muted-foreground hover:text-foreground transition-colors"
			onclick={() => (expanded = !expanded)}
		>
			{expanded ? 'Hide' : 'Show'} details
		</button>
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
					{#each [...pair.comparison.linkKey, ...pair.comparison.leData].filter((f) => f.status !== 'both_missing') as field}
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
