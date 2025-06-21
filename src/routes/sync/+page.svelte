<script lang="ts">
	import { goto } from "$app/navigation";
	import { btStore } from "@/state";
	import { onMount } from "svelte";
	import Sync from "./Sync.svelte";

	onMount(()=>{
		if (!btStore.state.windows || !btStore.state.linux) {
			goto("/");
		}
	})

</script>

<div>
	<Sync/>
</div>
