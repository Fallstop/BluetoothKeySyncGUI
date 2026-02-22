<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { ArrowRightCircle, Check, X } from "lucide-svelte";
	import { Button } from "../../ui/button";
	import type { Edge } from "@xyflow/svelte";
	import { NodeID } from "./nodeCanvas/NodeID";
	import { btStore } from "@/state";
	import type { BluetoothDevice } from "#root/bindings";

	let { edges = [] }: { edges?: Edge[] } = $props();

	type SyncProposal = {
		sourceDevice: BluetoothDevice;
		targetDevice: BluetoothDevice;
		sourceOS: string;
		targetOS: string;
		action: 'copy_keys';
	};

	function generateProposals(edges: Edge[]): SyncProposal[] {
		const proposals: SyncProposal[] = [];
		
		for (const edge of edges) {
			const sourceNodeId = NodeID.parse(edge.source);
			const targetNodeId = NodeID.parse(edge.target);
			
			if (!sourceNodeId || !targetNodeId) continue;
			
			// Find the actual devices from the store
			const sourceController = btStore.state[sourceNodeId.os.toLowerCase() as 'windows' | 'linux']?.controllers
				.find(c => c.address === sourceNodeId.controller);
			const targetController = btStore.state[targetNodeId.os.toLowerCase() as 'windows' | 'linux']?.controllers
				.find(c => c.address === targetNodeId.controller);
				
			const sourceDevice = sourceController?.devices.find(d => d.address === sourceNodeId.device);
			const targetDevice = targetController?.devices.find(d => d.address === targetNodeId.device);
			
			if (sourceDevice && targetDevice) {
				proposals.push({
					sourceDevice,
					targetDevice,
					sourceOS: sourceNodeId.os,
					targetOS: targetNodeId.os,
					action: 'copy_keys'
				});
			}
		}
		
		return proposals;
	}

	let proposals = $derived(generateProposals(edges));
	let isApplying = $state(false);

	async function applyChanges() {
		isApplying = true;
		try {
			// Here you would send the proposals to your backend
			console.log('Applying changes:', proposals);
			
			// Simulate API call
			await new Promise(resolve => setTimeout(resolve, 1000));
			
			// Clear edges after successful application
			edges.length = 0;
		} catch (error) {
			console.error('Failed to apply changes:', error);
		} finally {
			isApplying = false;
		}
	}
</script>


<div class="flex-1"></div>
<div class="w-full p-4 sticky bottom-0 flex flex-row-reverse mt-auto h-min pointer-events-none">
	<Dialog.Root>
		<Dialog.Trigger>
			<Button 
				variant="outline" 
				class="pointer-events-auto" 
				disabled={proposals.length === 0}
			> 
				<ArrowRightCircle/> 
				Review Changes {proposals.length > 0 ? `(${proposals.length})` : ''}
			</Button>
		</Dialog.Trigger>
		<Dialog.Content class="max-w-2xl">
			<Dialog.Header>
				<Dialog.Title>Bluetooth Key Sync Proposals</Dialog.Title>
				<Dialog.Description>
					Review the proposed Bluetooth key synchronizations between your devices.
				</Dialog.Description>
			</Dialog.Header>
			
			<div class="space-y-4 max-h-96 overflow-y-auto">
				{#each proposals as proposal, index}
					<div class="border rounded-lg p-4 space-y-2">
						<div class="flex items-center justify-between">
							<h4 class="font-medium">Sync #{index + 1}</h4>
							<div class="flex items-center gap-2 text-sm text-muted-foreground">
								<span class="px-2 py-1 bg-blue-100 text-blue-800 rounded">
									{proposal.sourceOS}
								</span>
								<ArrowRightCircle class="w-4 h-4" />
								<span class="px-2 py-1 bg-green-100 text-green-800 rounded">
									{proposal.targetOS}
								</span>
							</div>
						</div>
						
						<div class="text-sm">
							<div class="grid grid-cols-2 gap-4">
								<div>
									<p class="font-medium">Source Device</p>
									<p class="text-muted-foreground">
										{proposal.sourceDevice.name || 'Unknown Device'}
									</p>
									<p class="text-xs text-muted-foreground">
										{proposal.sourceDevice.address}
									</p>
								</div>
								<div>
									<p class="font-medium">Target Device</p>
									<p class="text-muted-foreground">
										{proposal.targetDevice.name || 'Unknown Device'}
									</p>
									<p class="text-xs text-muted-foreground">
										{proposal.targetDevice.address}
									</p>
								</div>
							</div>
						</div>
						
						<div class="text-xs text-muted-foreground">
							Action: Copy Bluetooth pairing keys from {proposal.sourceOS} to {proposal.targetOS}
						</div>
					</div>
				{/each}
				
				{#if proposals.length === 0}
					<div class="text-center py-8 text-muted-foreground">
						<p>No sync connections found.</p>
						<p class="text-sm">Draw connections between devices to create sync proposals.</p>
					</div>
				{/if}
			</div>
			
			{#if proposals.length > 0}
				<div class="flex justify-end gap-2 pt-4 border-t">
					<Dialog.Close>
						<Button variant="outline">
							<X class="w-4 h-4 mr-2" />
							Cancel
						</Button>
					</Dialog.Close>
					<Button 
						onclick={applyChanges} 
						disabled={isApplying}
						class="bg-green-600 hover:bg-green-700"
					>
						{#if isApplying}
							<div class="w-4 h-4 mr-2 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
							Applying...
						{:else}
							<Check class="w-4 h-4 mr-2" />
							Apply Changes
						{/if}
					</Button>
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Root>
</div>
