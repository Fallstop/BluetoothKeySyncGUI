<script lang="ts">
	import "@xyflow/svelte/dist/style.css";
	import { ConnectionLineType, MarkerType, SvelteFlow, SvelteFlowProvider } from "@xyflow/svelte";
	import type { MatchedControllers } from "#root/src/routes/sync/Sync.svelte";
	import type { BluetoothController, BluetoothDevice } from "#root/bindings";

	import DeviceNode, { deviceDimension } from "./DeviceNode.svelte";
	import FloatingEdge from "./FloatingEdge.svelte";
	import ControllerGroupNode, { controllerGroupDimension } from "./ControllerGroupNode.svelte";
	import { remToPixels } from "./utils";

	let { matchedControllers }: { matchedControllers: MatchedControllers } =
		$props();

	const nodeTypes = { deviceNode: DeviceNode, controllerNode: ControllerGroupNode };

	function deviceColumn(controller: BluetoothController | null, x_initial: number, y_initial: number) {
		if (!controller) {
			return [];
		}

		let devices = (controller?.devices || []);

		let header_node = {
			id: `${x_initial}-header`,
			type: "controllerNode",
			data: { controller, device_length: devices.length },
			position: { x: remToPixels(x_initial), y: remToPixels(y_initial) },
		};

		const headerOffset = controllerGroupDimension.headerHeight;

		return [
			header_node,
			... devices.map((device, index) => ({
				id: `${x_initial}-${index}`,
				type: "deviceNode",
				data: { device },
				dragHandle: ".custom-drag-handle",
				position: { x: remToPixels(x_initial + controllerGroupDimension.generalMargin), y: remToPixels(y_initial + headerOffset + (deviceDimension.height  + controllerGroupDimension.generalMargin) * index)},
				parentNode: header_node.id,
			}))
		];
	}

	function deriveNodes(controllers: MatchedControllers) {
		return controllers.flatMap((controller, index) => {
			const y_initial = 0;

			const interColumnSpacing = 4 + controllerGroupDimension.width;

			return [
				...deviceColumn(controller.windows, 0, y_initial),
				...deviceColumn(controller.linux, interColumnSpacing, y_initial),
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

<div style:width="100vw" style:height="50vh">
	<SvelteFlowProvider>
		<!-- maxZoom={1}
		minZoom={1}
		panOnDrag={false}
		selectionOnDrag={false} -->
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
