<script module>
	import type { BluetoothDevice } from '#root/bindings';
	import type { Node } from '@xyflow/svelte';

  export type DeviceNodeType = Node<{ device: BluetoothDevice }, 'device'>;
</script>

<script lang="ts">
  import { Position, useSvelteFlow, type NodeProps } from '@xyflow/svelte';
	import Device from '../Device.svelte';

  let { id, data }: NodeProps<DeviceNodeType> = $props();

  let { updateNodeData } = useSvelteFlow();
</script>

<div class="device-node border-white border-4 min-w-[200px] min-h-[100px] rounded-lg p-2 bg-white ">
	Test
	<Device device={data.device} />
</div>
