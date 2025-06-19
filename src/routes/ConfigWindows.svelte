<script lang="ts">
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import CodeBlock from "@/components/CodeBlock.svelte";
	import BluetoothDevicesTree from "@/components/BluetoothDevicesTree.svelte";
	import { ExternalLink, CheckCircle, Bluetooth } from "lucide-svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import * as Dialog from "@/components/ui/dialog";
	import { rpc } from "@/api";
	import { btStore, windowsState } from "@/state";
	import { Progress } from "@/components/ui/progress";

	let dialogOpen = $state(false);
	let textState: string | null = $state(null);

	async function readHiveFile(path: string | null) {
		textState = null
		console.log(path);
		if (dialogOpen) {
			return;
		}

		if (!path) {
			textState = "No file selected.";
			return;
		}

		dialogOpen = true;
		let response = await rpc.windows.parse_windows_hive(path);

		if (response.type === "Error") {
			console.error("Error processing hive file:", response.data);
			textState = `Error: ${response.data}`;
			return;
		}

		windowsState.state.lastWindowsHiveFile = response.data.source_path;
		btStore.state.windows = response.data;
	}

	async function selectWindowsDir() {
		// Open a dialog
		const windowsPath = await open({
			multiple: false,
			directory: true,
			title: "Select the Windows directory",
			defaultPath: windowsState.state.lastWindowsDirectory || undefined,
		});

		if (!windowsPath) {
			textState = "No directory selected.";
			return;
		}

		windowsState.state.lastWindowsDirectory = windowsPath;

		readHiveFile(windowsPath);
	}

	async function selectWindowsHiveFile() {
		// Open a dialog
		const file = await open({
			multiple: false,
			directory: false,
			title: "Select the Windows hive file",
			defaultPath: windowsState.state.lastWindowsHiveFile || undefined,
		});

		if (!file) {
			textState = "No file selected.";
			return;
		}

		windowsState.state.lastWindowsHiveFile = file;

		readHiveFile(file);
	}

	function clearWindowsData() {
		btStore.state.windows = null;
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
				{#if btStore.state.windows}
					Successfully extracted {btStore.state.windows.controllers.reduce(
						(_, y) => y.devices.length,
						0,
					)} devices.
				{/if}
			</Dialog.Description>
		</Dialog.Header>

		{#if btStore.state.windows}
			<div class="py-4">
				<BluetoothDevicesTree data={btStore.state.windows} />
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
	<div
		class="absolute -top-2 z-100 -left-2 border-2 border-accent rounded-full p-1 bg-foreground text-background font-bold px-2"
	>
		1
	</div>
	{#if btStore.state.windows}
		<div
			class="absolute -top-2 -right-2 border-2 border-green-500 rounded-full p-1 bg-green-500 text-white"
		>
			<CheckCircle class="h-4 w-4" />
		</div>
	{/if}
	<Card.Header>
		<Card.Title class="flex items-center gap-2">
			Select your Windows drive
		</Card.Title>
		<Card.Description>
			We need to scan the Windows OS drive to find the Bluetooth keys.
			<br />
			Please select the base directory of your mounted Windows installation, alternatively,
			you can directly select the <CodeBlock>SYSTEM</CodeBlock> hive file usually
			found at <CodeBlock>/Windows/System32/config/SYSTEM</CodeBlock>.
			<Button
				href="https://phoenixnap.com/kb/mount-ntfs-linux"
				target="_blank"
				referrerpolicy="no-referrer"
				variant="link"
				class="!pl-0"
			>
				<ExternalLink />
				How do I mount a NTFS filesystem?
			</Button>
		</Card.Description>
	</Card.Header>
	<Card.Footer
		class="flex-row gap-2 {btStore.state.windows ? 'justify-between' : ''}"
	>
		{#if btStore.state.windows}
			<div
				class="border rounded-lg p-3 flex-1 flex flex-row items-center gap-2"
			>
				<Bluetooth class="h-4 w-4" />
				<div>
					<div class="font-medium">
						Extracted {btStore.state.windows.controllers.reduce(
							(_, y) => y.devices.length,
							0,
						)} devices from Windows
					</div>
					<div class="text-sm text-muted-foreground">
						<!-- {controller.address} • {controller.devices.length} device(s) -->
					</div>
				</div>
				<Button onclick={clearWindowsData} variant="outline" class="ml-auto"
					>Clear Data</Button
				>
			</div>
		{:else}
			<Button onclick={selectWindowsDir}>Select the Windows directory</Button>
			<Button onclick={selectWindowsHiveFile} variant="outline"
				>Select the Hive File</Button
			>
		{/if}
	</Card.Footer>
</Card.Root>
