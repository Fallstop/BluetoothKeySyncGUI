<script lang="ts">
	import type { BluetoothData, BluetoothDevice, HostDistributions } from '#root/bindings';
	import type { AuthMethod } from '$lib/state';
	import DeviceDetailsDialog from './bluetooth/syncing/DeviceDetailsDialog.svelte';
	import { Github } from 'lucide-svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	let detailsOpen = $state(false);
	let detailsData: { device: BluetoothDevice; os: HostDistributions; controllerAddress: string } | null = $state(null);

	function showDeviceDetails(device: BluetoothDevice, os: HostDistributions, controllerAddress: string) {
		detailsData = { device, os, controllerAddress };
		detailsOpen = true;
	}

	let {
		windowsData,
		linuxData,
		lastWindowsPath,
		onSelectWindowsDir,
		onSelectWindowsHive,
		onGrantLinuxAccess,
		onCancelLinuxAccess,
		onContinueToSync,
		onClearWindows,
		onClearLinux,
		windowsLoading,
		linuxLoading,
		windowsError,
		linuxError,
		authMethod = $bindable('pkexec'),
	}: {
		windowsData: BluetoothData | null;
		linuxData: BluetoothData | null;
		lastWindowsPath: string | null;
		onSelectWindowsDir: () => void;
		onSelectWindowsHive: () => void;
		onGrantLinuxAccess: () => void;
		onCancelLinuxAccess: () => void;
		onContinueToSync: () => void;
		onClearWindows: () => void;
		onClearLinux: () => void;
		windowsLoading: boolean;
		linuxLoading: boolean;
		windowsError: string | null;
		linuxError: string | null;
		authMethod: AuthMethod;
	} = $props();

	function deviceCount(data: BluetoothData): number {
		return data.controllers.reduce((sum, c) => sum + c.devices.length, 0);
	}

	let canSync = $derived(windowsData !== null && linuxData !== null);

	type Panel = 'windows' | 'linux' | 'sync';

	// The "natural" step based on data state
	let suggestedPanel: Panel = $derived(
		windowsData === null ? 'windows' : linuxData === null ? 'linux' : 'sync'
	);

	// Manual override - lets users click back to review completed steps
	let manualPanel: Panel | null = $state(null);

	// Reset manual override when the suggested panel advances
	$effect(() => {
		// Reading suggestedPanel here creates a dependency
		suggestedPanel;
		manualPanel = null;
	});

	let activePanel: Panel = $derived(manualPanel ?? suggestedPanel);

	function selectStep(panel: Panel) {
		// Allow clicking the current suggested step or any completed step
		if (panel === suggestedPanel) {
			manualPanel = null;
		} else if (panel === 'windows') {
			manualPanel = 'windows';
		} else if (panel === 'linux' && windowsData !== null) {
			manualPanel = 'linux';
		} else if (panel === 'sync' && canSync) {
			manualPanel = 'sync';
		}
	}

	function isStepClickable(panel: Panel): boolean {
		if (panel === suggestedPanel) return true;
		if (panel === 'windows') return windowsData !== null;
		if (panel === 'linux') return windowsData !== null;
		if (panel === 'sync') return canSync;
		return false;
	}
</script>

