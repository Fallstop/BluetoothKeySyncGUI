<script lang="ts">
	import type { BluetoothDevice, HostDistributions } from '#root/bindings';
	import type { KeyFieldComparison, MatchedDevicePair, ManualMatch, SyncDirection } from './matching';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Check, X, AlertTriangle, Minus } from 'lucide-svelte';
	import { osColor } from './os-theme';

	type PairData = {
		mode: 'pair';
		pair: MatchedDevicePair | ManualMatch;
		direction?: SyncDirection | null;
	};

	type SingleData = {
		mode: 'single';
		device: BluetoothDevice;
		os: HostDistributions;
		controllerAddress?: string;
	};

	let {
		open = $bindable(false),
		data
	}: {
		open?: boolean;
		data: PairData | SingleData;
	} = $props();

	function statusIcon(status: KeyFieldComparison['status']) {
		switch (status) {
			case 'match':
				return { icon: Check, color: '#22c55e' };
			case 'mismatch':
				return { icon: X, color: '#ef4444' };
			case 'source_only':
			case 'target_only':
				return { icon: AlertTriangle, color: '#f59e0b' };
			case 'os_not_available':
				return { icon: Minus, color: 'rgba(250,250,250,0.2)' };
			case 'both_missing':
				return { icon: Minus, color: 'rgba(250,250,250,0.3)' };
		}
	}

	function displayValue(value: string | null, status?: KeyFieldComparison['status']): string {
		if (!value) {
			return status === 'os_not_available' ? 'N/A' : '--';
		}
		return value;
	}

	function formatValue(val: string | number | boolean | null | undefined): string {
		if (val == null) return '--';
		if (typeof val === 'boolean') return val ? 'true' : 'false';
		return String(val);
	}

	// Build key fields for a single device
	function singleDeviceFields(device: BluetoothDevice): Array<{ label: string; value: string; group: string }> {
		const fields: Array<{ label: string; value: string; group: string }> = [];

		if (device.link_key) {
			fields.push({ label: 'Link Key', value: device.link_key.key, group: 'Classic' });
			fields.push({ label: 'Key Type', value: formatValue(device.link_key.key_type), group: 'Classic' });
			fields.push({ label: 'PIN Length', value: formatValue(device.link_key.pin_length), group: 'Classic' });
		}

		if (device.le_data) {
			const le = device.le_data;
			if (le.long_term_key) {
				fields.push({ label: 'Long Term Key', value: le.long_term_key.key, group: 'LE' });
				fields.push({ label: 'LTK Authenticated', value: formatValue(le.long_term_key.authenticated), group: 'LE' });
				fields.push({ label: 'EDIV', value: formatValue(le.long_term_key.ediv), group: 'LE' });
				fields.push({ label: 'Rand', value: formatValue(le.long_term_key.rand), group: 'LE' });
				fields.push({ label: 'Key Length', value: formatValue(le.long_term_key.key_length), group: 'LE' });
			}
			if (le.peripheral_long_term_key) {
				fields.push({ label: 'Peripheral LTK', value: le.peripheral_long_term_key.key, group: 'LE Peripheral' });
				fields.push({ label: 'Peripheral Authenticated', value: formatValue(le.peripheral_long_term_key.authenticated), group: 'LE Peripheral' });
				fields.push({ label: 'Peripheral EDIV', value: formatValue(le.peripheral_long_term_key.ediv), group: 'LE Peripheral' });
				fields.push({ label: 'Peripheral Rand', value: formatValue(le.peripheral_long_term_key.rand), group: 'LE Peripheral' });
				fields.push({ label: 'Peripheral Key Length', value: formatValue(le.peripheral_long_term_key.key_length), group: 'LE Peripheral' });
			}
			if (le.identity_resolving_key) {
				fields.push({ label: 'Identity Resolving Key', value: le.identity_resolving_key, group: 'LE' });
			}
			if (le.local_signature_key) {
				fields.push({ label: 'Local CSRK', value: le.local_signature_key.key, group: 'LE Signature' });
				fields.push({ label: 'Local Counter', value: formatValue(le.local_signature_key.counter), group: 'LE Signature' });
			}
			if (le.remote_signature_key) {
				fields.push({ label: 'Remote CSRK', value: le.remote_signature_key.key, group: 'LE Signature' });
				fields.push({ label: 'Remote Counter', value: formatValue(le.remote_signature_key.counter), group: 'LE Signature' });
			}
			if (le.address_type) {
				fields.push({ label: 'Address Type', value: le.address_type, group: 'LE' });
			}
		}

		return fields;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="glass-dialog glass-dialog-wide">
		<Dialog.Header>
			{#if data.mode === 'pair'}
				{@const pair = data.pair}
				<Dialog.Title class="glass-dialog-title">
					{pair.windowsDevice.name ?? pair.linuxDevice.name ?? 'Unknown Device'}
				</Dialog.Title>
				<Dialog.Description class="glass-dialog-desc">
					Key comparison between Windows and Linux
				</Dialog.Description>
			{:else}
				<Dialog.Title class="glass-dialog-title">
					{data.device.name ?? 'Unknown Device'}
				</Dialog.Title>
				<Dialog.Description class="glass-dialog-desc">
					{data.os} device details
				</Dialog.Description>
			{/if}
		</Dialog.Header>

		{#if data.mode === 'pair'}
			<!-- Pair: device addresses -->
			{@const pair = data.pair}
			<div class="dialog-device-info">
				<div class="device-addr-row">
					<span class="os-pill" style="background: {osColor('Windows').pillBg}; border-color: {osColor('Windows').pillBorder}; color: {osColor('Windows').pillText}">Win</span>
					<span class="addr-mono">{pair.windowsDevice.address}</span>
					{#if pair.windowsDevice.device_type}
						<span class="type-badge">{pair.windowsDevice.device_type}</span>
					{/if}
				</div>
				<div class="device-addr-row">
					<span class="os-pill" style="background: {osColor('Linux').pillBg}; border-color: {osColor('Linux').pillBorder}; color: {osColor('Linux').pillText}">Lin</span>
					<span class="addr-mono">{pair.linuxDevice.address}</span>
					{#if pair.linuxDevice.device_type}
						<span class="type-badge">{pair.linuxDevice.device_type}</span>
					{/if}
				</div>
			</div>

			<!-- Key comparison table -->
			<div class="key-grid">
				{#each [...pair.comparison.linkKey, ...pair.comparison.leData].filter((f) => f.status !== 'both_missing') as field, i}
					{@const si = statusIcon(field.status)}
					<div class="key-row pair-row" class:key-row-na={field.status === 'os_not_available'}>
						<div class="key-row-label">
							<span class="key-field-label">{field.label}</span>
						</div>
						<div class="pair-value-cell">
							<span class="os-dot" style="background: {osColor('Windows').hex}"></span>
							<span class="key-value-full">{displayValue(field.windowsValue, field.status)}</span>
						</div>
						<div class="pair-status-icon">
							<si.icon class="h-3.5 w-3.5" style="color: {si.color}" />
						</div>
						<div class="pair-value-cell">
							<span class="os-dot" style="background: {osColor('Linux').hex}"></span>
							<span class="key-value-full">{displayValue(field.linuxValue, field.status)}</span>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<!-- Single device -->
			{@const device = data.device}
			{@const colors = osColor(data.os)}
			<div class="dialog-device-info">
				<div class="device-addr-row">
					<span class="os-pill" style="background: {colors.pillBg}; border-color: {colors.pillBorder}; color: {colors.pillText}">{data.os}</span>
					<span class="addr-mono">{device.address}</span>
					{#if device.device_type}
						<span class="type-badge">{device.device_type}</span>
					{/if}
				</div>
				{#if data.controllerAddress}
					<div class="controller-addr">
						Controller: <span class="addr-mono">{data.controllerAddress}</span>
					</div>
				{/if}
			</div>

			{@const fields = singleDeviceFields(device)}
			{#if fields.length === 0}
				<div class="no-keys">No keys stored for this device.</div>
			{:else}
				{@const groups = [...new Set(fields.map(f => f.group))]}
				<div class="key-grid">
					{#each groups as group}
						<div class="group-label">{group}</div>
						{#each fields.filter(f => f.group === group) as field, i}
							<div class="key-row">
								<div class="key-row-label">
									<span class="key-field-label">{field.label}</span>
								</div>
								<div class="key-row-values">
									<div class="key-value-line">
										<span class="key-value-full">{displayValue(field.value)}</span>
									</div>
								</div>
							</div>
						{/each}
					{/each}
				</div>
			{/if}
		{/if}

		<Dialog.Footer class="glass-dialog-footer">
			<Dialog.Close class="gf-btn ghost">Close</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<style lang="css">
	.dialog-device-info {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 12px;
	}

	.device-addr-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.os-pill {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 6px;
		border: 1px solid;
		font-size: 11px;
		font-weight: 600;
		flex-shrink: 0;
	}

	.addr-mono {
		font-size: 12px;
		font-family: ui-monospace, monospace;
		color: rgba(250, 250, 250, 0.55);
	}

	.type-badge {
		font-size: 10px;
		font-weight: 500;
		padding: 1px 6px;
		border-radius: 4px;
		background: rgba(255, 255, 255, 0.06);
		color: rgba(250, 250, 250, 0.35);
	}

	.controller-addr {
		font-size: 11px;
		color: rgba(250, 250, 250, 0.3);
		margin-top: 2px;
	}

	.key-grid {
		display: flex;
		flex-direction: column;
		gap: 3px;
		max-height: 55vh;
		overflow-y: auto;
		padding-right: 4px;
	}

	.group-label {
		font-size: 11px;
		font-weight: 600;
		color: rgba(250, 250, 250, 0.35);
		letter-spacing: 0.03em;
		padding: 8px 0 2px;
	}

	.group-label:first-child {
		padding-top: 0;
	}

	.key-row {
		display: flex;
		gap: 12px;
		padding: 8px 10px;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.04);
	}

	.key-row.key-row-na {
		opacity: 0.45;
	}

	.key-row-label {
		flex: 0 0 clamp(100px, 25%, 180px);
		display: flex;
		align-items: flex-start;
		gap: 6px;
		padding-top: 1px;
	}

	.key-field-label {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.45);
		font-weight: 500;
	}

	/* Side-by-side pair layout */
	.pair-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.pair-value-cell {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.pair-status-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
	}

	/* Single device stacked layout */
	.key-row-values {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.key-value-line {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.os-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.key-value-full {
		flex: 1;
		min-width: 0;
		font-size: 11px;
		font-family: ui-monospace, monospace;
		color: rgba(250, 250, 250, 0.65);
		word-break: break-all;
		user-select: text;
		-webkit-user-select: text;
	}

	.no-keys {
		text-align: center;
		padding: 24px 0;
		font-size: 13px;
		color: rgba(250, 250, 250, 0.3);
	}
</style>
