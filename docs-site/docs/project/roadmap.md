---
title: Roadmap
sidebar_position: 2
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Roadmap

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
- In 0.1.4, build persistent queue controls with previous/next behavior, playback restoration, and proper playback state.
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
