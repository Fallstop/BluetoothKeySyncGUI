<script lang="ts">
	import { goto } from "$app/navigation";
	import { btStore } from "@/state";
	import Sync from "./Sync.svelte";

	$effect(() => {
		if (!btStore.state.windows || !btStore.state.linux) {
			goto("/");
		}
	});
</script>

<Sync/>
