<script lang="ts">
	import "@xyflow/svelte/dist/style.css";
	import { ConnectionLineType, MarkerType, SvelteFlow, SvelteFlowProvider } from "@xyflow/svelte";
	import type { MatchedControllers } from "#root/src/routes/sync/Sync.svelte";
	import type { BluetoothDevice } from "#root/bindings";

	import DeviceNode from "./DeviceNode.svelte";
	import FloatingEdge from "./FloatingEdge.svelte";

	let { matchedControllers }: { matchedControllers: MatchedControllers } =
		$props();

	const nodeTypes = { deviceNode: DeviceNode };

	function deviceColumn(devices: BluetoothDevice[], x_initial: number, y_initial: number) {
		return devices.map((device, index) => ({
			id: `${x_initial}-${index}`,
			type: "deviceNode",
			data: { device },
			dragHandle: ".custom-drag-handle",
			position: { x: x_initial, y: y_initial + 100 * index },
		}));
	}

	function deriveNodes(controllers: MatchedControllers) {
		return controllers.flatMap((controller, index) => {
			const x_initial = index * 100; // Adjust the spacing between controllers
			const y_initial = 0;

			let windowsDevices = controller.windows?.devices || [];
			let linuxDevices = controller.linux?.devices || [];

			return [
				...deviceColumn(windowsDevices, 0, y_initial + 100),
				...deviceColumn(linuxDevices, 250, y_initial + 100),
			];
		});
	}

	const edgeTypes = {
    floating: FloatingEdge,
  };

	const defaultEdgeOptions = {
    type: 'floating',
    markerEnd: {
      type: MarkerType.ArrowClosed,
    },
  };

	let nodes = $state.raw(deriveNodes(matchedControllers));

	let edges = $state.raw([]);
</script>

<!-- maxZoom={1}
minZoom={1}
panOnDrag={false}
selectionOnDrag={false} -->
<div style:width="100vw" style:height="50vh">
	<SvelteFlowProvider>
		<SvelteFlow
			bind:nodes
			bind:edges
			fitView
			viewport={{ x: 0, y: 0, zoom: 1 }}
			{edgeTypes}
			{defaultEdgeOptions}
			connectionLineType={ConnectionLineType.Straight}
			{nodeTypes}
			class="bg-background!"
		/>
	</SvelteFlowProvider>
</div>
