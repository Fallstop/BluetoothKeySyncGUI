<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { rpc } from "@/api";
	import BluetoothDevicesTree from "@/components/BluetoothDevicesTree.svelte";
	import CodeBlock from "@/components/CodeBlock.svelte";
	import * as Dialog from "@/components/ui/dialog";
	import { Progress } from "@/components/ui/progress";
	import { btStore } from "@/state";
	import { ExternalLink } from "lucide-svelte";

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
			<Dialog.Title>Bluetooth Devices Found</Dialog.Title>
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
				<BluetoothDevicesTree data={btStore.state.linux} />
			</div>
		{:else if textState}
			<div class="py-4">
				<pre class="text-xs overflow-x-auto bg-muted p-4 rounded">
          {textState}
        </pre>
			</div>
		{:else}
			<div class="py-4">
			Processing hive file...
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
	<div class="absolute -top-2 z-100 -left-2 border-2 border-accent rounded-full  p-1 bg-foreground text-background font-bold px-2">
		2
	</div>
	<Card.Header>
		<Card.Title>Elevate Linux Permission</Card.Title>
		<Card.Description>
			We need to be able to read & write to the Linux bluetooth keys. The button below will open a terminal window and run the command to grant root access to this application.
		</Card.Description>
	</Card.Header>
	<Card.Footer class="flex-row gap-2">
		<Button type="submit" onclick={extractLinuxBT}>Grant Root Access</Button>
	</Card.Footer>
</Card.Root>
