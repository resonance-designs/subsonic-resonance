---
title: Resonance
sidebar_position: 1
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Resonance

![Static Badge](https://img.shields.io/badge/Version-0.1.5-orange)
![Static Badge](https://img.shields.io/badge/Latest_Release-v0.1.5-green)

## Current functionality

- Connect multiple Subsonic or OpenSubsonic servers from Settings → Connections.
- Authenticate with an OpenSubsonic API key or Subsonic salted password token.
- Fall back to legacy directory-based browsing for servers that omit modern ID3 browsing results.
- Verify new connections with `ping` before registering them.
- Discover versioned OpenSubsonic extensions and conservatively gate optional features when capability support is unknown.
- Aggregate starred artists, albums, and tracks across compatible providers in a dedicated Favorites view.
- Add or remove provider-qualified favorites from artist details, album details, track lists, and the Favorites view with optimistic rollback on failure.
- Report provider-qualified now-playing and completed scrobbles without interrupting playback, with a persistent global opt-out in Settings.
- Query every connected provider concurrently as one unified library.
- Browse albums from all available providers in a paginated grid without selecting a globally active connection.
- Retrieve successive provider album pages instead of limiting each collection to its first 30 entries.
- Open dedicated provider-qualified album pages with release metadata, artwork, and playable track lists.
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
