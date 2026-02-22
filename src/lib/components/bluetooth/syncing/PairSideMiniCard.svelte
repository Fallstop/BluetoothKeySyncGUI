<script lang="ts">
	import type { BluetoothDevice, HostDistributions } from '#root/bindings';
	import { osColor } from './os-theme';
	import KeyIndicators from './KeyIndicators.svelte';

	let {
		device,
		os,
		side,
		dragClass = '',
		canDrag = false,
		onpointerdown,
		onpointermove,
		onpointerup,
		onpointercancel
	}: {
		device: BluetoothDevice;
		os: HostDistributions;
		side: 'left' | 'right';
		dragClass?: string;
		canDrag?: boolean;
		onpointerdown?: (e: PointerEvent) => void;
		onpointermove?: (e: PointerEvent) => void;
		onpointerup?: (e: PointerEvent) => void;
		onpointercancel?: () => void;
	} = $props();

	let colors = $derived(osColor(os));
	let borderClass = $derived(side === 'left' ? `border-l-4 ${colors.borderL}` : `border-r-4 ${colors.borderR}`);
	let textAlign = $derived(side === 'right' ? 'text-right' : '');
	let indicatorJustify = $derived(side === 'right' ? 'justify-end' : '');
	let pairSide = $derived(os === 'Windows' ? 'win' : 'lin');
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	data-pair-side={pairSide}
	class="flex-1 min-w-0 rounded-md {borderClass} bg-muted/30 px-3 py-2 transition-all {dragClass} {canDrag ? 'cursor-grab active:cursor-grabbing' : ''}"
	{onpointerdown}
	{onpointermove}
	{onpointerup}
	{onpointercancel}
	onlostpointercapture={onpointercancel}
>
	<div class="font-medium text-sm truncate {textAlign}">
		{device.name ?? 'Unknown Device'}
	</div>
	<KeyIndicators {device} class="mt-0.5 {indicatorJustify}" />
</div>
