# Changelog

All notable changes to Subsonic Resonance will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Added fallback from ID3-based `getAlbumList2`/`getAlbum` calls to legacy `getAlbumList`/`getMusicDirectory` responses for partial Subsonic implementations such as Bandcamp's open beta.
- Made Subsonic response decoding tolerant of numeric IDs, string-encoded numeric metadata, and negative “unknown” sentinels used by Bandcamp, and improved schema errors so the failing endpoint and field are reported precisely.

### Documentation

- Added the planned theme and interface-customization system to the `0.2` roadmap.
- Added a tracked `TODO.md` covering semantic theme tokens, dark/light/system modes, user colors, typography, custom logos, persistence, import/export, developer theming, security, accessibility, and quality gates.
- Expanded the roadmap and tracked backlog for unified concurrent access to all providers, provider-qualified media identities, mixed-source playlists and queues, and local audio libraries covering WAV, MP3, FLAC, AAC/M4A, Ogg Vorbis, and Opus.

## [0.1.0] - 2026-07-18

### Added

- Rust workspace with provider-neutral core, OpenSubsonic adapter, Axum backend, Leptos UI, and Tauri desktop crates.
- Provider-neutral `MusicProvider` trait and shared artist, album, track, album-detail, status, and error models.
- OpenSubsonic/Subsonic client supporting:
  - API-key authentication.
  - Salted MD5 password-token authentication without transmitting the plaintext password in request URLs.
  - Connection verification through `ping`.
  - Newest-album retrieval through `getAlbumList2`.
  - Album and track retrieval through `getAlbum`.
  - Track search through `search3`.
  - Cover-art and audio-stream URL generation.
  - Connection and request timeouts.
- In-memory multi-provider registry in the Axum backend.
- REST endpoints to register, list, inspect, remove, and query individual providers.
- Provider-scoped endpoints for albums, album tracks, search, cover art, and audio streams.
- Credential-aware artwork and audio proxying that keeps provider secrets out of the browser.
- HTTP byte-range forwarding for stream seeking, including content-range and accept-range response headers.
- Optional environment-configured default provider.
- Browser interface built with Leptos CSR and WebAssembly.
- Responsive night-drive hi-fi visual system for desktop, tablet, and mobile layouts.
- Settings area with General and Connections tabs.
- Add Provider dialog with name, URL, username, password/API-key selection, validation, live connection verification, and visible errors.
- Provider source list and active-provider switching in the sidebar.
- Provider removal with active-provider fallback selection.
- Live Home library backed by the selected server, including recent albums, album tracks, search, artwork, and audio playback.
- Loading, empty-library, backend-unavailable, and provider-error states.
- Tauri 2 Windows desktop shell scaffold and Windows application icon.
- Keyboard-accessible dialogs and settings navigation, responsive provider management, visible focus states, and reduced-motion support.
- Authentication and OpenSubsonic response-envelope unit tests.
- Project setup, configuration, security, development, and roadmap documentation.

### Changed

- Replaced hard-coded `sonora.local` connection presentation with a dynamic provider registry.
- Replaced mock Home albums and tracks with live OpenSubsonic responses.
- Changed album discovery from recently played albums to newest albums so libraries populate without playback history.
- Made environment provider configuration optional; the backend can start with no providers.
- Added default INFO-level application logging when `RUST_LOG` is not set.
- Updated the player to use provider-scoped real audio streams.

### Security

- Provider secrets are retained in backend memory rather than browser storage.
- New providers are verified before insertion into the registry.
- API keys do not include a username in OpenSubsonic requests.
- Password authentication generates a new random salt and token for each request.
- TLS certificate validation remains enabled.

### Known limitations

- Providers added through the UI are lost when the backend restarts.
- The desktop shell does not yet manage the backend lifecycle or operating-system credential storage.
- Albums, Artists, Playlists, and dedicated Search pages remain placeholders.
- The browser UI currently assumes the backend is available at `http://127.0.0.1:3000/api`.
- Native media controls, queue persistence, offline caching, downloads, and additional provider APIs are not implemented.
