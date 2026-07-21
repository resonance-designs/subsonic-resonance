# Subsonic Resonance

![Static Badge](https://img.shields.io/badge/Version-0.1.5-orange)
![Static Badge](https://img.shields.io/badge/Latest_Release-v0.1.5-green)

Subsonic Resonance is a Windows-first, cross-platform OpenSubsonic streaming client written in Rust. It uses a shared Leptos/WebAssembly interface for the browser and Tauri desktop shell, with provider integrations behind a provider-neutral Rust API.

The project is currently at `0.1.5` and under active development toward the `0.2.0` library-experience release.

## Current functionality

- Connect multiple Subsonic or OpenSubsonic servers from Settings → Connections.
- Authenticate with an OpenSubsonic API key or Subsonic salted password token.
- Fall back to legacy directory-based browsing for servers that omit modern ID3 browsing results.
- Verify new connections with `ping` before registering them.
- Query every connected provider concurrently as one unified library.
- Browse albums from all available providers without selecting a globally active connection.
- Retrieve successive provider album pages instead of limiting each collection to its first 30 entries.
- Filter Albums, Artists, Playlists, and Search by provider without disabling or changing any configured connection.
- Sort the dedicated Albums page by title, artist, year, and source.
- Browse and filter artists from every connected source, with album-count sorting and provider attribution.
- Open provider-qualified artist pages to browse every release reported for that artist and inspect or play individual release tracks.
- Browse server playlists from every connected source, load provider-qualified playlist tracks, and play them through the shared queue.
- Search every connected library from the dedicated Search page.
- Load provider-qualified album tracks and play results from Home, Albums, or Search.
- Continue displaying successful sources when another provider is offline, unauthorized, invalid, or times out.
- Proxy cover artwork and audio without exposing credentials to the browser.
- Stream audio through the browser with byte-range support for seeking.
- Randomize the current queue, repeat the current track, or repeat the complete album, playlist, or search queue for the active session.
- Persist the provider-qualified playback queue, selected track, in-track position, volume, mute, shuffle, and repeat preferences across browser reloads, with previous, next, inspect, remove, and clear controls.
- Control playback, seeking, volume, shuffle, repeat, and the persistent queue from one unified player frame.
- Browse other albums, playlists, searches, and application views without replacing the active queue or interrupting the current stream.
- Run the same Leptos UI in a browser or Tauri 2 desktop shell.

Home, Albums, Artists, Playlists, and Search now use the unified library service. Creating, editing, and persisting mixed-source playlists remains future work.

## Architecture

| Crate                         | Responsibility                                                                  |
| ----------------------------- | ------------------------------------------------------------------------------- |
| `resonance-core`              | Provider-neutral domain models and the `MusicProvider` contract.                |
| `resonance-provider-subsonic` | OpenSubsonic authentication, browsing, search, artwork, and stream URL support. |
| `resonance-server`            | Axum provider registry, browser API, and credential-aware media proxy.          |
| `subsonic-resonance-ui`       | Shared Leptos CSR interface compiled to WebAssembly.                            |
| `resonance-desktop`           | Tauri 2 Windows/desktop shell.                                                  |

```text
      Browser / Tauri UI
              │
              ▼
Axum API and provider registry
              │
              ▼
      MusicProvider trait
              │
              ▼
OpenSubsonic / Subsonic server
```

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

## Run the browser client

Start the Rust backend from the repository root:

```powershell
cargo run -p resonance-server
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

| Variable               | Purpose                                 | Default                       |
| ---------------------- | --------------------------------------- | ----------------------------- |
| `RESONANCE_SERVER_URL` | Optional default Subsonic server URL    | unset                         |
| `RESONANCE_API_KEY`    | API key for the default provider        | unset                         |
| `RESONANCE_USERNAME`   | Username for password authentication    | unset                         |
| `RESONANCE_PASSWORD`   | Password used to generate salted tokens | unset                         |
| `RESONANCE_BIND`       | Axum API listen address                 | `127.0.0.1:3000`              |
| `RUST_LOG`             | Rust tracing filter                     | built-in application defaults |

## Run the Windows desktop shell

Install the Tauri CLI:

```powershell
cargo install tauri-cli --version '^2' --locked
cargo tauri dev --config crates/desktop/tauri.conf.json
```

The desktop shell is scaffolded, but it does not yet launch or embed the Axum provider service, use the Windows credential vault, or expose native media controls. The browser workflow is the currently supported development path.

## Security and persistence

- Provider secrets are sent only to the local Rust backend and the configured music server.
- Passwords and API keys are not written to browser `localStorage`.
- Providers added from the UI are held in backend memory and survive browser refreshes.
- Restarting the backend clears UI-added providers; the optional environment provider is recreated at startup.
- TLS certificates are validated. A server using an untrusted self-signed certificate will be rejected.

Durable secrets should eventually be stored through the operating-system credential vault, not in browser storage or plaintext configuration.

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

The Bash script can also be invoked directly with `bash scripts/build-and-run.sh`. Both launchers remain attached after startup so `Ctrl+C` can reliably stop the processes and release their ports. Before compiling or starting a service, the launchers check required ports; Windows also detects a running `resonance-server.exe` that would lock the Cargo build artifact. When conflicts are found, the launcher identifies the processes and asks for permission before stopping them. Declining exits without building or starting Resonance.

## Documentation site

The Docusaurus project in `docs-site` generates project documentation from this README, `TODO.md`, `CHANGELOG.md`, and `LICENSING.md`.

The site uses the same dark application palette and Subsonic/Resonance brand lockup as the client. Its responsive landing-page hero pairs the project identity and primary documentation links with a current Albums-library screenshot, and the global footer links to project documentation, development resources, releases, and licensing guidance. The Docusaurus light/dark toggle and template theme selector are disabled so the published site consistently uses the Resonance theme.

Install the Node dependencies once:

```powershell
npm install
pnpm --dir docs-site install --ignore-scripts
```

Synchronize or run the documentation site:

```powershell
npm run docs:sync
npm run docs:start
npm run docs:build
npm run docs:quality
```

The Lighthouse assessment runner has two modes:

| Target                           | Offline deterministic     | AI-authored with Codex       |
| -------------------------------- | ------------------------- | ---------------------------- |
| Browser application on port 8088 | `npm run lighthouse`      | `npm run lighthouse:ai`      |
| Documentation site on port 3001  | `npm run lighthouse:docs` | `npm run lighthouse:docs:ai` |

Every mode writes timestamped Lighthouse JSON and Markdown accessibility-assessment files beneath `artifacts/lighthouse`. Offline mode is the default, requires no AI service, and renders the reusable template directly from Lighthouse evidence. AI mode runs the same audit, creates the deterministic draft, and then starts a new read-only, ephemeral Codex session to author the final assessment. It does not resume or share context with an existing Codex conversation.

AI mode requires the optional Codex CLI prerequisite. Run `codex login` if it is not already authenticated. GPT-5.6 Sol is used by default; set `CODEX_ASSESSMENT_MODEL` to an available Codex model name to override it. The Codex CLI uses the authenticated account and applicable usage limits—installing the CLI does not run GPT-5.6 Sol locally. If Codex generation fails or its response lacks required report sections, the command exits unsuccessfully while retaining the deterministic assessment so the Lighthouse evidence is not lost.

The generated report distinguishes automated evidence from the manual keyboard, screen-reader, workflow, and other human review required to establish WCAG 2.2 AA conformance. For another target, use `node scripts/run-lighthouse.js <url> --label <name>` for offline mode or append `--ai` for AI mode.

Use `npm run version:sync -- 0.1.2` to synchronize an explicit project version across Cargo, Node, and documentation metadata. Without an argument, the script uses the current Cargo workspace version.

## Release automation

Preview release branch and tag creation without changing Git:

```powershell
npm run git:release:dry-run
```

After committing all intended changes, create and publish the release:

```powershell
npm run git:release
```

The release command reads the Cargo workspace version, confirms all Cargo, Node, documentation, and changelog versions agree, requires a clean worktree, runs the Rust tests and documentation build, creates `release/<version>`, creates the annotated `v<version>` tag, and pushes the branch and tag to `origin` atomically. It stops without changing Git when a local or remote release branch/tag already exists. Pass `-- --skip-checks` only when the release checks have already been completed independently.

## Roadmap

### 0.2 — Library experience

- Introduce a semantic design-token theme system built on CSS custom properties.
- Add a Settings → Interface tab with dark, light, and system appearance modes.
- Add user-selectable accent, heading, surface, text, and player colors with accessible defaults and reset controls.
- Add separate font selectors for interface text, headings, metadata, and player elements using bundled and system-safe choices.
- Support a custom application logo with validation, preview, reset, and a safe built-in fallback.
- Publish a documented theme definition format so contributors can add themes without rewriting components.
- Persist and restore interface preferences before first paint to avoid theme flashes.
- Allow playlists and the playback queue to mix tracks from multiple Subsonic servers, Bandcamp, and future local-library providers.
- Preserve each playlist item's source identity and report unavailable sources without discarding the rest of the playlist.
- Add full album-detail navigation and user-facing pagination while retaining the existing album sorting, filtering, provider attribution, and paged provider discovery.
- Add native operating-system media controls.
- Add favorites, scrobbling, play-queue restoration, and server capability detection.
- Improve loading, empty, offline, authentication-expired, and partial-failure states.

### 0.3 — Secure desktop application

- Embed or launch the Rust provider service from Tauri.
- Store credentials using Windows Credential Manager and equivalent vaults on other platforms.
- Add local folders as first-class library providers alongside remote connections.
- Index common local audio formats including WAV, MP3, FLAC, AAC/M4A, Ogg Vorbis, Opus, and other formats supported by the playback backend.
- Read local tags, embedded artwork, duration, codec, bitrate, disc/track numbers, and ReplayGain metadata without modifying source files.
- Watch configured folders for additions, changes, moves, and deletions, with manual rescan and clear permission/error reporting.
- Support multiple local folders and removable/network volumes while preserving stable provider-qualified track identities.
- Add native media keys, system media controls, notifications, tray behavior, and single-instance handling.
- Produce signed Windows installers and an update workflow.

### 0.4 — Persistence and offline support

- Persist provider metadata, preferences, queues, and playback position.
- Add encrypted credential migration and provider editing.
- Add artwork caching, configurable audio cache, downloads, and offline playback.
- Add per-provider transcoding and bandwidth preferences.
- Persist unified playlists and queues containing items from any combination of local and remote providers.

### Later

- Add more music-service provider adapters behind `MusicProvider`.
- Target macOS and Linux desktop packages.
- Add accessibility audits, localization, telemetry controls, and broader integration testing.

## Known limitations

- UI-added providers are memory-only and must be re-added after a backend restart.
- Server playlists are currently read-only; creating, editing, and persisting mixed-source playlists is not yet implemented.
- The desktop shell depends on a separately running backend today.
- The browser API base is currently fixed to `http://127.0.0.1:3000/api`.
- Provider registration accepts local-network URLs by design; the backend should not be exposed to untrusted networks in its current form.

See [CHANGELOG.md](CHANGELOG.md) for release history.
See [TODO.md](TODO.md) for the tracked implementation backlog and acceptance criteria.
See [LICENSING.md](LICENSING.md) for the intended distribution model, source-access policy, and third-party licensing checklist.
