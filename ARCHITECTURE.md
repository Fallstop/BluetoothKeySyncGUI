# Bluetooth Key Sync GUI — Architecture Reference

## What This Project Does

A Tauri v2 desktop app that synchronizes Bluetooth pairing keys between Windows and Linux. When dual-booting, Bluetooth devices store separate pairing keys per OS, so switching OS requires re-pairing. This tool reads keys from both OSes and syncs them.

## Tech Stack

| Layer    | Technology                          | Version |
|----------|-------------------------------------|---------|
| Frontend | SvelteKit (SSG via adapter-static)  | 2.x     |
| UI       | Svelte 5 (runes), Bits UI, Tailwind CSS 4 | 5.x / 2.x / 4.x |
| Canvas   | @xyflow/svelte (node graph)         | 1.1.x   |
| Backend  | Tauri 2, Rust, Tokio                | 2.5.x   |
| RPC      | taurpc + specta (typed IPC)         | 0.5.1 / 2.0.0-rc.22 |
| Package  | pnpm (frontend), Cargo workspace (backend) | — |

## Directory Structure

```
├── src/                          # SvelteKit frontend
│   ├── routes/
│   │   ├── +page.svelte          # Home — 3-step setup wizard
│   │   ├── ConfigWindows.svelte  # Step 1: select Windows SYSTEM hive
│   │   ├── ConfigLinux.svelte    # Step 2: elevate & scan Linux BT config
│   │   └── sync/
│   │       └── Sync.svelte       # Step 3: node canvas for matching devices
│   └── lib/
│       ├── api/index.ts          # taurpc proxy (imports from bindings.ts)
│       ├── state.ts              # Svelte stores (windowsState, btStore)
│       └── components/
│           ├── bluetooth/        # Device tree, controller, device components
│           │   └── syncing/
│           │       ├── ChangeRequest.svelte        # Sync approval dialog
│           │       └── nodeCanvas/                  # @xyflow canvas components
│           │           ├── DeviceCanvas.svelte
│           │           ├── DeviceNode.svelte
│           │           ├── ControllerGroupNode.svelte
│           │           └── FloatingEdge.svelte
│           └── ui/               # Bits UI primitives (button, card, dialog, etc.)
│
├── bindings.ts                   # Auto-generated TS types from specta/taurpc
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri app builder & plugin registration
│   │   ├── main.rs               # Entry point
│   │   └── api/
│   │       ├── mod.rs            # taurpc router setup
│   │       ├── message.rs        # Message<T> response envelope (Success/Error)
│   │       ├── windows_api.rs    # parse_windows_hive() — reads SYSTEM hive
│   │       ├── linux_api.rs      # parse_local_config() — spawns elevated scanner
│   │       └── sync_api.rs       # apply_sync_proposals() — STUB, not yet implemented
│   │
│   ├── bluetooth_model/          # Shared data model crate
│   │   └── src/bluetooth_data.rs # BluetoothData, Controller, Device, LinkKey, LEKey
│   │
│   ├── elevated_scrapper_standalone/  # Standalone binary for privilege escalation
│   │   └── src/
│   │       ├── main.rs           # CLI entry, outputs JSON to stdout
│   │       └── scan_filesystem.rs # Reads /var/lib/bluetooth/ (needs root)
│   │
│   └── app_macros/               # Proc-macro crate (#[ipc_type] for specta)
│
├── build-elevated-scrapper.sh    # Builds the elevated scanner binary
├── package.json                  # Frontend deps (pnpm)
└── pnpm-lock.yaml
```

## How It Works

### Data Flow

```
Windows SYSTEM hive file
        │
        ▼
  windows_api.rs ──► nt_hive2 parses registry
        │               keys from BTHPORT\Parameters\Keys
        ▼
   BluetoothData ──────────────────────────┐
                                           ▼
                                      btStore (Svelte)
                                           │
                                      Sync.svelte
                                      matches controllers
                                      by MAC address
                                           │
                                      DeviceCanvas
                                      (@xyflow nodes)
                                           │
                                      User draws edges
                                      connecting devices
                                           │
                                      ChangeRequest
                                      generates SyncProposals
                                           │
                                      sync_api.rs (STUB)
                                           ▼
                                      Write keys to target OS
                                      (NOT YET IMPLEMENTED)

/var/lib/bluetooth/
        │
        ▼
  elevated_scrapper (sudo)
        │ JSON on stdout
        ▼
  linux_api.rs ──► parses JSON
        │
        ▼
   BluetoothData ──────────────────────────┘
```

### RPC Layer

Frontend communicates with backend via **taurpc** — a typed RPC layer that auto-generates TypeScript bindings from Rust trait definitions via **specta**.

- Rust: `#[taurpc::procedures]` traits in `api/*.rs`
- Generated: `bindings.ts` at project root
- Frontend: `import { rpc } from "@/api"` then `rpc.windows.parse_windows_hive(path)`

### State Management

- **`windowsState`** — Persistent (tauri-store), saves last selected Windows directory/file path
- **`btStore`** — Non-persistent (intentionally), holds parsed BluetoothData for both OSes. Keys are sensitive and cleared on exit.

## Current Development Status

**Implemented:**
- Windows SYSTEM hive parsing (classic + LE keys)
- Linux /var/lib/bluetooth scanning with privilege escalation
- Setup wizard UI (file selection, elevation prompt)
- Node-based visual device matching canvas
- Change request / sync proposal generation UI

**Not yet implemented:**
- `sync_api.rs` — actual key writing logic is a stub
- Key format conversion between Windows ↔ Linux
- Rolling MAC address handling
