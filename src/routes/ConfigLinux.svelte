<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import CodeBlock from "@/components/CodeBlock.svelte";
	import { ExternalLink } from "lucide-svelte";
</script>

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
		<Button type="submit">Grant Root Access</Button>
	</Card.Footer>
</Card.Root>
