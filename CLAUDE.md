# Stark-Link

Cross-device control & communication platform.

## Project Structure
- `stark-link-core/` — Rust core library (crypto, discovery, protocol, transfer, clipboard, connection)
- `stark-link-desktop/` — Tauri 2.0 + React + TypeScript desktop app
- `relay-server/` — Node.js + TypeScript WebSocket relay server
- `.github/workflows/` — CI/CD pipelines

## Build Commands
- Core: `cd stark-link-core && cargo build`
- Desktop: `cd stark-link-desktop && npm install && npm run tauri dev`
- Relay: `cd relay-server && npm install && npm run dev`
- Full test: `cd stark-link-core && cargo test`

## Conventions
- Rust: use `thiserror` for errors, `serde` for serialization, `tokio` for async
- Frontend: React functional components, TypeScript strict mode
- All messages use MessagePack binary serialization over the wire
- All traffic encrypted with AES-256-GCM after X25519 key exchange
