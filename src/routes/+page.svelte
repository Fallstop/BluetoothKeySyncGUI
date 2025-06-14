<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import ConfigWindows from "./ConfigWindows.svelte";
	import ConfigLinux from "./ConfigLinux.svelte";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="flex flex-col justify-center p-4 max-w-3xl mx-auto gap-4">
  <h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Let's get connected!</h1>

  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <ConfigWindows/>
	<ConfigLinux/>
</main>


