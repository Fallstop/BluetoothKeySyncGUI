<script lang="ts">
	import type {
		MatchResult,
		ManualMatch,
		SyncSelections,
		SyncDirection,
		UnmatchedDevice
	} from './matching';
	import { pairKey, manualPairKey, deviceKey } from './matching';
	import DevicePairRow from './DevicePairRow.svelte';
	import UnpairedDeviceCard from './UnpairedDeviceCard.svelte';
	import DragOverlay from './DragOverlay.svelte';
	import { osColor } from './os-theme';

	let {
		matchResult,
		manualMatches,
		unpairedDevices,
		deletions,
		selections = $bindable(),
		onmanualmatch,
		onunlink,
		onautounlink,
		ontoggledelete
	}: {
		matchResult: MatchResult;
		manualMatches: ManualMatch[];
		unpairedDevices: UnmatchedDevice[];
		deletions: Set<string>;
		selections: SyncSelections;
		onmanualmatch?: (winDevice: UnmatchedDevice, linDevice: UnmatchedDevice, direction?: SyncDirection) => void;
		onunlink?: (matchId: string) => void;
		onautounlink?: (pairKey: string) => void;
		ontoggledelete?: (device: UnmatchedDevice) => void;
	} = $props();

	let selectedDevice = $state<UnmatchedDevice | null>(null);

	// --- Drag state ---
	const DRAG_THRESHOLD = 5;
	let dragPotential = $state<{
		device: UnmatchedDevice;
		startX: number;
		startY: number;
	} | null>(null);
	let isDragging = $state(false);
	let dragPos = $state({ x: 0, y: 0 });
	let dragStartPos = $state({ x: 0, y: 0 });
	let dragHoverDevice = $state<UnmatchedDevice | null>(null);
	let suppressClick = $state(false);

	// Align unpaired devices: match by MAC, then name, then remainder
	let alignedRows = $derived.by(() => {
		const win = unpairedDevices.filter((d) => d.os === 'Windows');
		const lin = unpairedDevices.filter((d) => d.os === 'Linux');
		const rows: Array<{ windows: UnmatchedDevice | null; linux: UnmatchedDevice | null }> = [];
		const usedWin = new Set<number>();
		const usedLin = new Set<number>();

		// Pass 1: Match by MAC address
		for (let wi = 0; wi < win.length; wi++) {
			if (usedWin.has(wi)) continue;
			const mac = win[wi].device.address.toUpperCase();
			for (let li = 0; li < lin.length; li++) {
				if (usedLin.has(li)) continue;
				if (lin[li].device.address.toUpperCase() === mac) {
					rows.push({ windows: win[wi], linux: lin[li] });
					usedWin.add(wi);
					usedLin.add(li);
					break;
				}
			}
		}

		// Pass 2: Match by name
		for (let wi = 0; wi < win.length; wi++) {
			if (usedWin.has(wi)) continue;
			const name = win[wi].device.name?.toLowerCase();
			if (!name) continue;
			for (let li = 0; li < lin.length; li++) {
				if (usedLin.has(li)) continue;
				if (lin[li].device.name?.toLowerCase() === name) {
					rows.push({ windows: win[wi], linux: lin[li] });
					usedWin.add(wi);
					usedLin.add(li);
					break;
				}
			}
		}

		// Pass 3: Remaining unmatched
		const remWin = win.filter((_, i) => !usedWin.has(i));
		const remLin = lin.filter((_, i) => !usedLin.has(i));
		const max = Math.max(remWin.length, remLin.length);
		for (let i = 0; i < max; i++) {
			rows.push({ windows: remWin[i] ?? null, linux: remLin[i] ?? null });
		}

		return rows;
	});

	let hasManualPairs = $derived(manualMatches.length > 0);
	let hasUnpaired = $derived(unpairedDevices.length > 0);
	let hasSynced = $derived(matchResult.alreadySynced.length > 0);
	let hasMatchedSection = $derived(hasSynced || hasManualPairs);

	// Cursor management during drag
	$effect(() => {
		if (isDragging) {
			document.body.style.cursor = 'grabbing';
			document.body.style.userSelect = 'none';
			return () => {
				document.body.style.cursor = '';
				document.body.style.userSelect = '';
			};
		}
	});

	function updateDirection(key: string, direction: SyncDirection) {
		const existing = selections.get(key);
		if (existing) {
			selections.set(key, { ...existing, direction });
			selections = new Map(selections);
		}
	}

	// --- Click handling ---
	function handleUnpairedClick(device: UnmatchedDevice) {
		if (suppressClick) {
			suppressClick = false;
			return;
		}
		if (!selectedDevice) {
			selectedDevice = device;
		} else if (selectedDevice === device) {
			selectedDevice = null;
		} else if (selectedDevice.os === device.os) {
			selectedDevice = device;
		} else {
			const winDevice = device.os === 'Windows' ? device : selectedDevice;
			const linDevice = device.os === 'Linux' ? device : selectedDevice;
			onmanualmatch?.(winDevice, linDevice);
			selectedDevice = null;
		}
	}

	function handleContainerClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('[data-unpaired-device]')) {
			selectedDevice = null;
		}
	}

	// --- Drag handling ---
	function getCardCenter(device: UnmatchedDevice): { x: number; y: number } {
		const key = `${device.controllerAddress}/${device.device.address}`;
		const el = document.querySelector(
			`[data-unpaired-key="${key}"][data-unpaired-os="${device.os}"]`
		);
		if (!el) return { x: 0, y: 0 };
		const rect = el.getBoundingClientRect();
		return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
	}

	function findDeviceAtPoint(x: number, y: number): UnmatchedDevice | null {
		const el = document.elementFromPoint(x, y);
		const card = el?.closest('[data-unpaired-device]') as HTMLElement | null;
		if (!card) return null;
		const os = card.dataset.unpairedOs;
		const key = card.dataset.unpairedKey;
		if (!os || !key) return null;
		return (
			unpairedDevices.find(
				(d) => d.os === os && `${d.controllerAddress}/${d.device.address}` === key
			) ?? null
		);
	}

	function handleCardPointerDown(device: UnmatchedDevice, e: PointerEvent) {
		if (e.button !== 0) return;
		dragPotential = {
			device,
			startX: e.clientX,
			startY: e.clientY
		};
		isDragging = false;
	}

	function handleWindowPointerMove(e: PointerEvent) {
		if (!dragPotential) return;

		const dx = e.clientX - dragPotential.startX;
		const dy = e.clientY - dragPotential.startY;

		if (!isDragging) {
			if (Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
				isDragging = true;
				selectedDevice = null;
				dragStartPos = getCardCenter(dragPotential.device);
			} else {
				return;
			}
		}

		dragPos = { x: e.clientX, y: e.clientY };

		const target = findDeviceAtPoint(e.clientX, e.clientY);
		if (target && target !== dragPotential.device && target.os !== dragPotential.device.os) {
			dragHoverDevice = target;
		} else {
			dragHoverDevice = null;
		}
	}

	function handleWindowPointerUp(_e: PointerEvent) {
		if (!dragPotential) return;

		if (isDragging) {
			suppressClick = true;
			requestAnimationFrame(() => {
				suppressClick = false;
			});

			if (dragHoverDevice && dragPotential.device.os !== dragHoverDevice.os) {
				const source = dragPotential.device;
				const target = dragHoverDevice;
				const winDevice = source.os === 'Windows' ? source : target;
				const linDevice = source.os === 'Linux' ? source : target;
				const direction: SyncDirection =
					source.os === 'Windows' ? 'win_to_linux' : 'linux_to_win';
				onmanualmatch?.(winDevice, linDevice, direction);
			}
		}

		dragPotential = null;
		isDragging = false;
		dragHoverDevice = null;
	}
