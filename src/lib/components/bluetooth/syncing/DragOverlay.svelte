<script lang="ts">
	import type { UnmatchedDevice } from './matching';
	import { osColor } from './os-theme';

	let {
		isDragging,
		dragPotential,
		dragStartPos,
		dragPos,
		dragHoverDevice,
		getCardCenter,
		onpointermove,
		onpointerup
	}: {
		isDragging: boolean;
		dragPotential: { device: UnmatchedDevice } | null;
		dragStartPos: { x: number; y: number };
		dragPos: { x: number; y: number };
		dragHoverDevice: UnmatchedDevice | null;
		getCardCenter: (device: UnmatchedDevice) => { x: number; y: number };
		onpointermove: (e: PointerEvent) => void;
		onpointerup: (e: PointerEvent) => void;
	} = $props();

	let lineColor = $derived(dragPotential ? osColor(dragPotential.device.os).hex : '#3b82f6');

	function bezierPath(x1: number, y1: number, x2: number, y2: number): string {
		// Horizontal offset for control points, clamped so the curve never overshoots
		const dist = Math.abs(x2 - x1);
		const offset = Math.min(dist * 0.4, 120);
		const sign = x2 >= x1 ? 1 : -1;
		return `M ${x1} ${y1} C ${x1 + offset * sign} ${y1}, ${x2 - offset * sign} ${y2}, ${x2} ${y2}`;
	}
</script>

<svelte:window {onpointermove} {onpointerup} />

{#if isDragging && dragPotential}
	<svg
		class="fixed inset-0 pointer-events-none"
		style="width: 100vw; height: 100vh; z-index: 50;"
	>
		{#if dragHoverDevice}
			{@const targetCenter = getCardCenter(dragHoverDevice)}
			<path
				d={bezierPath(dragStartPos.x, dragStartPos.y, targetCenter.x, targetCenter.y)}
				fill="none"
				stroke={lineColor}
				stroke-width="2.5"
				opacity="0.85"
				stroke-dasharray="8 4"
				class="flowing-dash"
			/>
			<circle
				cx={targetCenter.x}
				cy={targetCenter.y}
				r="4"
				fill={lineColor}
				opacity="0.85"
			/>
		{:else}
			<path
				d={bezierPath(dragStartPos.x, dragStartPos.y, dragPos.x, dragPos.y)}
				fill="none"
				stroke={lineColor}
				stroke-width="2"
				stroke-dasharray="6 3"
				opacity="0.5"
			/>
			<circle cx={dragPos.x} cy={dragPos.y} r="3" fill={lineColor} opacity="0.5" />
		{/if}
		<circle
			cx={dragStartPos.x}
			cy={dragStartPos.y}
			r="4"
			fill={lineColor}
			opacity="0.85"
		/>
	</svg>
{/if}

<style>
	.flowing-dash {
		animation: dash-flow 0.6s linear infinite;
	}

	@keyframes dash-flow {
		to {
			stroke-dashoffset: -12;
		}
	}
</style>
