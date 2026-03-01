<script lang="ts">
	import type { BluetoothDevice, HostDistributions } from '#root/bindings';
	import { osColor } from './os-theme';
	import KeyIndicators from './KeyIndicators.svelte';

	let {
		device,
		os,
		side,
		isDragSource = false,
		isDragTarget = false,
		canDrag = false,
		onpointerdown,
		onpointermove,
		onpointerup,
		onpointercancel
	}: {
		device: BluetoothDevice;
		os: HostDistributions;
		side: 'left' | 'right';
		isDragSource?: boolean;
		isDragTarget?: boolean;
		canDrag?: boolean;
		onpointerdown?: (e: PointerEvent) => void;
		onpointermove?: (e: PointerEvent) => void;
		onpointerup?: (e: PointerEvent) => void;
		onpointercancel?: () => void;
	} = $props();

	let colors = $derived(osColor(os));
	let pairSide = $derived(os === 'Windows' ? 'win' : 'lin');
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	data-pair-side={pairSide}
	class="mini-card"
	class:side-left={side === 'left'}
	class:side-right={side === 'right'}
	class:drag-source={isDragSource}
	class:drag-target={isDragTarget}
	class:can-drag={canDrag}
	style="--os-border: {colors.borderColor}; --os-ring: {colors.ringColor}; --os-ring-hover: {colors.ringHoverColor}; --os-gradient: {colors.accentGradient}"
	{onpointerdown}
	{onpointermove}
	{onpointerup}
	{onpointercancel}
	onlostpointercapture={onpointercancel}
>
	<div class="device-name" class:text-right={side === 'right'}>
		{device.name ?? 'Unknown Device'}
	</div>
	<div class="device-mac" class:text-right={side === 'right'}>
		{device.address}
	</div>
	<div class="indicator-row">
		<KeyIndicators {device} align={side === 'right' ? 'right' : 'left'} />
	</div>
</div>

<style lang="css">
	.mini-card {
		flex: 1;
		min-width: 0;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.06);
		padding: 10px 14px;
		transition: all 0.2s;
		position: relative;
		overflow: hidden;
	}

	.mini-card.side-left {
		border-left: 2.5px solid var(--os-border);
	}

	.mini-card.side-right {
		border-right: 2.5px solid var(--os-border);
	}

	.mini-card.can-drag {
		cursor: grab;
	}

	.mini-card.can-drag:active {
		cursor: grabbing;
	}

	.mini-card.drag-source {
		box-shadow: 0 0 0 2px var(--os-ring);
		transform: scale(0.98);
	}

	.mini-card.drag-target {
		box-shadow: 0 0 0 2px var(--os-ring-hover);
		transform: scale(1.02);
	}

	.device-name {
		font-weight: 500;
		font-size: 14px;
		color: rgba(250, 250, 250, 0.85);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.device-name.text-right {
		text-align: right;
	}

	.device-mac {
		font-size: 11px;
		font-family: ui-monospace, monospace;
		color: rgba(250, 250, 250, 0.35);
		margin-top: 2px;
	}

	.device-mac.text-right {
		text-align: right;
	}

	.indicator-row {
		margin-top: 6px;
	}
</style>