</script>

<DragOverlay
	{isDragging}
	{dragPotential}
	{dragStartPos}
	{dragPos}
	{dragHoverDevice}
	{getCardCenter}
	onpointermove={handleWindowPointerMove}
	onpointerup={handleWindowPointerUp}
/>

<div class="space-y-4">
	<!-- Matched Devices: synced pairs (readonly) + manual matches (editable) -->
	{#if hasMatchedSection}
		<div>
			<h3 class="text-sm font-medium text-muted-foreground mb-2">Matched Devices</h3>
			<div class="space-y-2">
				{#each matchResult.alreadySynced as pair (pairKey(pair.controllerAddress, pair.windowsDevice.address))}
					<DevicePairRow {pair} readonly />
				{/each}
				{#each manualMatches as match (match.id)}
					{@const key = manualPairKey(match.id)}
					{@const sel = selections.get(key)}
					<DevicePairRow
						pair={match}
						direction={sel?.direction ?? null}
						isManual
						ondirectionchange={(d) => updateDirection(key, d)}
						onunlink={() => onunlink?.(match.id)}
					/>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Unpaired Devices (two-column grid) -->
	{#if hasUnpaired}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div onclick={handleContainerClick}>
			<h3 class="text-sm font-medium text-muted-foreground mb-1">Devices</h3>
			<p class="text-xs text-muted-foreground mb-3">
				Click or drag a device to one on the other side to pair them.
			</p>
			<div class="grid grid-cols-2 gap-x-3 gap-y-2">
				<!-- Column headers -->
				<div class="text-xs font-medium {osColor('Windows').text}">
					Windows
				</div>
				<div class="text-xs font-medium {osColor('Linux').text}">
					Linux
				</div>

				<!-- Aligned device rows -->
				{#each alignedRows as row (
					(row.windows ? row.windows.controllerAddress + '/' + row.windows.device.address : '') +
					'|' +
					(row.linux ? row.linux.controllerAddress + '/' + row.linux.device.address : '')
				)}
					<div>
						{#if row.windows}
							{@const device = row.windows}
							{@const isDeleted = deletions.has(deviceKey(device.os, device.controllerAddress, device.device.address))}
							<UnpairedDeviceCard
								{device}
								selected={!isDeleted && selectedDevice === device}
								selectionActive={selectedDevice !== null}
								isTarget={!isDeleted && selectedDevice !== null &&
									selectedDevice.os !== device.os}
								isDragOver={!isDeleted && dragHoverDevice === device}
								markedForDeletion={isDeleted}
								onclick={() => handleUnpairedClick(device)}
								onpointerdown={(e) => handleCardPointerDown(device, e)}
								ondelete={() => ontoggledelete?.(device)}
							/>
						{/if}
					</div>
					<div>
						{#if row.linux}
							{@const device = row.linux}
							{@const isDeleted = deletions.has(deviceKey(device.os, device.controllerAddress, device.device.address))}
							<UnpairedDeviceCard
								{device}
								selected={!isDeleted && selectedDevice === device}
								selectionActive={selectedDevice !== null}
								isTarget={!isDeleted && selectedDevice !== null &&
									selectedDevice.os !== device.os}
								isDragOver={!isDeleted && dragHoverDevice === device}
								markedForDeletion={isDeleted}
								onclick={() => handleUnpairedClick(device)}
								onpointerdown={(e) => handleCardPointerDown(device, e)}
								ondelete={() => ontoggledelete?.(device)}
							/>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Empty state -->
	{#if !hasMatchedSection && !hasUnpaired}
		<div class="text-center py-12 text-muted-foreground">
			<p class="text-sm">
				No devices found. Load both Windows and Linux Bluetooth data first.
			</p>
		</div>
	{/if}
</div>
