<script module lang="ts">
	import type { BluetoothDevice } from '#root/bindings';
	import { Handle, type Node, type NodeProps } from '@xyflow/svelte';

  export type ControllerNodeType = Node<{ controller: BluetoothController, device_length: number }, 'device'>;

	const generalMargin = 1.25;
	const headerHeight = 2 + generalMargin *2;
	export const controllerGroupDimension = {
		width: deviceDimension.width + generalMargin*2,
		height: 1 + headerHeight,
		headerHeight,
		generalMargin
	}
</script>

<script lang="ts">
	import type { BluetoothController, BluetoothData } from "#root/bindings";
	import {
		Bluetooth,
	} from "lucide-svelte";
	import { slide } from "svelte/transition";
	import Device from "../Device.svelte";
	import { deviceDimension } from './DeviceNode.svelte';

  let { id, data }: NodeProps<ControllerNodeType> = $props();

</script>

<div class="border rounded-lg -z-10" style="width: {controllerGroupDimension.width}rem">
	<div
		class="flex items-center gap-2 w-full text-left rounded p-5 -m-2"
	>
		<Bluetooth class="h-4 w-4" />
		<div>
			<div class="font-medium">
				{data.controller.name || "Bluetooth Controller"}
			</div>
			<div class="text-sm text-muted-foreground">
				{data.controller.address} • {data.controller.devices.length} device(s)
			</div>
		</div>
	</div>

	<div
		class="m-4 space-y-4"
		transition:slide={{ axis: "y", duration: 200 }}
	>
	<div style="height: {data.device_length * deviceDimension.height}rem;"></div>
	</div>
</div>
