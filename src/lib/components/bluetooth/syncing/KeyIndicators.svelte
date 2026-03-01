<script lang="ts">
	import type { BluetoothDevice } from '#root/bindings';

	let {
		device,
		align = 'left'
	}: {
		device: BluetoothDevice;
		align?: 'left' | 'right';
	} = $props();
</script>

<div class="indicators" class:align-right={align === 'right'}>
	<span class="pill" class:pill-present={!!device.link_key}>Link</span>
	<span class="pill" class:pill-present={!!device.le_data}>LE</span>
</div>

<style lang="css">
	.indicators {
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.indicators.align-right {
		justify-content: flex-end;
	}

	.pill {
		display: inline-block;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.02em;
		padding: 1px 7px;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.04);
		color: rgba(250, 250, 250, 0.2);
		border: 1px solid rgba(255, 255, 255, 0.04);
		line-height: 1.4;
	}

	.pill.pill-present {
		background: rgba(34, 197, 94, 0.1);
		color: #4ade80;
		border-color: rgba(34, 197, 94, 0.2);
	}
</style>
