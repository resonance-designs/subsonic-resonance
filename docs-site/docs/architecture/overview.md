---
title: Architecture
sidebar_position: 1
---

<!-- Generated from README.md by scripts/sync-docs.js. Do not edit directly. -->

# Architecture

| Crate                                  | Responsibility                                                                  |
| -------------------------------------- | ------------------------------------------------------------------------------- |
| `subsonic-resonance-core`              | Provider-neutral domain models and the `MusicProvider` contract.                |
| `subsonic-resonance-provider-subsonic` | OpenSubsonic authentication, browsing, search, artwork, and stream URL support. |
| `subsonic-resonance-server`            | Axum provider registry, browser API, and credential-aware media proxy.          |
| `subsonic-resonance-ui`                | Shared Leptos CSR interface compiled to WebAssembly.                            |
| `subsonic-resonance-desktop`           | Tauri 2 Windows/desktop shell.                                                  |

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
