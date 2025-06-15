<script lang="ts">
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import CodeBlock from "@/components/CodeBlock.svelte";
	import { ExternalLink } from "lucide-svelte";
	import { open } from '@tauri-apps/plugin-dialog';
	import * as Dialog from "@/components/ui/dialog";
	import { rpc } from "@/api";
	import { windowsState } from "@/state";

	let dialogOpen = $state(false);

	let textState = $state();

	async function readHiveFile(path: string | null) {
		console.log(path);
		if (dialogOpen) {
			return;
		}

		if (!path) {
			textState = "No file selected.";
			return;
		}
		textState = "Processing hive file...";

		dialogOpen = true;
		let response = await rpc.parse_windows_hive(path);

		if (response.type === "Error") {
			console.error("Error processing hive file:", response.data);
			textState = `Error: ${response.data}`;
			return;
		}

		textState = JSON.stringify(response, null,'\t');
		windowsState.state.lastWindowsHiveFile = response.data.source_path;
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
</script>

<Dialog.Root open={dialogOpen} onOpenChange={(open) => {
	dialogOpen = open;
	if (!open) {
		textState = "";
	}
}}>
  <Dialog.Content class="w-lg">
    <Dialog.Header>
      <Dialog.Title>Processed Hive file</Dialog.Title>
      <Dialog.Description>
				<code>
					{textState}

				</code>
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button type="submit">Continue</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Card.Root class="w-full relative">
	<div class="absolute -top-2 z-100 -left-2 border-2 border-accent rounded-full  p-1 bg-foreground text-background font-bold px-2">
		1
	</div>
	<Card.Header>
		<Card.Title>Select your Windows drive</Card.Title>
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
	<Card.Footer class="flex-row gap-2">
		<Button onclick={selectWindowsDir}>Select the Windows directory</Button>
		<Button onclick={selectWindowsHiveFile} variant="outline">Select the Hive File</Button>
	</Card.Footer>
</Card.Root>
