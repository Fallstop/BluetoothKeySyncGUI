<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { ArrowRightCircle } from "lucide-svelte";
	import { Button } from "../../ui/button";
</script>


<div class="flex-1"></div>
<div class="w-full p-4 sticky bottom-0 flex flex-row-reverse mt-auto h-min pointer-events-none">
	<Dialog.Root>
		<Dialog.Trigger>
				<Button variant="outline" class="pointer-events-auto"> <ArrowRightCircle/> Review Changes</Button>
			</Dialog.Trigger>
			<Dialog.Content>
				<Dialog.Header>
				<Dialog.Title>Are you sure absolutely sure?</Dialog.Title>
				<Dialog.Description>
					This action cannot be undone. This will permanently delete your account
					and remove your data from our servers.
				</Dialog.Description>
			</Dialog.Header>
		</Dialog.Content>
	</Dialog.Root>

</div>
