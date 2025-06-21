<script lang="ts">
	import BluetoothDevicesTree from "@/components/BluetoothDevicesTree.svelte";
	import { btStore } from "@/state";

</script>
<div class="grid grid-cols-2">
	<div class="p-4">
		<h2 class="text-xl font-bold mb-4">Windows Bluetooth Devices</h2>
		{#if btStore.state.windows}
			<BluetoothDevicesTree data={btStore.state.windows} />
		{:else}
			<p>No Windows Bluetooth data found.</p>
		{/if}
	</div>

	<div class="p-4">
		<h2 class="text-xl font-bold mb-4">Linux Bluetooth Devices</h2>
		{#if btStore.state.linux}
			<BluetoothDevicesTree data={btStore.state.linux} />
		{:else}
			<p>No Linux Bluetooth data found.</p>
		{/if}
	</div>

</div>
