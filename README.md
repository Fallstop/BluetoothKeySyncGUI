# Bluetooth Key Sync

A desktop app that syncs Bluetooth pairing keys between Windows and Linux, so you don't have to re-pair your devices every time you switch operating systems.

Built with Tauri 2, SvelteKit, and Rust.

## The Problem

When you pair a Bluetooth device, both your computer and the device store a shared secret key. The trouble is that your Bluetooth adapter has the same MAC address on both Windows and Linux - so when you pair a device on one OS, it overwrites the key the device had stored for the other. You end up having to re-pair every time you reboot into a different OS.

The [classic workaround](https://unix.stackexchange.com/a/255510/704807) involves manually extracting keys from the Windows registry and copying them into Linux's BlueZ config files. It works, but it's tedious, error-prone, and breaks down with anything beyond simple devices.

Existing CLI tools that attempt to automate this tend to fall apart when they encounter device types the author didn't test - dual-mode devices with both Classic and LE keys, BlueZ config quirks, or devices with rolling MAC addresses like the Logitech MX Master series.

## What This Does

Bluetooth Key Sync reads pairing data from both operating systems, lets you visually match devices, and writes the synced keys - handling the edge cases that trip up other tools.

**Step 1 - Load Windows data.** Point the app at your Windows partition (or a SYSTEM registry hive file directly). It parses the Bluetooth pairing keys from the registry without needing Windows to be running.

**Step 2 - Load Linux data.** The app spawns an elevated helper binary to read BlueZ's config files from `/var/lib/bluetooth/`. You'll be prompted for your password.

**Step 3 - Match and sync.** Devices are auto-matched where possible. For anything ambiguous, you can manually pair them. Choose a sync direction and apply - the app writes the keys to the target OS config.

### Supported Device Types

- Bluetooth Classic
- Bluetooth Low Energy (LE)
- Dual-mode devices (Classic + LE)
- Devices with rolling MAC addresses

## How It Works

The frontend communicates with the Rust backend through [taurpc](https://github.com/oscartbeaumont/tauri-specta), which auto-generates TypeScript bindings from Rust types via [specta](https://github.com/oscartbeaumont/specta).

Windows pairing data is extracted from the SYSTEM registry hive using [nt_hive2](https://crates.io/crates/nt_hive2) - no Windows APIs needed, just the raw file from a mounted partition. Linux data comes from BlueZ's config files under `/var/lib/bluetooth/`, which requires root access to read. A separate elevated binary handles this, communicating with the main app over stdin/stdout.

Parsed data from both sides is normalized into a common `BluetoothData` model (controllers, devices, and their associated keys), which the frontend uses to present the matching interface and generate sync proposals.

## Design Goals

- **Progressive disclosure.** The default flow should be approachable for someone who's never touched a terminal. Advanced options are available but not in the way.
- **Graceful failure.** Bluetooth pairing has countless edge cases. The app should handle what it can and clearly report what it can't, rather than silently corrupting configs.
- **Broad device support.** If a device pairs over Bluetooth, this tool should be able to sync it - Classic, LE, dual-mode, rolling MAC, all of it.

## License

MIT - see [LICENSE.md](./LICENSE.md).
