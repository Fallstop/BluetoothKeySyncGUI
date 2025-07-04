<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { rpc } from "@/api";
	import DevicesTree from "@/components/bluetooth/DevicesTree.svelte";
	import * as Dialog from "@/components/ui/dialog";
	import { Progress } from "@/components/ui/progress";
	import { btStore } from "@/state";
	import { Bluetooth, CheckCircle, ExternalLink } from "lucide-svelte";

	let dialogOpen = $state(false);
	let textState: string | null = $state(null);

	async function extractLinuxBT() {
		textState = null;
		dialogOpen = true;
		let response = await rpc.linux.parse_local_config();
		if (response.type === "Error") {
			console.error("Error processing hive file:", response.data);
			textState = `Error: ${response.data}`;
			return;
		}
		btStore.state.linux = response.data;
	}
</script>

<Dialog.Root
	open={dialogOpen}
	onOpenChange={(open) => {
		dialogOpen = open;
		if (!open) {
			textState = "";
		}
	}}
>
	<Dialog.Content class="max-w-4xl max-h-[80vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>
				{#if btStore.state.linux}
					{#if btStore.state.linux.controllers.length > 0}
						Bluetooth Devices Found
					{:else}
						No Bluetooth data found
					{/if}
				{:else}
					Analysing stored bluetooth configuration
				{/if}
			</Dialog.Title>
			<Dialog.Description>
				{#if btStore.state.linux}
					Successfully extracted {btStore.state.linux.controllers.reduce(
						(_, y) => y.devices.length,
						0,
					)} devices.
				{/if}
			</Dialog.Description>
		</Dialog.Header>

		{#if btStore.state.linux}
			<div class="py-4">
				<DevicesTree data={btStore.state.linux} />
			</div>
		{:else if textState}
			<div class="py-4">
				<pre class="text-xs overflow-x-auto bg-muted p-4 rounded">
          {textState}
        </pre>
			</div>
		{:else}
			<div class="py-4">
			Starting permission scrapper...
			</div>
			<div class="py-4">
				<Progress value={null}  />
			</div>
		{/if}

		<Dialog.Footer>
			<Button onclick={() => (dialogOpen = false)}>Done</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>


<Card.Root class="w-full relative">
	<div class="absolute -top-2 z-10 -left-2 border-1 border-accent-foreground rounded-full p-1 bg-accent font-bold px-2 w-8 h-8 flex justify-center items-center">
		2
	</div>
		{#if btStore.state.linux}
		<div
			class="absolute -top-2 -right-2 border-2 border-green-500 rounded-full p-1 bg-green-500 text-white"
		>
			<CheckCircle class="h-5 w-5" />
		</div>
	{/if}

	<Card.Header>
		<Card.Title class={ btStore.state.linux ? "line-through opacity-80" : ""}>
			Elevate Linux Permission
		</Card.Title>
		<Card.Description>
			We need to be able to read & write to the Linux bluetooth keys. The button below will open a terminal window and run the command to grant root access to this application.
		</Card.Description>
	</Card.Header>
	<Card.Footer class="flex-row gap-2">
				{#if btStore.state.linux}
			<div
				class="border rounded-lg p-3 flex-1 flex flex-row items-center gap-2"
			>
				<Bluetooth class="h-4 w-4" />
				<div>
					<div class="font-medium">
						Extracted {btStore.state.linux.controllers.reduce(
							(_, y) => y.devices.length,
							0,
						)} devices from Linux
					</div>
					<div class="text-sm text-muted-foreground">
						<!-- {controller.address} • {controller.devices.length} device(s) -->
					</div>
				</div>
				<Button onclick={()=>{btStore.state.linux = null;}} variant="outline" class="ml-auto"
					>Clear Data</Button
				>
			</div>
		{:else}
			<Button type="submit" onclick={extractLinuxBT}>Grant Root Access</Button>
		{/if}
	</Card.Footer>
</Card.Root>
