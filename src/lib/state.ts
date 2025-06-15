import { RuneStore, type State } from '@tauri-store/svelte';

interface WindowsState extends State {
	lastWindowsDirectory: string | null;
	lastWindowsHiveFile: string | null;
}

export const windowsState = new RuneStore<WindowsState>('windowsState', {
	lastWindowsDirectory: null, lastWindowsHiveFile: null
}, {
	autoStart: true,
	saveOnChange: true,

  // You can also debounce or throttle when saving.
  // This is optional. The default behavior is to save immediately.
  saveStrategy: 'debounce',
  saveInterval: 500,
});
