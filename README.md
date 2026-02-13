### IRC Client (Tauri + SvelteKit)

A desktop IRC client built with Tauri (Rust backend) and SvelteKit + TypeScript frontend. The app focuses on a clean two-panel UI with server/channel management, real-time message updates, and IRC event handling powered by a custom Rust IRC core.

### Features

- Connect to IRC servers with host/port/TLS/nickname settings
- Join channels and track members, topics, and unread counters
- Real-time event handling for `JOIN`, `PART`, `QUIT`, `NICK`, `TOPIC`, and messages
- Message view with user/system message rendering
- Server and channel management modals

### Tech Stack

- Frontend: `SvelteKit`, `TypeScript`, `Tailwind CSS`
- Backend: `Rust`, `Tauri v2`, custom `kirc` IRC core
- Communication: Tauri `invoke` commands and `kirc:event` event stream

### Architecture Overview

#### Frontend (SvelteKit)

- `src/routes/+page.svelte` listens to `kirc:event` and updates the UI state.
- `src/stores/stores.svelte.ts` stores servers, channels, current selection, and unread counts.
- `src/types/*.svelte.ts` defines UI payloads and IRC domain types.
- UI is split into server/channel sidebar, `MessageView`, and message input panel.

#### Backend (Rust + Tauri)

- `src-tauri/src/lib.rs` registers Tauri plugins, menu, state, and commands.
- `kirc` module handles IRC connection lifecycle, message routing, and event emission.
- Commands exposed to frontend:
  - `connect_server`
  - `join_channel`
  - `send_message`
- Events are emitted to the frontend as `kirc:event` payloads.

### Project Structure

```
.
├── src/                      # SvelteKit frontend
│   ├── routes/               # UI pages and components
│   ├── stores/               # Svelte stores
│   └── types/                # UI/IRC types and payloads
├── src-tauri/                # Rust + Tauri backend
│   ├── src/                  # Rust source (kirc, commands, state)
│   └── tauri.conf.json       # Tauri build configuration
├── package.json              # Frontend scripts and deps
└── README.md
```

### Getting Started

#### Prerequisites

- Node.js (for SvelteKit/Vite)
- Rust toolchain (for Tauri backend)
- Deno (optional; see note below)

#### Install Dependencies

```
npm install
```

#### Run in Development

```
npm run tauri dev
```

#### Build for Production

```
npm run tauri build
```

#### Deno Task Note

`src-tauri/tauri.conf.json` currently uses `deno task dev` and `deno task build` for `beforeDevCommand` and `beforeBuildCommand`. If you prefer `npm`, update those commands to `npm run dev` and `npm run build`, or provide a `deno.json` with equivalent tasks.

### License

MIT
