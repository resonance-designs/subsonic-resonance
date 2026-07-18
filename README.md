# Subsonic Resonance

A Windows-first, cross-platform OpenSubsonic streaming client written in Rust. One Leptos UI runs in browsers and inside a Tauri 2 desktop shell; provider integrations sit behind a provider-neutral Rust trait.

## Architecture

- `resonance-core`: domain models and the `MusicProvider` contract.
- `resonance-provider-subsonic`: OpenSubsonic authentication, browsing, search, artwork, and stream URLs.
- `resonance-server`: Axum browser backend/proxy. Credentials stay server-side.
- `resonance-ui`: shared Leptos CSR interface.
- `resonance-desktop`: Tauri 2 Windows/Desktop shell.

## Run the browser client

Install Trunk and the WebAssembly target once:

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

Start the API. Providers can now be added from Settings → Connections. Environment configuration remains available for an optional default provider:

```powershell
$env:RESONANCE_SERVER_URL='https://music.example.com/'
$env:RESONANCE_API_KEY='your-api-key'
cargo run -p resonance-server
```

In another terminal:

```powershell
Set-Location crates/ui
trunk serve index.html --address 127.0.0.1 --port 8080 --open
```

## Run the Windows desktop shell

Install the Tauri CLI, then run from the repository root:

```powershell
cargo install tauri-cli --version '^2' --locked
cargo tauri dev --config crates/desktop/tauri.conf.json
```

Windows development requires Microsoft C++ Build Tools and WebView2. The current desktop slice hosts the shared UI; connecting its secure credential storage and native media controls is the next milestone.

## Configuration

| Variable               | Purpose                            | Default          |
| ---------------------- | ---------------------------------- | ---------------- |
| `RESONANCE_SERVER_URL` | Optional default Subsonic URL      | optional         |
| `RESONANCE_API_KEY`    | Preferred OpenSubsonic API key     | optional         |
| `RESONANCE_USERNAME`   | Username for salted-token fallback | optional         |
| `RESONANCE_PASSWORD`   | Password for salted-token fallback | optional         |
| `RESONANCE_BIND`       | Web API listen address             | `127.0.0.1:3000` |

Providers added from the web interface are verified immediately and retained in backend memory. Their secrets are never written to browser storage. Restarting the backend clears UI-added providers; durable storage in the operating-system credential vault is planned for the desktop integration.
