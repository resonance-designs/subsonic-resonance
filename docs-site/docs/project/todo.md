---
title: Implementation tracker
sidebar_position: 3
---

<!-- Generated from TODO.md by scripts/sync-docs.js. Do not edit directly. -->

# Implementation tracker

This file tracks implementation work that is planned but not yet complete. Keep items narrowly scoped, mark them complete only after validation, and move release-worthy changes into `CHANGELOG.md`.

## 0.2 roadmap

### Library experience

- [x] Replace global active-provider state with an enabled-provider registry and unified-library query service.
- [x] Query enabled providers concurrently with per-provider timeouts, partial results, and source-specific error reporting.
- [x] Add optional source filters without hiding or disabling the other configured providers globally.
- [x] Introduce provider-qualified `MediaId` values for the implemented track, album, artist, playlist, artwork, route, and persistent playback-queue flows; favorites and caches will adopt them as those features are added.
- [x] Define deterministic merge, sort, and pagination behavior for current album and track results arriving from multiple providers; cross-provider metadata deduplication remains future work.
- [x] Update Home, Albums, Search, and player state to consume aggregated results while showing each item's source.
- [x] Update Artists to consume aggregated results after adding the required provider contract and API endpoints.
- [x] **0.1.4:** Add provider-qualified artist detail navigation with complete provider-reported releases and release-track browsing.
- [ ] Allow playlists and queues to contain provider-qualified tracks from different remote and local providers.
- [ ] Persist mixed-source playlists without embedding credentials or unstable stream URLs.
- [ ] Handle a missing, offline, removed, or unauthorized provider at item level so the rest of a mixed playlist remains usable.
- [ ] Add integration tests covering duplicate remote IDs, concurrent provider failures, mixed-source ordering, and partial playback availability.
- [ ] Implement a local-filesystem `MusicProvider` adapter.
- [ ] Add desktop folder selection, permission handling, multiple library roots, and removable/network-volume behavior.
- [ ] Index WAV, MP3, FLAC, AAC/M4A, Ogg Vorbis, and Opus, with an extensible format capability table.
- [ ] Extract tags, embedded artwork, duration, codec, bitrate, sample rate, disc/track numbers, and ReplayGain metadata.
- [ ] Define stable local track identities that survive metadata changes and file moves when possible.
- [ ] Add incremental filesystem watching, manual rescans, cancellation, progress reporting, and recovery after interrupted scans.
- [ ] Keep indexing read-only and document symlink, hidden-file, permission, and unsupported/corrupt-file behavior.
- [ ] Add local artwork and audio serving that prevents path traversal and exposes only indexed files.
- [ ] Test large libraries, Unicode paths, long Windows paths, duplicate files, cue sheets, multi-disc albums, and malformed tags.
- [x] Complete the first live Albums and dedicated Search page slice.
- [x] Complete the first read-only Playlists page using provider-qualified server playlists and tracks; mixed-source creation and persistence remain future work.
- [ ] Persist providers securely using the operating-system credential vault.
- [ ] Integrate the Rust backend lifecycle into the Tauri desktop application.
- [x] Add session playback modes for randomized order, repeat-current, and repeat-queue behavior.
- [x] **0.1.4:** Persist provider-qualified queue entries and the selected track, with previous, next, inspect, remove, and clear controls.
- [x] **0.1.4:** Restore in-track playback position plus volume, mute, shuffle, and repeat preferences without autoplaying after reload.
- [ ] Add native operating-system media controls.
- [ ] Add provider editing, offline caching, downloads, and per-provider transcoding settings.

### Theme and interface customization

#### 1. Theme foundation

- [ ] Inventory every hard-coded UI color, font, radius, shadow, spacing value, and component state in `crates/ui/base.css`, `crates/ui/components.css`, and `crates/ui/player.css`.
- [ ] Define semantic CSS custom properties instead of component-specific color names. Initial tokens should cover:
  - [ ] Page, sidebar, panel, elevated, input, and player surfaces.
  - [ ] Primary, secondary, muted, inverse, and disabled text.
  - [ ] Accent, accent-hover, focus, success, warning, and destructive states.
  - [ ] Heading, link, border, artwork-placeholder, and progress colors.
  - [ ] Interface, heading, metadata, and player font families.
  - [ ] Font sizes, weights, line heights, radii, shadows, and motion preferences where useful.
- [ ] Replace current `--ink`, `--cream`, `--amber`, and other presentation-oriented variables with semantic tokens or compatibility aliases.
- [ ] Ensure components consume tokens exclusively; theme switching must not require component markup changes.
- [ ] Add a serializable Rust `ThemeDefinition` model with a schema/version field.
- [ ] Define a stable developer-facing theme format and document required, optional, and fallback values.
- [ ] Reject unknown or invalid token values safely instead of injecting arbitrary CSS text.

