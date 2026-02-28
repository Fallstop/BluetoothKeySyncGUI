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
</script>

<svelte:window {onpointermove} {onpointerup} />

{#if isDragging && dragPotential}
	<svg
		class="fixed inset-0 pointer-events-none"
		style="width: 100vw; height: 100vh; z-index: 50;"
	>
		<defs>
			<filter id="drag-glow" x="-50%" y="-50%" width="200%" height="200%">
				<feGaussianBlur in="SourceGraphic" stdDeviation="3" result="blur" />
				<feMerge>
					<feMergeNode in="blur" />
					<feMergeNode in="SourceGraphic" />
				</feMerge>
			</filter>
		</defs>
		{#if dragHoverDevice}
			{@const targetCenter = getCardCenter(dragHoverDevice)}
			<line
				x1={dragStartPos.x}
				y1={dragStartPos.y}
				x2={targetCenter.x}
				y2={targetCenter.y}
				stroke={lineColor}
				stroke-width="2.5"
				opacity="0.9"
				filter="url(#drag-glow)"
			/>
			<circle
				cx={targetCenter.x}
				cy={targetCenter.y}
				r="4"
				fill={lineColor}
				opacity="0.9"
				filter="url(#drag-glow)"
			/>
		{:else}
			<line
				x1={dragStartPos.x}
				y1={dragStartPos.y}
				x2={dragPos.x}
				y2={dragPos.y}
				stroke={lineColor}
				stroke-width="2"
				stroke-dasharray="6 3"
				opacity="0.6"
			/>
			<circle cx={dragPos.x} cy={dragPos.y} r="3" fill={lineColor} opacity="0.6" />
		{/if}
		<circle
			cx={dragStartPos.x}
			cy={dragStartPos.y}
			r="4"
			fill={lineColor}
			opacity="0.9"
			filter="url(#drag-glow)"
		/>
	</svg>
{/if}
