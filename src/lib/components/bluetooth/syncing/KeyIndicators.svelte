<script lang="ts">
	import type { BluetoothDevice } from '#root/bindings';
	import { Check, X } from 'lucide-svelte';

	let {
		device,
		align = 'left'
	}: {
		device: BluetoothDevice;
		align?: 'left' | 'right';
	} = $props();
</script>

<div class="indicators" class:align-right={align === 'right'}>
	{#if device.link_key}
		<span class="key-present">
			<Check class="inline h-3 w-3" /> Link
		</span>
	{:else}
		<span class="key-absent">
			<X class="inline h-3 w-3" /> Link
		</span>
	{/if}
	{#if device.le_data}
		<span class="key-present">
			<Check class="inline h-3 w-3" /> LE
		</span>
	{:else}
		<span class="key-absent">
			<X class="inline h-3 w-3" /> LE
		</span>
	{/if}
</div>

<style lang="css">
	.indicators {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
	}

	.indicators.align-right {
		justify-content: flex-end;
	}

	.key-present {
		color: #4ade80;
	}

	.key-absent {
		color: rgba(250, 250, 250, 0.2);
	}
</style>