#### 2. Built-in appearance modes

- [ ] Add built-in Dark and Light theme definitions with equivalent semantic coverage.
- [ ] Add a System mode that follows `prefers-color-scheme` and reacts when the operating-system preference changes.
- [ ] Set the document `color-scheme` and `theme-color` metadata to match the active theme.
- [ ] Apply the saved theme before the main UI paints to prevent a light/dark flash.
- [ ] Preserve the existing reduced-motion behavior across every theme.

#### 3. Settings → Interface

- [ ] Add an `Interface` tab beside General and Connections.
- [ ] Add appearance selection for System, Dark, and Light.
- [ ] Add accessible color controls for:
  - [ ] Accent color.
  - [ ] Heading color.
  - [ ] Primary and muted text colors.
  - [ ] Page, panel, and sidebar surfaces.
  - [ ] Player and progress colors.
- [ ] Pair native color inputs with editable hex values and visible labels.
- [ ] Validate and normalize colors before applying them.
- [ ] Provide a live preview that does not require saving or reloading.
- [ ] Provide per-field reset and Reset Theme controls.
- [ ] Warn when a chosen foreground/background combination fails the project contrast threshold.
- [ ] Make all controls keyboard accessible and usable at mobile widths.

#### 4. Typography customization

- [ ] Define curated, cross-platform font stacks that work without network access.
- [ ] Add independent selectors for interface/body, headings, metadata, and player typography.
- [ ] Include a readable default, a serif/editorial option, a geometric option, and a high-legibility option.
- [ ] Preview each font using realistic application text.
- [ ] Ensure font changes do not cause navigation truncation, player overflow, or layout shift at supported breakpoints.
- [ ] Document how developers can bundle and register another font while respecting its license.
- [ ] Do not fetch fonts from third-party CDNs by default.

#### 5. Custom logo

- [ ] Add a logo selector with preview, replace, and reset actions.
- [ ] Accept only supported local image types such as PNG, WebP, JPEG, and SVG after explicit sanitization.
- [ ] Enforce file-size and dimension limits and show actionable validation errors.
- [ ] Preserve aspect ratio and verify the logo in desktop sidebar, mobile header, light mode, and dark mode.
- [ ] Always retain the built-in Resonance mark as a fallback.
- [ ] Store browser logo data in IndexedDB rather than `localStorage`; use application-managed storage for desktop builds.
- [ ] Treat SVG as untrusted input and sanitize it or rasterize it before display.

#### 6. Persistence and import/export

- [ ] Persist theme preferences separately from provider credentials.
- [ ] Version stored preferences and define migrations for future token changes.
- [ ] Add Export Theme and Import Theme actions using the documented theme format.
- [ ] Exclude machine-specific paths, credentials, and unrelated application settings from exported themes.
- [ ] Validate imported themes before persisting or applying them.
- [ ] Recover automatically to the built-in theme when stored data is corrupt or incomplete.

#### 7. Developer customization

- [ ] Add a theme-authoring document with token reference, examples, screenshots, and validation commands.
- [ ] Provide a complete example third-party theme without duplicating component CSS.
- [ ] Decide whether contributed themes are compiled into the application, loaded from manifests, or both.
- [ ] Add a development-only theme inspector or token preview page.
- [ ] Document compatibility guarantees and deprecation rules for theme schema versions.

#### 8. Quality gates

- [ ] Add unit tests for theme serialization, validation, defaults, migration, and corrupt-data recovery.
- [ ] Add browser tests for switching modes, refreshing, System mode changes, reset behavior, and import/export.
- [ ] Add visual regression coverage for Dark, Light, and one customized theme at desktop and mobile sizes.
- [ ] Check WCAG contrast for default themes and surface warnings for invalid custom combinations.
- [ ] Confirm custom values cannot inject styles, URLs, markup, or scripts.
- [ ] Confirm provider switching, dialogs, artwork, errors, and native audio controls remain legible in every built-in theme.

#### Theme definition of done

- [ ] Users can select System, Dark, or Light and see the choice persist without a flash on reload.
- [ ] Users can customize documented color and typography roles from Settings → Interface.
- [ ] Users can preview, save, reset, export, and import safe theme settings.
- [ ] Users can set and remove a validated custom logo.
- [ ] Developers can add a compatible theme using documented tokens and a versioned theme definition.
- [ ] Built-in themes pass responsive, keyboard, reduced-motion, and contrast checks.
