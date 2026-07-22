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
- Optional: Codex CLI, authenticated with access to GPT-5.6 Sol, for AI-authored Lighthouse accessibility assessments

Install the browser tooling once:

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

## Browser client

Start the Rust backend from the repository root:

```powershell
cargo run -p subsonic-resonance-server
```

The backend listens on `http://127.0.0.1:3000` by default. Environment credentials are optional because providers can be added from the UI.

In a second terminal:

```powershell
Set-Location crates/ui
trunk serve index.html --address 127.0.0.1 --port 8088 --open
```

Open `http://127.0.0.1:8088`, then use Settings → Connections → Add Provider. The backend verifies the connection before it appears in the source list.

If either process was already running when the code changed, stop it with `Ctrl+C` and restart it.

## Optional default provider

The backend can register one provider at startup from environment variables. Prefer API-key authentication:

```powershell
$env:RESONANCE_SERVER_URL='https://music.example.com/'
$env:RESONANCE_API_KEY='your-api-key'
cargo run -p subsonic-resonance-server
```

For salted password-token authentication:

```powershell
$env:RESONANCE_SERVER_URL='https://music.example.com/'
$env:RESONANCE_USERNAME='listener'
$env:RESONANCE_PASSWORD='your-password'
cargo run -p subsonic-resonance-server
```

### Environment variables

| Variable               | Purpose                                 | Default                       |
| ---------------------- | --------------------------------------- | ----------------------------- |
| `RESONANCE_SERVER_URL` | Optional default Subsonic server URL    | unset                         |
| `RESONANCE_API_KEY`    | API key for the default provider        | unset                         |
| `RESONANCE_USERNAME`   | Username for password authentication    | unset                         |
| `RESONANCE_PASSWORD`   | Password used to generate salted tokens | unset                         |
| `RESONANCE_BIND`       | Axum API listen address                 | `127.0.0.1:3000`              |
| `RUST_LOG`             | Rust tracing filter                     | built-in application defaults |

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
npm run ui:start
npm run app:start
```

`cargo:build` compiles the complete Rust workspace. `server:start` starts only the Axum API on port 3000, while `ui:start` starts only the browser UI on port 8088. Use `app:start` to run both processes together.

Windows developers can also use the interactive build launcher. It builds the complete Cargo workspace and then prompts whether to start both the API and browser UI or only the API:

```powershell
npm run build:interactive:win
```

The underlying script can also be invoked directly with `.\scripts\build-and-run.ps1`.

On Linux and macOS, use the equivalent Bash launcher:

```bash
npm run build:interactive:unix
```

The Bash script can also be invoked directly with `bash scripts/build-and-run.sh`. Both launchers remain attached after startup so `Ctrl+C` can reliably stop the processes and release their ports. Before compiling or starting a service, the launchers check required ports; Windows also detects a running `subsonic-resonance-server.exe` that would lock the Cargo build artifact. When conflicts are found, the launcher identifies the processes and asks for permission before stopping them. Declining exits without building or starting Resonance.
