import { RuneStore, type State } from '@tauri-store/svelte';
import type { BluetoothData, HostDistributions } from '#root/bindings';
import { SvelteMap } from 'svelte/reactivity';

interface WindowsState extends State {
	lastWindowsDirectory: string | null;
	lastWindowsHiveFile: string | null;
}

export const windowsState = new RuneStore<WindowsState>('windowsState', {
	lastWindowsDirectory: null,
	lastWindowsHiveFile: null
}, {
	autoStart: true,
	saveOnChange: true,

  // You can also debounce or throttle when saving.
  // This is optional. The default behavior is to save immediately.
  saveStrategy: 'debounce',
  saveInterval: 500,
});

interface BtStore extends State {
	windows: BluetoothData | null;
	linux: BluetoothData | null;
}

// Don't persis the bluetooth data, for security reasons.
export const btStore = new RuneStore<BtStore>('btStore', {
	windows: null,
	linux: null
}, {
	autoStart: true,
	saveOnChange: false,
	saveOnExit: false,

});
