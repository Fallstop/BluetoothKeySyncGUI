<script lang="ts">
	import { rpc } from '$lib/api';
	import { appSettings } from '$lib/state';
	import { getVersion } from '@tauri-apps/api/app';
	import { openUrl } from '@tauri-apps/plugin-opener';

	let updateAvailable = $state(false);
	let latestVersion = $state('');
	let releaseUrl = $state('');
	let visible = $state(false);

	let dismissed = $derived(
		appSettings.state.dismissedUpdateVersion === latestVersion
	);

	$effect(() => {
		const timeout = setTimeout(async () => {
			try {
				const currentVersion = await getVersion();
				const result = await rpc.updates.check_for_update(currentVersion);
				if (result.type === 'Success' && result.data.update_available) {
					updateAvailable = true;
					latestVersion = result.data.latest_version;
					releaseUrl = result.data.release_url;
					visible = true;
				}
			} catch {
				// Silent failure
			}
		}, 3000);

		return () => clearTimeout(timeout);
	});

	function dismiss() {
		appSettings.state.dismissedUpdateVersion = latestVersion;
		visible = false;
	}

	function viewRelease() {
		openUrl(releaseUrl);
	}
</script>

{#if updateAvailable && visible && !dismissed}
	<div class="update-banner">
		<div class="update-content">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
				<polyline points="7 10 12 15 17 10" />
				<line x1="12" y1="15" x2="12" y2="3" />
			</svg>
			<span class="update-text">
				v{latestVersion} available
			</span>
			<button class="update-view-btn" onclick={viewRelease}>
				View Release
			</button>
			<button class="update-dismiss-btn" onclick={dismiss} aria-label="Dismiss">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<line x1="18" y1="6" x2="6" y2="18" />
					<line x1="6" y1="6" x2="18" y2="18" />
				</svg>
			</button>
		</div>
	</div>
{/if}

<style>
	.update-banner {
		position: fixed;
		bottom: 20px;
		right: 20px;
		z-index: 50;
		animation: slide-up 0.3s ease-out;
	}

	.update-content {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: 10px;
		border: 1px solid rgba(96, 165, 250, 0.25);
		background: rgba(96, 165, 250, 0.08);
		backdrop-filter: blur(12px);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
		color: rgba(250, 250, 250, 0.85);
		font-size: 13px;
	}

	.update-content > svg {
		color: var(--accent-blue, #60a5fa);
		flex-shrink: 0;
	}

	.update-text {
		font-weight: 500;
		white-space: nowrap;
	}

	.update-view-btn {
		padding: 4px 10px;
		border-radius: 6px;
		border: 1px solid rgba(96, 165, 250, 0.3);
		background: rgba(96, 165, 250, 0.15);
		color: #93c5fd;
		font-family: inherit;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.update-view-btn:hover {
		background: rgba(96, 165, 250, 0.25);
		border-color: rgba(96, 165, 250, 0.5);
	}

	.update-dismiss-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border-radius: 4px;
		border: none;
		background: transparent;
		color: rgba(250, 250, 250, 0.35);
		cursor: pointer;
		transition: all 0.2s;
	}

	.update-dismiss-btn:hover {
		color: rgba(250, 250, 250, 0.7);
		background: rgba(255, 255, 255, 0.06);
	}

	@keyframes slide-up {
		from {
			opacity: 0;
			transform: translateY(12px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
