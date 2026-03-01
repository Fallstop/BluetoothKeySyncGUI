<script lang="ts">
	import SetupWizard from '$lib/components/SetupWizard.svelte';
	import { goto } from '$app/navigation';
	import { open } from '@tauri-apps/plugin-dialog';
	import { rpc } from '@/api';
	import { appSettings, btStore, windowsState } from '@/state';

	let windowsLoading = $state(false);
	let linuxLoading = $state(false);
	let windowsError: string | null = $state(null);
	let linuxError: string | null = $state(null);

	async function readHiveFile(path: string) {
		windowsLoading = true;
		windowsError = null;
		try {
			const response = await rpc.windows.parse_windows_hive(path);
			if (response.type === 'Error') {
				windowsError = response.data;
				return;
			}
			windowsState.state.lastWindowsHiveFile = response.data.source_path;
			btStore.state.windows = response.data;
		} catch (e) {
			windowsError = e instanceof Error ? e.message : String(e);
		} finally {
			windowsLoading = false;
		}
	}

	async function onSelectWindowsDir() {
		const windowsPath = await open({
			multiple: false,
			directory: true,
			title: 'Select the Windows directory',
			defaultPath: windowsState.state.lastWindowsDirectory || undefined
		});
		if (!windowsPath) return;
		windowsState.state.lastWindowsDirectory = windowsPath;
		readHiveFile(windowsPath);
	}

	async function onSelectWindowsHive() {
		const file = await open({
			multiple: false,
			directory: false,
			title: 'Select the Windows SYSTEM hive file',
			defaultPath: windowsState.state.lastWindowsHiveFile || undefined
		});
		if (!file) return;
		windowsState.state.lastWindowsHiveFile = file;
		readHiveFile(file);
	}

	async function onGrantLinuxAccess() {
		linuxLoading = true;
		linuxError = null;
		try {
			const response = await rpc.linux.parse_local_config(appSettings.state.authMethod);
			if (response.type === 'Error') {
				linuxError = response.data;
				return;
			}
			btStore.state.linux = response.data;
		} catch (e) {
			linuxError = e instanceof Error ? e.message : String(e);
		} finally {
			linuxLoading = false;
		}
	}

	async function onCancelLinuxAccess() {
		try {
			await rpc.linux.cancel_linux_access();
		} catch {
			// ignore cancellation errors
		}
		linuxLoading = false;
		linuxError = 'Cancelled';
	}

	function onContinueToSync() {
		goto('/sync');
	}

	function onClearWindows() {
		btStore.state.windows = null;
		windowsError = null;
	}

	function onClearLinux() {
		btStore.state.linux = null;
		linuxError = null;
	}
</script>

<SetupWizard
	windowsData={btStore.state.windows}
	linuxData={btStore.state.linux}
	lastWindowsPath={windowsState.state.lastWindowsHiveFile || windowsState.state.lastWindowsDirectory}
	{onSelectWindowsDir}
	{onSelectWindowsHive}
	{onGrantLinuxAccess}
	{onCancelLinuxAccess}
	{onContinueToSync}
	{onClearWindows}
	{onClearLinux}
	{windowsLoading}
	{linuxLoading}
	{windowsError}
	{linuxError}
	bind:authMethod={appSettings.state.authMethod}
/>
