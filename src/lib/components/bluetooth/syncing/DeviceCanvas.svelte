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
			return {
				nodes: [],
				total_height: 0
			};
		}

		let devices = (controller?.devices || []);

		let device_height = (devices.length * (deviceDimension.height + controllerGroupDimension.generalMargin));

		let header_node = {
			id: `controller-${controller.address}-${x_initial}`,
			type: "controllerNode",
			dragHandle: ".custom-drag-handle",
			data: { controller, device_height: device_height },
			position: { x: remToPixels(x_initial), y: remToPixels(y_initial) },
		};

		const headerOffset = controllerGroupDimension.headerHeight;

		const nodes = [
			header_node,
			... devices.map((device, index) => ({
				id: `device-${device.address}-${x_initial}-${index}`,
				type: "deviceNode",
				data: { device },
				dragHandle: ".custom-drag-handle",
				position: { x: remToPixels(x_initial + controllerGroupDimension.generalMargin), y: remToPixels(y_initial + headerOffset + (deviceDimension.height  + controllerGroupDimension.generalMargin) * index)},
				parentNode: header_node.id,
			}))
		];

		const total_height = headerOffset + device_height;

		return {
			nodes,
			total_height
		}
	}

	function deriveNodes(controllers: MatchedControllers) {
		const interRowSpacing = 2;

		let currentY = 0;

		return controllers.flatMap((controller, index) => {
			const y_initial = currentY;

			const interColumnSpacing = 4 + controllerGroupDimension.width;

			const windows = deviceColumn(controller.windows, 0, y_initial);
			const linux = deviceColumn(controller.linux, interColumnSpacing, y_initial);

			currentY += Math.max(windows.total_height, linux.total_height) + interRowSpacing;

			return [
				...windows.nodes,
				...linux.nodes,
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

<div style:width="100vw" style:height="100vh">
	<!-- maxZoom={1}
	minZoom={1}
	panOnDrag={false}
	selectionOnDrag={false} -->
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