<div class="gf-root">
	<div class="gradient-mesh"></div>
	<button class="github-link" onclick={() => openUrl('https://github.com/Fallstop/BluetoothKeySyncGUI')} title="View on GitHub">
		<Github size={16} />
	</button>

	<div class="gf-content max-w-[860px] pt-12">
		<!-- Header -->
		<div class="gf-header">
			<div class="logo-mark">
				<img src="/logo.svg" alt="BT Key Sync" width="36" height="36" />
			</div>
			<h1 class="gf-title">BT Key Sync</h1>
			<p class="gf-tagline">Sync Bluetooth pairing keys between Windows & Linux</p>
		</div>

		<!-- Main layout -->
		<div class="main-layout">
			<!-- Left stepper -->
			<div class="stepper">
				<button
					class="step-item"
					class:step-done={windowsData !== null}
					class:step-active={activePanel === 'windows'}
					class:step-clickable={isStepClickable('windows')}
					onclick={() => selectStep('windows')}
				>
					<div class="step-dot">
						{#if windowsData}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12" /></svg>
						{:else if windowsLoading}
							<div class="dot-spinner"></div>
						{:else}
							<span class="dot-num">1</span>
						{/if}
					</div>
					<div class="step-info">
						<div class="step-label">Windows Source</div>
						<div class="step-sublabel">
							{#if windowsData}
								{deviceCount(windowsData)} device{deviceCount(windowsData) !== 1 ? 's' : ''} found
							{:else if windowsLoading}
								Scanning...
							{:else if windowsError}
								Error
							{:else}
								Select registry hive
							{/if}
						</div>
					</div>
				</button>

				<div class="step-line" class:line-done={windowsData !== null}></div>

				<button
					class="step-item"
					class:step-done={linuxData !== null}
					class:step-active={activePanel === 'linux'}
					class:step-clickable={isStepClickable('linux')}
					onclick={() => selectStep('linux')}
				>
					<div class="step-dot">
						{#if linuxData}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12" /></svg>
						{:else if linuxLoading}
							<div class="dot-spinner"></div>
						{:else}
							<span class="dot-num">2</span>
						{/if}
					</div>
					<div class="step-info">
						<div class="step-label">Linux Source</div>
						<div class="step-sublabel">
							{#if linuxData}
								{deviceCount(linuxData)} device{deviceCount(linuxData) !== 1 ? 's' : ''} found
							{:else if linuxLoading}
								Requesting access...
							{:else if linuxError}
								Error
							{:else}
								Grant root access
							{/if}
						</div>
					</div>
				</button>

				<div class="step-line" class:line-done={linuxData !== null}></div>

				<button
					class="step-item"
					class:step-active={activePanel === 'sync'}
					class:step-clickable={isStepClickable('sync')}
					onclick={() => selectStep('sync')}
				>
					<div class="step-dot">
						<span class="dot-num">3</span>
					</div>
					<div class="step-info">
						<div class="step-label">Sync Devices</div>
						<div class="step-sublabel">
							{#if canSync}
								Ready to match
							{:else}
								Waiting for data
							{/if}
						</div>
					</div>
				</button>
			</div>

			<!-- Right panel -->
			<div class="context-panel">
				{#if activePanel === 'windows'}
					<div class="panel-card">
						<div class="panel-title">Windows Bluetooth Data</div>
						{#if windowsLoading}
							<div class="panel-loading">
								<div class="loading-bar"><div class="loading-fill"></div></div>
								<p class="loading-text">Parsing SYSTEM registry hive...</p>
							</div>
						{:else if windowsError}
							<div class="panel-error">
								<div class="error-icon">!</div>
								<p class="error-msg">{windowsError}</p>
								<div class="btn-group">
									<button class="gf-btn" onclick={onSelectWindowsDir}>Select Directory</button>
									<button class="gf-btn ghost" onclick={onSelectWindowsHive}>Select File</button>
								</div>
							</div>
						{:else if windowsData}
							<div class="panel-data">
								<div class="summary-row">
									<div class="summary-card">
										<div class="summary-val">{windowsData.controllers.length}</div>
										<div class="summary-label">Controllers</div>
									</div>
									<div class="summary-card">
										<div class="summary-val">{deviceCount(windowsData)}</div>
										<div class="summary-label">Devices</div>
									</div>
								</div>
								{#each windowsData.controllers as controller}
									<div class="device-group">
										<div class="group-header">{controller.address}{controller.name ? ` - ${controller.name}` : ''}</div>
										{#each controller.devices as device}
											<button class="device-row device-row-btn" onclick={() => showDeviceDetails(device, 'Windows', controller.address)}>
												<span class="d-name">{device.name ?? 'Unknown'}</span>
												<span class="d-type">{device.device_type}</span>
											</button>
										{/each}
									</div>
								{/each}
								<button class="gf-btn ghost small mt-3" onclick={onClearWindows}>Clear</button>
							</div>
						{:else}
							<p class="panel-desc">
								Select your mounted Windows partition or the SYSTEM hive file directly to extract Bluetooth pairing keys.
							</p>
							{#if lastWindowsPath}
								<p class="last-path">Last used: {lastWindowsPath}</p>
							{/if}
							<div class="btn-group">
								<button class="gf-btn" onclick={onSelectWindowsDir}>Select Directory</button>
								<button class="gf-btn ghost" onclick={onSelectWindowsHive}>Select Hive File</button>
							</div>
						{/if}
					</div>
				{:else if activePanel === 'linux'}
					<div class="panel-card">
						<div class="panel-title-row">
							<div class="panel-title">Linux Bluetooth Data</div>
							{#if linuxError}
								<div class="error-badge">
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
										<path d="M18 6L6 18M6 6l12 12" />
									</svg>
									<span>Failed</span>
								</div>
							{/if}
						</div>
						{#if linuxLoading}
							<div class="panel-loading">
								<div class="loading-bar"><div class="loading-fill"></div></div>
								<p class="loading-text">Requesting elevated access...</p>
								<button class="gf-btn ghost small mt-3" onclick={onCancelLinuxAccess}>Cancel</button>
							</div>
						{:else if linuxError}
							<div class="panel-error">
								<p class="error-detail">{linuxError}</p>
								<p class="error-help">Enter your password when prompted to grant root access for reading Bluetooth configuration.</p>
								<div class="error-actions-row">
									<button class="gf-btn" onclick={onGrantLinuxAccess}>
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
											<path d="M1 4v6h6" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
										</svg>
										Try Again
									</button>
									<div class="auth-method-toggle">
										<button
											class="auth-option"
											class:auth-active={authMethod === 'pkexec'}
											onclick={() => authMethod = 'pkexec'}
										>pkexec</button>
										<button
											class="auth-option"
											class:auth-active={authMethod === 'sudo_askpass'}
											onclick={() => authMethod = 'sudo_askpass'}
										>sudo + askpass</button>
									</div>
								</div>
							</div>
						{:else if linuxData}
							<div class="panel-data">
								<div class="summary-row">
									<div class="summary-card">
										<div class="summary-val">{linuxData.controllers.length}</div>
										<div class="summary-label">Controllers</div>
									</div>
									<div class="summary-card">
										<div class="summary-val">{deviceCount(linuxData)}</div>
										<div class="summary-label">Devices</div>
									</div>
								</div>
								{#each linuxData.controllers as controller}
									<div class="device-group">
										<div class="group-header">{controller.address}{controller.name ? ` - ${controller.name}` : ''}</div>
										{#each controller.devices as device}
											<button class="device-row device-row-btn" onclick={() => showDeviceDetails(device, 'Linux', controller.address)}>
												<span class="d-name">{device.name ?? 'Unknown'}</span>
												<span class="d-type">{device.device_type}</span>
											</button>
										{/each}
									</div>
								{/each}
								<button class="gf-btn ghost small mt-3" onclick={onClearLinux}>Clear</button>
							</div>
						{:else}
							<p class="panel-desc">
								Grant elevated access to read Bluetooth configuration from /var/lib/bluetooth/.
							</p>
							<div class="error-actions-row">
								<button class="gf-btn" onclick={onGrantLinuxAccess}>Grant Root Access</button>
								<div class="auth-method-toggle">
									<button
										class="auth-option"
										class:auth-active={authMethod === 'pkexec'}
										onclick={() => authMethod = 'pkexec'}
									>pkexec</button>
									<button
										class="auth-option"
										class:auth-active={authMethod === 'sudo_askpass'}
										onclick={() => authMethod = 'sudo_askpass'}
									>sudo + askpass</button>
								</div>
							</div>
						{/if}
					</div>
				{:else}
					<div class="panel-card sync-card">
						<div class="panel-title">Ready to Sync</div>
						{#if canSync}
							<div class="sync-summary">
								<div class="sync-pair">
									<div class="sync-os">Windows</div>
									<div class="sync-count">{deviceCount(windowsData!)} devices</div>
								</div>
								<div class="sync-arrow">
									<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
										<path d="M7 16l5-5-5-5M17 8l-5 5 5 5" />
									</svg>
								</div>
								<div class="sync-pair">
									<div class="sync-os">Linux</div>
									<div class="sync-count">{deviceCount(linuxData!)} devices</div>
								</div>
							</div>
							<button class="gf-btn primary continue-btn" onclick={onContinueToSync}>
								Continue to Sync
								<svg class="continue-arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
									<path d="M5 12h14" />
									<path d="M13 6l6 6-6 6" />
								</svg>
							</button>
						{:else}
							<p class="panel-desc">Complete both scans above to proceed.</p>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

{#if detailsData}
	<DeviceDetailsDialog bind:open={detailsOpen} data={{ mode: 'single', device: detailsData.device, os: detailsData.os, controllerAddress: detailsData.controllerAddress }} />
{/if}

<style lang="css">
	.gf-header {
		text-align: center;
		margin-bottom: 2.25rem;
	}

	.logo-mark {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.08);
		margin-bottom: 1.25rem;
	}

	.gf-title {
		font-size: 32px;
		font-weight: 700;
		letter-spacing: -0.03em;
		margin: 0;
		background: linear-gradient(135deg, #fafafa 0%, #a78bfa 50%, #60a5fa 100%);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}

	.gf-tagline {
		color: rgba(250, 250, 250, 0.45);
		font-size: 15px;
		font-weight: 400;
		margin: 0.5rem 0 0;
	}

	.github-link {
		position: fixed;
		top: 12px;
		left: 12px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 6px;
		border: none;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.05);
		color: rgba(250, 250, 250, 0.4);
		cursor: pointer;
		transition: color 0.2s, background 0.2s;
		z-index: 10;
	}

	.github-link:hover {
		color: rgba(250, 250, 250, 0.8);
		background: rgba(255, 255, 255, 0.1);
	}

	/* Main layout */
	.main-layout {
		display: grid;
		grid-template-columns: 220px 1fr;
		gap: 2rem;
		align-items: start;
	}

	/* Stepper */
	.stepper {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
	}

	.step-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		margin: 0 -12px;
		border: none;
		background: transparent;
		font-family: inherit;
		border-radius: 8px;
		width: calc(100% + 24px);
		text-align: left;
		cursor: default;
		transition: background 0.2s;
	}

	.step-item.step-clickable {
		cursor: pointer;
	}

	.step-item.step-clickable:hover {
		background: rgba(255, 255, 255, 0.04);
	}

	.step-item.step-active {
		background: rgba(167, 139, 250, 0.06);
	}

	.step-dot {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 1.5px solid rgba(255, 255, 255, 0.12);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: rgba(255, 255, 255, 0.3);
		transition: all 0.3s;
	}

	.step-item.step-active .step-dot {
		border-color: #a78bfa;
		color: #a78bfa;
		box-shadow: 0 0 12px rgba(167, 139, 250, 0.2);
	}

	.step-item.step-done .step-dot {
		border-color: #22c55e;
		background: rgba(34, 197, 94, 0.1);
		color: #22c55e;
	}

	.dot-num {
		font-size: 13px;
		font-weight: 600;
	}

	.dot-spinner {
		width: 14px;
		height: 14px;
		border: 2px solid rgba(167, 139, 250, 0.3);
		border-top-color: #a78bfa;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.step-info {
		min-width: 0;
	}

	.step-label {
		font-size: 14px;
		font-weight: 500;
		color: #fafafa;
	}

	.step-sublabel {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
		margin-top: 1px;
	}

	.step-line {
		width: 1.5px;
		height: 24px;
		background: rgba(255, 255, 255, 0.08);
		margin-left: 15px;
		transition: background 0.3s;
	}

	.step-line.line-done {
		background: rgba(34, 197, 94, 0.3);
	}

	/* Context panel */
	.context-panel {
		min-height: 300px;
	}

	.panel-card {
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 12px;
		padding: 1.75rem;
		background: rgba(255, 255, 255, 0.02);
		backdrop-filter: blur(8px);
	}

	.panel-title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	.panel-title-row .panel-title {
		margin-bottom: 0;
	}

	.panel-title {
		font-size: 16px;
		font-weight: 600;
		margin-bottom: 1rem;
		color: #fafafa;
	}

	.panel-desc {
		font-size: 14px;
		line-height: 1.6;
		color: rgba(250, 250, 250, 0.45);
		margin: 0 0 1.25rem;
	}

	.last-path {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.25);
		margin: 0 0 1rem;
		word-break: break-all;
	}

	/* Loading */
	.panel-loading {
		padding: 1rem 0;
	}

	.loading-bar {
		height: 3px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 2px;
		overflow: hidden;
		margin-bottom: 1rem;
	}

	.loading-fill {
		width: 30%;
		height: 100%;
		background: linear-gradient(90deg, #a78bfa, #60a5fa);
		border-radius: 2px;
		animation: load-sweep 1.5s ease-in-out infinite;
	}

	@keyframes load-sweep {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(400%); }
	}

	.loading-text {
		font-size: 13px;
		color: rgba(250, 250, 250, 0.4);
		margin: 0;
	}

	/* Error */
	.panel-error {
		padding: 0.5rem 0;
	}

	.error-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 12px 4px 8px;
		border-radius: 20px;
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.15);
		color: #f87171;
		font-size: 13px;
		font-weight: 500;
	}

	.error-detail {
		font-size: 14px;
		color: rgba(250, 250, 250, 0.6);
		margin: 0 0 0.5rem;
		line-height: 1.5;
		font-weight: 500;
	}

	.error-help {
		font-size: 13px;
		color: rgba(250, 250, 250, 0.3);
		margin: 0 0 1.25rem;
		line-height: 1.5;
	}

	.error-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.2);
		color: #ef4444;
		font-size: 14px;
		font-weight: 700;
		margin-bottom: 0.75rem;
	}

	.error-msg {
		font-size: 13px;
		color: rgba(239, 68, 68, 0.7);
		margin: 0 0 1rem;
		line-height: 1.5;
	}

	/* Data display */
	.panel-data {
		padding: 0;
	}

	.summary-row {
		display: flex;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}

	.summary-card {
		text-align: center;
		padding: 8px 16px;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.06);
	}

	.summary-val {
		font-size: 24px;
		font-weight: 700;
		color: #fafafa;
		line-height: 1;
	}

	.summary-label {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.35);
		margin-top: 4px;
	}

	.device-group {
		margin-bottom: 0.75rem;
	}

	.group-header {
		font-size: 11px;
		color: rgba(250, 250, 250, 0.35);
		font-variant-numeric: tabular-nums;
		margin-bottom: 4px;
		letter-spacing: 0.02em;
	}

	.device-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 6px 10px;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.03);
		margin-bottom: 3px;
		font-size: 13px;
		transition: background 0.15s;
	}

	.device-row:hover {
		background: rgba(255, 255, 255, 0.05);
	}

	.device-row-btn {
		border: none;
		font-family: inherit;
		width: 100%;
		text-align: left;
		cursor: pointer;
	}

	.d-name {
		color: rgba(250, 250, 250, 0.7);
	}

	.d-type {
		font-size: 11px;
		color: rgba(250, 250, 250, 0.25);
		padding: 2px 8px;
		background: rgba(255, 255, 255, 0.04);
		border-radius: 6px;
	}

	/* Sync panel */
	.sync-card {
		border-color: rgba(167, 139, 250, 0.15);
	}

	.sync-summary {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.sync-pair {
		text-align: center;
	}

	.sync-os {
		font-size: 12px;
		color: rgba(250, 250, 250, 0.4);
		font-weight: 500;
		letter-spacing: 0.02em;
		margin-bottom: 2px;
	}

	.sync-count {
		font-size: 18px;
		font-weight: 600;
		color: #fafafa;
	}

	.sync-arrow {
		color: rgba(250, 250, 250, 0.2);
	}

	/* Continue button */
	.continue-btn {
		width: 100%;
		justify-content: center;
		padding: 10px 20px;
		font-size: 14px;
	}

	.continue-arrow {
		transition: transform 0.2s;
	}

	.continue-btn:hover .continue-arrow {
		transform: translateX(3px);
	}

	/* Buttons */
	.btn-group {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	/* Error actions row */
	.error-actions-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	/* Auth method toggle */
	.auth-method-toggle {
		display: inline-flex;
		border-radius: 8px;
		border: 1px solid rgba(255, 255, 255, 0.08);
		overflow: hidden;
		flex-shrink: 0;
	}

	.auth-option {
		padding: 5px 12px;
		font-size: 12px;
		font-family: inherit;
		font-weight: 400;
		color: rgba(250, 250, 250, 0.3);
		background: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.15s;
	}

	.auth-option:not(:last-child) {
		border-right: 1px solid rgba(255, 255, 255, 0.08);
	}

	.auth-option:hover {
		color: rgba(250, 250, 250, 0.5);
		background: rgba(255, 255, 255, 0.03);
	}

	.auth-option.auth-active {
		color: #a78bfa;
		background: rgba(167, 139, 250, 0.08);
		font-weight: 500;
	}
</style>
