---
title: Project status
sidebar_position: 1
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Project status

## Current functionality

- Connect multiple Subsonic or OpenSubsonic servers from Settings → Connections.
- Authenticate with an OpenSubsonic API key or Subsonic salted password token.
- Fall back to legacy directory-based browsing for servers that omit modern ID3 browsing results.
- Verify new connections with `ping` before registering them.
- Query every connected provider concurrently as one unified library.
- Browse albums from all available providers without selecting a globally active connection.
- Retrieve successive provider album pages instead of limiting each collection to its first 30 entries.
- Filter and sort the dedicated Albums page by title, artist, year, and source.
- Search every connected library from the dedicated Search page.
- Load provider-qualified album tracks and play results from Home, Albums, or Search.
- Continue displaying successful sources when another provider is offline, unauthorized, invalid, or times out.
- Proxy cover artwork and audio without exposing credentials to the browser.
- Stream audio through the browser with byte-range support for seeking.
- Run the same Leptos UI in a browser or Tauri 2 desktop shell.

The Artists and Playlists pages remain placeholders. Home, Albums, and Search use the unified library service.

## Security and persistence

- Provider secrets are sent only to the local Rust backend and the configured music server.
- Passwords and API keys are not written to browser `localStorage`.
- Providers added from the UI are held in backend memory and survive browser refreshes.
- Restarting the backend clears UI-added providers; the optional environment provider is recreated at startup.
- TLS certificates are validated. A server using an untrusted self-signed certificate will be rejected.

Durable secrets should eventually be stored through the operating-system credential vault, not in browser storage or plaintext configuration.

## Known limitations

- UI-added providers are memory-only and must be re-added after a backend restart.
- Artists and Playlists remain placeholders; Home, Albums, and Search are connected to live unified-library data.
- The desktop shell depends on a separately running backend today.
- The browser API base is currently fixed to `http://127.0.0.1:3000/api`.
- Provider registration accepts local-network URLs by design; the backend should not be exposed to untrusted networks in its current form.

See [Changelog](../releases/changelog) for release history.
See [implementation tracker](./todo) for the tracked implementation backlog and acceptance criteria.
See [licensing guide](./licensing) for the intended distribution model, source-access policy, and third-party licensing checklist.
