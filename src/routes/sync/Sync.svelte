<script lang="ts" module>
	export type MatchedControllers = {windows: BluetoothController | null, linux: BluetoothController | null}[];
</script>

<script lang="ts">
	import type { BluetoothController, BluetoothData } from "#root/bindings";
	import Controller from "@/components/bluetooth/Controller.svelte";
	import DeviceModel from "@/components/bluetooth/syncing/DeviceCanvas.svelte";
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

	let matched_controllers = $derived(matchControllers(btStore.state.windows, btStore.state.linux))

</script>

<DeviceModel/>

<div class="grid grid-cols-2 p-4 gap-y-4 gap-x-16 max-w-6xl mx-auto">
	<div>
		<h2 class="text-xl font-bold mb-4">Windows Bluetooth Devices</h2>
	</div>

	<div>
		<h2 class="text-xl font-bold mb-4">Linux Bluetooth Devices</h2>
	</div>

	{#each matched_controllers as { windows, linux }}
		{#if windows}
			<Controller class="self-baseline" controller={windows} />
		{:else}
			<div class="text-muted-foreground self-center">No matching Windows controller</div>
		{/if}
		{#if linux}
			<Controller class="self-baseline" controller={linux} />
		{:else}
			<div class="text-muted-foreground">No matching Linux controller</div>
		{/if}
	{/each}

	{#if matched_controllers.length === 0}
		<div class="text-muted-foreground col-span-2">
			No controllers found between Windows and Linux data.
		</div>
	{/if}

</div>
