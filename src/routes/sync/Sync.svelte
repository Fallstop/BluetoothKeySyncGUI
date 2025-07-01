<script lang="ts" module>
	export type MatchedControllers = {windows: BluetoothController | null, linux: BluetoothController | null}[];
</script>

<script lang="ts">
	import type { BluetoothController, BluetoothData } from "#root/bindings";
	import ChangeRequest from "@/components/bluetooth/syncing/ChangeRequest.svelte";
	import DeviceCanvas from "@/components/bluetooth/syncing/nodeCanvas/DeviceCanvas.svelte";
	import { btStore } from "@/state";

	function matchControllers(windows: BluetoothData | null, linux: BluetoothData | null) {
		if (!windows || !linux) return [];
		const aControllers = windows.controllers || [];
		const bControllers = linux.controllers || [];

		let unmatchedBController = new Set(bControllers.map(controller => controller.address));

		let mapping: MatchedControllers = [];
		for (const aController of aControllers) {
			const matchedController = bControllers.find(bController => bController.address === aController.address);

			if (matchedController) {
				mapping.push({
					windows: aController,
					linux: matchedController
				});

				unmatchedBController.delete(matchedController.address);

			} else {
				mapping.push({
					windows: aController,
					linux: null
				});
			}
		}

		for (const bController of unmatchedBController) {
			const unmatchedController = bControllers.find(controller => controller.address === bController);
			if (unmatchedController) {
				mapping.push({
					windows: null,
					linux: unmatchedController
				});
			}
		}


		// We now have a mapping of controllers from both datasets
		// Let's sort them by the number of devices they have
		mapping.sort((a, b) => {
			const aDevices = (a.windows?.devices?.length || 0) + (a.linux?.devices?.length || 0);
			const bDevices = (b.windows?.devices?.length || 0) + (b.linux?.devices?.length || 0);

			return bDevices - aDevices; // Sort descending by number of devices
		});

		return mapping;
	}

	let matchedControllers = $derived(matchControllers(btStore.state.windows, btStore.state.linux))

</script>

<div class="p-4 mx-auto">
	<DeviceCanvas {matchedControllers} />
</div>


<ChangeRequest />
