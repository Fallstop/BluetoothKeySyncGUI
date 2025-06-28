<script module lang="ts">
	import type { BluetoothDevice } from '#root/bindings';
	import type { Node } from '@xyflow/svelte';

  export type DeviceNodeType = Node<{ device: BluetoothDevice }, 'device'>;
</script>

<script lang="ts">
  import { Handle, Position, useConnection, useSvelteFlow, type NodeProps } from '@xyflow/svelte';
	import Device from '../Device.svelte';

  let { id, data }: NodeProps<DeviceNodeType> = $props();

	const connection = useConnection();

  let isTarget = $derived(
    connection.current.inProgress &&
      connection.current.fromHandle?.nodeId !== id,
  );

  let { updateNodeData } = useSvelteFlow();
</script>

<div class="deviceNone">
		{#if !connection.current.inProgress}
      <Handle
        class="customHandle"
        position={Position.Right}
        type="source"
        style="z-index: 1;"
      />
    {/if}
    <Handle
      class="customHandle"
      position={Position.Left}
      type="target"
      isConnectableStart={false}
    />
	<div class="hidden custom-drag-handle pointer-events-none"></div>
	<Device device={data.device} />
</div>

<style lang="scss">
	.deviceNone {
		position: relative;

		:global(.customHandle) {
			position: absolute;
			width: 100%;
			height: 100%;
			top: 0;
			left: 0;
			z-index: 0;
			border-radius: 0;
			border: none;
			transform: none;
			opacity: 0;
		}

	}
</style>
