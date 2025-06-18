<script lang="ts">
	import { Progress as ProgressPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		max = 100,
		value,
		...restProps
	}: WithoutChildrenOrChild<ProgressPrimitive.RootProps> = $props();
</script>

<ProgressPrimitive.Root
	bind:ref
	data-slot="progress"
	class={cn("bg-primary/20 relative h-2 w-full overflow-hidden rounded-full", className)}
	{value}
	{max}
	{...restProps}
>
	<div
		data-slot="progress-indicator"
		class="bg-primary h-full w-full flex-1 transition-all {value === null ? 'indeterminate-progress' : ''}"
		style="{value === null ? '' : `transform: translateX(-${100 - (100 * (value ?? 0)) / (max ?? 1)}%)`}"
	></div>
</ProgressPrimitive.Root>

<style lang="scss">
	.indeterminate-progress {
		animation: indeterminate-progress 2s ease-in-out infinite;
	}

	@keyframes indeterminate-progress {
			0% {
					transform: translateX(-100%);
			}
			25% {
					scale: 1;
			}
			50% {
					scale: 0.5 1;
					transform: translateX(0%);
			}
			75% {
					scale: 1;
			}
			100% {
					transform: translateX(100%);
			}
	}

</style>
