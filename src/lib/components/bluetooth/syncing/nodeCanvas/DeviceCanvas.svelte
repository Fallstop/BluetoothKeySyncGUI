<script lang="ts" module>
	export type DeviceCanvasProps = {
		matchedControllers: MatchedControllers
	};
</script>

<script lang="ts">
	import "@xyflow/svelte/dist/style.css";
	import { ConnectionLineType, MarkerType, SvelteFlow, SvelteFlowProvider, type Edge, type IsValidConnection, type Node } from "@xyflow/svelte";
	import type { MatchedControllers } from "#root/src/routes/sync/Sync.svelte";
	import type { BluetoothController, BluetoothDevice, HostDistributions } from "#root/bindings";

	import DeviceNode, { deviceDimension } from "./DeviceNode.svelte";
	import FloatingEdge from "./FloatingEdge.svelte";
	import ControllerGroupNode, { controllerGroupDimension } from "./ControllerGroupNode.svelte";
	import { remToPixels } from "./utils";
	import { NodeID } from "./NodeID"

	let { matchedControllers }: DeviceCanvasProps =	$props();

	const nodeTypes = { deviceNode: DeviceNode, controllerNode: ControllerGroupNode };

	function deviceColumn(controller: BluetoothController | null, host: HostDistributions, x_initial: number, y_initial: number) {
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
				id: new NodeID(host, controller.address, device.address).toString(),
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

	let graphDimensions = $state({height: 0, width: 0});

	function deriveNodes(controllers: MatchedControllers) {
		const interRowSpacing = 2;

		let currentY = 0;
		const interColumnSpacing = 4 + controllerGroupDimension.width;

		const final_nodes = controllers.flatMap((controller, index) => {
			const y_initial = currentY;


			const windows = deviceColumn(controller.windows, "Windows", 0, y_initial);
			const linux = deviceColumn(controller.linux, "Linux", interColumnSpacing, y_initial);

			currentY += Math.max(windows.total_height, linux.total_height) + interRowSpacing;

			return [
				...windows.nodes,
				...linux.nodes,
			];

		});

		graphDimensions.height = currentY;
		graphDimensions.width = interColumnSpacing + controllerGroupDimension.width;

		return final_nodes;
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


	let nodes = $state.raw<Node[]>(deriveNodes(matchedControllers));

	let edges = $state.raw<Edge[]>([]);


	$effect(()=>{
		const knownNodes = new Set();

		const proposed_edges =  edges.reverse().filter((edge)=>{
			if (!edge.source || !edge.target) {
				return true;
			}

			if (knownNodes.has(edge.source) || knownNodes.has(edge.target)) {
				// Already used in the recent edge
				return false;
			}

			knownNodes.add(edge.source);
			knownNodes.add(edge.target);

			return true;
		}).reverse();

		if (proposed_edges.length != edges.length) {
			// We can't directly resign, we only do if we have found a node to remove, otherwise we cause an infinite loop
			edges = proposed_edges;
		}
	})

	const isValidConnection: IsValidConnection = (connection) => {
		let source = NodeID.parse(connection.source);
		let target = NodeID.parse(connection.target);

		if (!source || !target) {
			// Needs to be between devices, which will parse.
			return false;
		}

		if (source.os == target.os) {
			// Can't copy keys intra-os
			return false;
		}

		return true;
	}

</script>

<div style:width="{graphDimensions.width}rem" style:height="{graphDimensions.height}rem">
	<SvelteFlowProvider>

		<SvelteFlow
		bind:nodes
		bind:edges
		minZoom={1}
		maxZoom={1}
		panOnDrag={false}
		zoomOnDoubleClick={false}
		zoomOnScroll={false}
		selectionOnDrag={false}
		preventScrolling={false}

			viewport={{ x: 0, y: 0, zoom: 1 }}
			{edgeTypes}
			{defaultEdgeOptions}
			connectionLineType={ConnectionLineType.Straight}
			{nodeTypes}
			{isValidConnection}

			proOptions={{hideAttribution:true}}
			class="bg-background!"
		/>
	</SvelteFlowProvider>
</div>
