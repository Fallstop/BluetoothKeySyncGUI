<script lang="ts">
	import { SvelteFlowProvider } from "@xyflow/svelte";
	import DeviceCanvas, { type DeviceCanvasProps } from "./DeviceCanvas.svelte";
	import type { MatchedControllers } from "#root/src/routes/sync/Sync.svelte";

	let props: DeviceCanvasProps = $props();
</script>

<div style:width="100vw" style:height="100vh">
	<SvelteFlowProvider>
		<DeviceCanvas {...props}/>
	</SvelteFlowProvider>
</div>
