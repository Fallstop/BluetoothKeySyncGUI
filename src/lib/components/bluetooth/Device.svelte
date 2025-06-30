<script lang="ts">
	import type { BluetoothDevice } from "#root/bindings";
	import { Check, Computer, Smartphone } from "lucide-svelte";

	let { device, style, class: userClass }: { device: BluetoothDevice, class?: string, style?: string } = $props();

	function getDeviceIcon(deviceType: string) {
		return deviceType === "Classic" ? Computer : Smartphone;
	}
	let DeviceIcon = getDeviceIcon(device.device_type);
</script>

<div
	class="flex items-center gap-2 p-2 border rounded bg-muted/30 {userClass}"
	style={style}
	draggable="true"
>
	<DeviceIcon class="h-4 w-4" />
	<div class="flex-1">
		<div class="font-medium text-sm">
			{device.name || "Unknown Device"}
		</div>
		<div class="text-xs text-muted-foreground">
			{device.address} • {device.device_type}
		</div>
		<div class="flex flex-row gap-4">
			{#if device.link_key}
				<span class="text-xs text-green-600 font-medium">
					<Check class="inline" /> Link Key
				</span>
			{/if}
			{#if device.le_data}
				<span class="text-xs text-blue-600 font-medium">
					<Check class="inline" /> LE Data
				</span>
			{/if}

		</div>
	</div>
</div>
