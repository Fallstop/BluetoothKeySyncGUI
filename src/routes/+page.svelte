<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { invoke } from "@tauri-apps/api/core";
	import ConfigWindows from "./ConfigWindows.svelte";
	import ConfigLinux from "./ConfigLinux.svelte";
	import * as Card from "@/components/ui/card";
	import { btStore } from "@/state";

	let name = $state("");
	let greetMsg = $state("");

</script>

<main class="flex flex-col justify-center p-4 max-w-3xl mx-auto gap-4">
	<h1 class="scroll-m-20 my-10 text-4xl font-extrabold tracking-tight lg:text-5xl">
		Let's get connected!
	</h1>


	<ConfigWindows />
	<ConfigLinux />
	<Card.Root class="w-full relative">
		<div
			class="absolute -top-2 z-10 -left-2 border-1 border-accent-foreground rounded-full p-1 bg-accent font-bold px-2 w-8 h-8 flex justify-center items-center"
		>
			3
		</div>
		<Card.Header>
			<Card.Title class="flex flex-row gap-2 justify-between items-center">
				<span>
					Start syncing your bluetooth devices
				</span>

				{#if btStore.state.windows && btStore.state.linux}
					<Button href="/sync">Continue</Button>
				{:else}
					<Button variant="outline" disabled>Continue</Button>
				{/if}
			</Card.Title>
		</Card.Header>
	</Card.Root>
</main>
