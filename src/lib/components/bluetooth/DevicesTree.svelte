<script lang="ts">
	import type { BluetoothData } from "#root/bindings";
	import Controller from "./Controller.svelte";

	interface Props {
		data: BluetoothData;
	}

	let { data }: Props = $props();
</script>

<div class="space-y-2">
	{#each data.controllers.toSorted((a, b) => b.devices.length - a.devices.length) as controller}
		<Controller {controller} expandable />
	{/each}
</div>
