# Void Drift Binary

## Overview
Standalone Tauri desktop app wrapping the Void Drift space shooter. Rust backend replaces Express/Socket.io relay.

## Architecture
- Tauri v2 desktop app with Rust backend
- Frontend: `src/index.html` — extracted client from VoidDrift's inline template
- Backend: `src-tauri/src/` — axum + tokio-tungstenite WebSocket relay on port 3800
- Sounds bundled as Tauri resources in `src-tauri/sounds/`
- LAN multiplayer: host runs in Tauri app, guests connect via browser at `http://<ip>:3800/`

## Key Commands
- `cargo tauri dev` — development mode
- `cargo tauri build` — production build (.app/.dmg on macOS)

## Port
Uses port 3800 for the WebSocket relay and static file serving.
