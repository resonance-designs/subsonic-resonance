---
title: Resonance
sidebar_position: 1
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Resonance

![Static Badge](https://img.shields.io/badge/Version-0.1.1-orange)
![Static Badge](https://img.shields.io/badge/Latest_Release-v0.1.1-green)

## Current functionality

- Connect multiple Subsonic or OpenSubsonic servers from Settings → Connections.
- Authenticate with an OpenSubsonic API key or Subsonic salted password token.
- Fall back to legacy directory-based browsing for servers that omit modern ID3 browsing results.
- Verify new connections with `ping` before registering them.
- Query every connected provider concurrently as one unified library.
- Browse albums from all available providers without selecting a globally active connection.
- Load provider-qualified album tracks and search every connected library from Home.
- Continue displaying successful sources when another provider is offline, unauthorized, invalid, or times out.
- Proxy cover artwork and audio without exposing credentials to the browser.
- Stream audio through the browser with byte-range support for seeking.
- Run the same Leptos UI in a browser or Tauri 2 desktop shell.

The Albums, Artists, Playlists, and dedicated Search pages are placeholders. Unified album browsing and search currently operate from Home.
