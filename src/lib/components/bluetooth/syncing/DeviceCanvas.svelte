<script lang="ts">
  import '@xyflow/svelte/dist/style.css';
  import { SvelteFlow, SvelteFlowProvider } from '@xyflow/svelte';
	import DeviceNode from './DeviceNode.svelte';
	import type { MatchedControllers } from '#root/src/routes/sync/Sync.svelte';

	let { matchedControllers }: { matchedControllers: MatchedControllers } = $props();

  const nodeTypes = { deviceNode: DeviceNode };

  // let nodes = $state.raw([
  //   {
  //     id: '1',
  //     type: 'deviceNode',
  //     position: { x: 0, y: 0 },
  //     data: { label: 'Hello' },
  //   },
  //   {
  //     id: '2',
  //     type: 'deviceNode',
  //     position: { x: 100, y: 100 },
  //     data: { device: 'World' },
  //   },
  // ]);

let nodes = $state.raw(matchedControllers.map((controller, index) => {
	return controller.windows?.devices.map((device, deviceIndex) => ({
		id: `w-${index}-${deviceIndex}`,
		type: 'deviceNode',
		position: { x: 0, y: index * 100 + deviceIndex * 50 },
		data: { device },
	}
	)) || [];
}).flat().concat(matchedControllers.map((controller, index) => {
	return controller.linux?.devices.map((device, deviceIndex) => ({
		id: `l-${index}-${deviceIndex}`,
		type: 'deviceNode',
		position: { x: 200, y: index * 100 + deviceIndex *
50 },
		data: { device },
	}
	)) || [];
}).flat()));

  let edges = $state.raw([
    { id: 'e1-2', source: '1', target: '2', type: 'smoothstep', label: 'to the' },
  ]);

</script>

<div style:width="100vw" style:height="50vh">

<SvelteFlowProvider>
  <SvelteFlow bind:nodes bind:edges panOnDrag={false} maxZoom={1} minZoom={1} selectionOnDrag={false} fitView viewport={{x:0,y:0,zoom:1}}  {nodeTypes} class="bg-background!" />
</SvelteFlowProvider>
</div>
