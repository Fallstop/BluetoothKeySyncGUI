<script lang="ts">
	import type { BluetoothController, BluetoothData } from "#root/bindings";
	import {
		ChevronDown,
		ChevronRight,
		Bluetooth,
		Smartphone,
		Computer,
		Check,
	} from "lucide-svelte";
	import { slide } from "svelte/transition";
	import Device from "./Device.svelte";

	interface Props {
		controller: BluetoothController;
		expandable?: boolean;
		class?: string;
	}

	let { controller, class: userClass, expandable }: Props = $props();

	let expandedController = $state(false);

	function toggleController() {
		expandedController = !expandedController;
	}
</script>

<div class="border rounded-lg {userClass} {expandable ? 'bg-muted/50' : ''}">
	<button
		class="flex items-center gap-2 w-full text-left rounded p-5 -m-2"
		disabled={!controller.devices.length || !expandable}
		onclick={toggleController}
	>
		{#if expandable}
			{#if !controller.devices.length}
				<span class="h-4 w-4"></span>
			{:else if expandedController}
				<ChevronDown class="h-4 w-4" />
			{:else}
				<ChevronRight class="h-4 w-4" />
			{/if}
		{/if}
		<Bluetooth class="h-4 w-4" />
		<div>
			<div class="font-medium">
				{controller.name || "Bluetooth Controller"}
			</div>
			<div class="text-sm text-muted-foreground">
				{controller.address} • {controller.devices.length} device(s)
			</div>
		</div>
	</button>

	{#if expandedController || !expandable}
		<div
			class=" m-4 space-y-4"
			transition:slide={{ axis: "y", duration: 200 }}
		>
			{#each controller.devices as device}
				<Device {device} />
			{/each}
		</div>
	{/if}
</div>
