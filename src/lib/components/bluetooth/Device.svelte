<script lang="ts">
	import type { BluetoothDevice } from "#root/bindings";
	import { Check, Computer, Smartphone } from "lucide-svelte";

	let { device }: { device: BluetoothDevice } = $props();

	function getDeviceIcon(deviceType: string) {
		return deviceType === "Classic" ? Computer : Smartphone;
	}
	let DeviceIcon = getDeviceIcon(device.device_type);
</script>

<div
	class="flex items-center gap-2 p-2 border rounded bg-muted/30"
	draggable="false"
>
	<DeviceIcon class="h-4 w-4" />
	<div class="flex-1">
		<div class="font-medium text-sm">
			{device.name || "Unknown Device"}
		</div>
		<div class="text-xs text-muted-foreground">
			{device.address} • {device.device_type}
		</div>
		{#if device.link_key}
			<div class="text-xs text-green-600 font-medium">
				<Check class="inline" /> Link Key Available
			</div>
		{/if}
		{#if device.le_data}
			<div class="text-xs text-blue-600 font-medium">
				<Check class="inline" /> LE Data Available
			</div>
		{/if}
	</div>
</div>
