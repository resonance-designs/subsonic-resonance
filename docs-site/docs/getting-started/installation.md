---
title: Installation and development
sidebar_position: 1
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Installation and development

## Prerequisites

- Stable Rust with the MSVC toolchain on Windows
- `wasm32-unknown-unknown` Rust target
- Trunk for the browser UI
- Microsoft C++ Build Tools and WebView2 for Tauri on Windows

Install the browser tooling once:

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

## Browser client

Start the Rust backend from the repository root:

```powershell
cargo run -p resonance-server
```

The backend listens on `http://127.0.0.1:3000` by default. Environment credentials are optional because providers can be added from the UI.

In a second terminal:

```powershell
Set-Location crates/ui
trunk serve index.html --address 127.0.0.1 --port 8080 --open
```

Open `http://127.0.0.1:8080`, then use Settings → Connections → Add Provider. The backend verifies the connection before it appears in the source list.

If either process was already running when the code changed, stop it with `Ctrl+C` and restart it.

## Optional default provider

The backend can register one provider at startup from environment variables. Prefer API-key authentication:

```powershell
$env:RESONANCE_SERVER_URL='https://music.example.com/'
$env:RESONANCE_API_KEY='your-api-key'
cargo run -p resonance-server
```

For salted password-token authentication:

```powershell
$env:RESONANCE_SERVER_URL='https://music.example.com/'
$env:RESONANCE_USERNAME='listener'
$env:RESONANCE_PASSWORD='your-password'
cargo run -p resonance-server
```

### Environment variables

| Variable | Purpose | Default |
| --- | --- | --- |
| `RESONANCE_SERVER_URL` | Optional default Subsonic server URL | unset |
| `RESONANCE_API_KEY` | API key for the default provider | unset |
| `RESONANCE_USERNAME` | Username for password authentication | unset |
| `RESONANCE_PASSWORD` | Password used to generate salted tokens | unset |
| `RESONANCE_BIND` | Axum API listen address | `127.0.0.1:3000` |
| `RUST_LOG` | Rust tracing filter | built-in application defaults |

## Windows desktop shell

Install the Tauri CLI:

```powershell
cargo install tauri-cli --version '^2' --locked
cargo tauri dev --config crates/desktop/tauri.conf.json
```

The desktop shell is scaffolded, but it does not yet launch or embed the Axum provider service, use the Windows credential vault, or expose native media controls. The browser workflow is the currently supported development path.

## Development checks

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace

Set-Location crates/ui
trunk build index.html
```

The root Node project also exposes shortcuts:

```powershell
npm run cargo:build
npm run server:start
```

`cargo:build` compiles the complete Rust workspace. `server:start` starts the Axum backend for local development.
