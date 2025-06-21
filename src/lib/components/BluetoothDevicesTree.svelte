<script lang="ts">
	import type { BluetoothData } from '#root/bindings';
	import * as Card from '$lib/components/ui/card/index.js';
	import { ChevronDown, ChevronRight, Bluetooth, Smartphone, Computer } from 'lucide-svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { fade, fly, slide } from 'svelte/transition';

	interface Props {
		data: BluetoothData;
	}

	let { data }: Props = $props();

	let expandedControllers = $state(new SvelteSet<string>());

	function toggleController(address: string) {
		console.log('Toggling controller:', expandedControllers);
		if (expandedControllers.has(address)) {
			expandedControllers.delete(address);
		} else {
			expandedControllers.add(address);
		}
	}

	function getDeviceIcon(deviceType: string) {
		return deviceType === 'Classic' ? Computer : Smartphone;
	}
</script>


		<div class="space-y-2">
			{#each data.controllers.toSorted((a,b)=>b.devices.length-a.devices.length) as controller}
				<div class="border rounded-lg p-3">
					<button
						class="flex items-center gap-2 w-full text-left hover:bg-muted/50 rounded p-2 -m-2"
						disabled={!controller.devices.length}
						onclick={() => toggleController(controller.address)}
					>
						{#if !controller.devices.length}
							<span class="h-4 w-4"></span>
						{:else if expandedControllers.has(controller.address)}
							<ChevronDown class="h-4 w-4" />
						{:else}
							<ChevronRight class="h-4 w-4" />
						{/if}
						<Bluetooth class="h-4 w-4" />
						<div>
							<div class="font-medium">
								{controller.name || 'Bluetooth Controller'}
							</div>
							<div class="text-sm text-muted-foreground">
								{controller.address} • {controller.devices.length} device(s)
							</div>
						</div>
					</button>

					{#if expandedControllers.has(controller.address)}
						<div class="ml-6 mt-2 space-y-2" transition:slide={{ axis: "y", duration: 200 }}>
							{#each controller.devices as device}
							{@const DeviceIcon = getDeviceIcon(device.device_type)}
								<div class="flex items-center gap-2 p-2 border rounded bg-muted/30" >
									<DeviceIcon class="h-4 w-4" />
									<div class="flex-1">
										<div class="font-medium text-sm">
											{device.name || 'Unknown Device'}
										</div>
										<div class="text-xs text-muted-foreground">
											{device.address} • {device.device_type}
										</div>
										{#if device.link_key}
											<div class="text-xs text-green-600 font-medium">
												✓ Link Key Available
											</div>
										{/if}
										{#if device.le_data}
											<div class="text-xs text-blue-600 font-medium">
												✓ LE Data Available
											</div>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
