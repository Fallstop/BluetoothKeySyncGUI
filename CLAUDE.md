# BluetoothKeySyncGUI

Tauri v2 desktop app that syncs Bluetooth pairing keys between Windows and Linux for dual-boot setups. Parses the Windows SYSTEM registry hive and Linux `/var/lib/bluetooth/` configs, lets users visually match devices on a node canvas, then writes the synced keys.

## Tech Stack

- **Frontend**: SvelteKit 2 (SSG via adapter-static) + Svelte 5 (runes) + Tailwind CSS 4 + Bits UI
- **Backend**: Tauri 2 + Rust (edition 2021) + Tokio
- **RPC**: taurpc + specta (pinned `=2.0.0-rc.22`) — auto-generates `bindings.ts` at project root
- **Node canvas**: @xyflow/svelte for device matching UI
- **Package manager**: pnpm

## Commands

```bash
pnpm install              # install frontend deps
pnpm tauri dev            # full dev (builds elevated binary + vite dev + Tauri backend)
pnpm tauri build          # production build
pnpm check                # svelte-check type checking
cargo build               # build Rust backend only
cargo check               # type-check Rust backend only
```

The `pnpm tauri dev/build` commands run `build-elevated-scrapper.sh` as a beforeDevCommand/beforeBuildCommand, which compiles the privilege-escalated scanner binary.

## Project Structure

```
src/                          # SvelteKit frontend (SSG, prerender, no SSR)
├── routes/
│   ├── +page.svelte          # Home: 3-step setup wizard
│   └── sync/Sync.svelte      # Node canvas for device matching
├── lib/
│   ├── api/index.ts          # taurpc proxy
│   ├── state.ts              # Svelte stores (windowsState, btStore)
│   └── components/
│       ├── bluetooth/        # Domain components (DevicesTree, Device, Controller)
│       │   └── syncing/nodeCanvas/  # @xyflow nodes/edges
│       └── ui/               # Bits UI primitives (button, card, dialog, progress)
src-tauri/
├── src/
│   ├── lib.rs                # App builder, plugin setup
│   ├── api/
│   │   ├── mod.rs            # taurpc router (merges all API handlers)
│   │   ├── windows_api.rs    # Windows SYSTEM hive parser (nt_hive2)
│   │   ├── linux_api.rs      # Linux BT config scanner (spawns elevated binary)
│   │   └── sync_api.rs       # Key writing (STUB — not yet implemented)
│   └── bluetooth/hive_parse.rs  # Windows registry parsing logic
├── bluetooth_model/          # Shared data types crate (BluetoothData, Controller, Device, Keys)
├── app_macros/               # Proc-macro crate (#[ipc_type] for serde + specta derives)
└── elevated_scrapper_standalone/  # Standalone binary that runs with root privileges
bindings.ts                   # Auto-generated TS types from specta (do not edit manually)
```

## Key Patterns

### RPC Layer (taurpc + specta)

Rust API traits in `src-tauri/src/api/` use `#[taurpc::procedures(path = "...")]`. Types shared across the boundary must use `#[app_macros::ipc_type]` which derives Serialize, Deserialize, specta::Type, Debug, and Clone. TypeScript bindings are auto-generated into `bindings.ts` during build.

Frontend calls: `await rpc.windows.parse_windows_hive(path)`, `rpc.linux.parse_local_config()`, etc.

### State Management

- **`windowsState`** — persistent (tauri-store/svelte), stores last selected Windows hive path
- **`btStore`** — non-persistent, cleared on exit for security (holds parsed Bluetooth data)

### Svelte 5 Runes

This project uses Svelte 5 runes (`$state`, `$derived`, `$effect`) — not legacy `let`-based reactivity or stores.

### Styling

Tailwind CSS 4 with `@tailwindcss/vite` plugin. Custom CSS variables for theming in `src/lib/app.css`. Dark mode via `mode-watcher`.

## Important Notes

- `sync_api.rs` is a **stub** — actual key writing logic is not yet implemented
- `specta` is pinned to `=2.0.0-rc.22` — do not change without testing taurpc compatibility
- The elevated_scrapper binary requires root/sudo to read `/var/lib/bluetooth/`
- `bindings.ts` is auto-generated — edit the Rust types, not this file
- No test suite is configured yet
