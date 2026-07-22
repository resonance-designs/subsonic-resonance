# Changelog

All notable changes to Subsonic Resonance will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2026-07-21

### Added

- Added dedicated album-detail navigation with artwork, release metadata, provider attribution, and track browsing through the persistent player.
- Added 24-album user-facing pagination over the filtered and sorted unified library, backed by complete 500-item batch discovery across all available album pages.
- Added OpenSubsonic extension discovery with version-aware capability checks, conservative legacy and discovery-failure defaults, and capability details in Settings → Connections.
- Added a unified Favorites view for provider-qualified artists, albums, and tracks with partial-provider failure reporting.
- Added version-gated Subsonic `star`, `unstar`, and `getStarred2` support with optimistic UI controls and rollback on failed mutations.
- Added version-gated Subsonic scrobbling with one-shot now-playing reports, completed submissions after half a track or four minutes, seek-resistant listened-time accounting, and duplicate protection across pause, navigation, queue restoration, and repeat playback.
- Added a persistent global scrobbling preference under Settings → General and provider-level favorites/scrobbling availability in Connections.

### Changed

- Deferred artist-catalog discovery until a `getArtist` response lacks attribution or conflicts with its reported album count, avoiding an extra provider request on valid responses.
- Removed the application base stylesheet’s no-op empty data-URL import.
- Extended version synchronization to update the README release badges before regenerating the documentation-site introduction.
- Corrected the roadmap to distinguish completed persistent queue and provider-qualified media identity work from planned native media controls.
- Distinguished each provider’s server version from its supported Subsonic API version in provider summaries.
- Renamed every Rust package, dependency, import, executable reference, and launcher command to the full `subsonic-resonance-*` project namespace.
- Moved generated-document drift detection ahead of expensive release checks and made release failures identify the changed files and required remediation.

### Fixed

- Replaced undefined media-card, playlist-card, and library-search `--blue` references with the established cyan accent token.
- Marked aggregated library responses incomplete when a provider task is cancelled or panics, while preserving successful items and provider-specific issues.

## [0.1.4] - 2026-07-19

### Added

- Added provider-qualified artist detail pages that load an artist’s complete provider-reported release list, release tracks, and shared-queue playback without interrupting the active stream during browsing.
- Added a versioned browser-side playback queue that restores provider-qualified tracks and the selected position without storing credentials or stream URLs.
- Added previous, next, queue inspection, track removal, and clear-queue controls to the shared player.
- Added a unified custom playback frame containing play/pause, seek progress, elapsed and duration times, mute, volume, previous/next, shuffle, repeat, and queue controls.
- Added versioned restoration for in-track position, volume, mute, shuffle, and repeat preferences, with periodic, pause, and page-exit checkpoints and no automatic playback after reload.

### Changed

- Replaced the player’s text-based shuffle, repeat, and volume controls with state-aware, accessible icons.
- Removed an unnecessary playlist-entry clone when converting OpenSubsonic playlist details into provider-neutral tracks.
- Consolidated the unified Artists and Playlists concurrency, timeout, qualification, sorting, and partial-failure flow into a shared provider aggregation helper.
- Consolidated album, artist, and playlist presentation into a shared media-card style while preserving their distinct artwork shapes and aspect ratios.
- Preserved restored queues across provider discovery, empty-provider states, and unified library refreshes; the first album now initializes playback only when no saved queue exists.
- Replaced the legacy base/override stylesheet pair with scoped `base.css`, `components.css`, and `player.css` bundles and removed overlapping `styles.css`/`live.css` ownership.
- Moved the canonical application images into `crates/ui/img` and updated both the UI build and documentation sync pipelines to consume that single source of truth.
- Exposed the bundled custom font family and its normal, bold, and bold-italic faces to both the application and documentation stylesheets without changing the current typography defaults.

### Fixed

- Added an album-catalog compatibility fallback for Subsonic servers that list artists but omit, reject, or return an invalid `getArtist` response.
- Scoped successful artist-detail responses by artist ID or name so providers cannot populate an artist page with unrelated collection albums.
- Validated artist-detail results against the stable artist-list identity and album count, rebuilding inconsistent or unattributed Bandcamp responses from the provider catalog.
- Unified the Artists, Playlists, Search, and Settings page headings—and the Settings section heading—with the established editorial typography used throughout Home and Albums.
- Added the standard external-link indicator and safe new-tab relationship attributes to the documentation footer's Resonance Designs link.
- Corrected invalid color and calculated-height syntax in the component stylesheet.
- Clamped provider-scoped artist request limits to the supported 1–500 range already enforced by the unified artist endpoint.
- Removed the Subsonic artist result cap that prevented unified offset pagination from reaching artists beyond the first 500 entries.
- Added request-generation guards to Artists and Playlists so stale provider-triggered responses cannot overwrite newer data, errors, or loading state.
- Stopped shuffled playback at the final queue item when repeat is disabled while retaining randomized wrapping for repeat-all mode.
- Decoupled user pause intent from transient media-element pause events so switching application views no longer terminates active playback.
- Separated browsed album, playlist, and Home search results from the persistent playback queue so selecting other library content no longer changes the queue index or interrupts the current track.
- Restored immediate track playback by handing explicit track selections directly to the persistent audio engine within the initiating user gesture.
- Prevented reactive queue updates from reassigning the active audio source and repeatedly resetting playback to the beginning.

## [0.1.3] - 2026-07-19

### Added

- Added provider-neutral artist retrieval, OpenSubsonic `getArtists` decoding, an album-derived compatibility fallback, and provider-qualified unified artist API responses with timeout and partial-failure handling.
- Added a live unified Artists page with filtering, album-count sorting, provider attribution, responsive artwork cards, and loading, empty, and error states.
- Added provider-neutral playlist summaries and details, OpenSubsonic `getPlaylists`/`getPlaylist` decoding, and provider-qualified unified playlist API responses with timeout and partial-failure handling.
- Added a live read-only Playlists page with filtering, source attribution, playlist artwork and metadata, track loading, and shared-queue playback.
- Added provider-ID source filters to Albums, Artists, Playlists, and Search so users can narrow a view without disabling any configured connection.
- Added session playback controls for randomized queue order, repeating the current track, and repeating the complete album, playlist, or search queue.

### Changed

- Replaced the workspace's `MIT OR Apache-2.0` declaration with an inherited proprietary `LICENSE.md` covering no-charge official binary use and separately contracted source access.
- Increased Subsonic album discovery requests from 30 to 500 items per page, matching the API maximum while preserving bounded pagination, album-ID deduplication, and repeated-page termination safeguards.
- Replaced documentation hero logo scaling based on raw asset dimensions with an explicit 305-pixel display size while retaining configured light and dark logo source fallbacks.
- Removed the documentation hero wordmark's fixed pixel-size override so its dedicated responsive `rem`/viewport `clamp()` sizing remains independent from the navbar branding.
- Reorganized the implementation tracker so theme customization and library-experience progress appear together under the 0.2 roadmap.

### Fixed

- Reused the shared album-track loader during unified library refreshes and reset the current track index whenever the selected album's tracks are replaced or cleared.
- Explicitly configured Trunk to emit shared UI image assets beneath `/img`, keeping copied logos compatible with the application navigation paths.
- Prevented overlapping album-track and search requests from allowing stale responses to overwrite newer UI state.

## [0.1.2] - 2026-07-19

### Added

- Added a dedicated unified Albums page with title, artist, and source filtering; title, artist, year, and source sorting; provider attribution; album selection; and shared playback.
- Added a dedicated unified Search page that queries every connected provider, reports partial provider failures, preserves provider-qualified results, and promotes selected results into the shared playback queue.
- Added responsive album grids, library toolbars, search states, empty results, selection states, and accessible form labels for the new library pages.
- Added a responsive two-column documentation landing-page hero with the Subsonic Resonance identity, primary documentation actions, and an application Albums-library preview.
- Added a global documentation footer linking project documentation, development resources, releases, and licensing guidance.

### Changed

- Changed the Albums and Search navigation destinations from placeholders to live unified-library experiences.
- Reused the shared player across Home, Albums, and Search while retaining the source identity required for credential-safe streaming.
- Replaced the application and documentation placeholder marks with the shared Subsonic Resonance logo, matched their two-line Subsonic/Resonance brand lockups, and aligned the documentation palette, Resonance wordmark, and primary navigation accents with the application colors.
- Disabled the documentation light/dark and template-theme selectors so the site consistently presents the application-aligned Resonance theme.
- Updated the documentation homepage and global navigation/footer styling for responsive desktop and mobile layouts, semantic heading structure, visible focus handling, and accessible image descriptions.

### Fixed

- Fixed album discovery stopping after the first 30-item request by paging Subsonic and Bandcamp-compatible album-list endpoints, detecting repeated pages from servers that ignore offsets, and loading the complete supported unified batch in the UI.

### Quality

- Verified workspace formatting, compilation, and tests; the WebAssembly UI build; the production documentation build; documentation synchronization; and Git whitespace checks.

## [0.1.1] - 2026-07-18

### Added

- Added stable UUID identifiers for newly registered providers.
- Added provider-qualified `MediaId` values that keep provider, media kind, and native item ID separate for albums, tracks, artists, artwork, and future playlists.
- Added concurrent unified-library endpoints for albums, album details, and track search across every connected provider.
- Added deterministic merged ordering for unified album and search results.
- Added per-provider query timeouts and structured partial-failure responses containing the affected provider, failure category, message, and retryability.
- Added source attribution to unified albums, tracks, search results, and now-playing information.
- Added core tests confirming that identical native IDs from different providers or media kinds cannot collide and that qualified IDs retain their structure through JSON serialization.
- Added a Docusaurus documentation application under `docs-site`, based on the Resonance Designs Docusaurus template.
- Added generated documentation pages for installation, architecture, project status, roadmap, implementation tracking, licensing, and release history.
- Added root Node commands for synchronizing, building, serving, and quality-checking the documentation site.
- Added Lighthouse commands for auditing the browser application and documentation site in offline deterministic or optional Codex AI-authored modes, with timestamped JSON reports, automatically generated Markdown accessibility assessments, a reusable WCAG 2.2 AA assessment template, validated AI output, explicit custom target labels, and a configurable minimum accessibility score.
- Added version synchronization across Cargo workspace metadata, the explicit UI crate version, Node packages and lockfile, documentation configuration, API metadata, README, Cargo lockfile, and generated release documentation.
- Added a guarded Git release command with version-consistency checks, build/test verification, collision detection, annotated tags, atomic branch/tag publishing, and a non-mutating dry-run mode.
- Added an `npm run build:interactive:win` PowerShell launcher that compiles the Rust workspace and offers to start either the complete browser application or only the API server.
- Added an equivalent `npm run build:interactive:unix` Bash launcher for Linux and macOS.

### Changed

- Replaced the globally active-provider UI model with a unified library in which every configured provider participates automatically.
- Changed Home, album loading, artwork retrieval, search, and audio streaming to route through provider-qualified media identities.
- Removed the Settings “Use” action and active-provider sidebar switching.
- Changed adding or removing a provider to refresh the complete unified library automatically.
- Preserved existing provider-scoped endpoints while unified-library consumers are migrated incrementally.
- Changed newly registered provider IDs from process-local `provider-N` counters to UUIDs in preparation for durable provider metadata.
- Adapted the Docusaurus template identity, repository links, feature flags, landing page, navigation, and version display for Resonance.
- Made `README.md`, `TODO.md`, `CHANGELOG.md`, and `LICENSING.md` the canonical sources for generated documentation pages.
- Configured documentation builds to fail on broken links and excluded the template's sample documentation from published routes.
- Corrected the root Cargo build command to compile the complete workspace and separated backend startup into its own command.
- Added separate UI and combined application startup commands, standardized the browser UI on port 8088, and moved the documentation server to port 3001 to avoid the API on port 3000.

### Fixed

- Added interactive pre-build and startup port/process checks with process IDs and opt-in termination, including detection of Windows executables that lock Cargo build artifacts, and disabled Trunk address lookup to prevent confusing local hostname aliases and duplicate-listener failures.
- Added fallback from ID3-based `getAlbumList2`/`getAlbum` calls to legacy `getAlbumList`/`getMusicDirectory` responses for partial Subsonic implementations such as Bandcamp's open beta.
- Made Subsonic response decoding tolerant of numeric IDs, string-encoded numeric metadata, and negative “unknown” sentinels used by Bandcamp, and improved schema errors so the failing endpoint and field are reported precisely.

- Prevented one unavailable, unauthorized, invalid, or timed-out provider from hiding successful library results returned by other providers.

### Security

- Kept provider credentials and authenticated upstream stream URLs out of qualified media identities and aggregate API responses.

### Documentation

- Added the planned theme and interface-customization system to the `0.2` roadmap.
- Added a tracked `TODO.md` covering semantic theme tokens, dark/light/system modes, user colors, typography, custom logos, persistence, import/export, developer theming, security, accessibility, and quality gates.
- Expanded the roadmap and tracked backlog for unified concurrent access to all providers, provider-qualified media identities, mixed-source playlists and queues, and local audio libraries covering WAV, MP3, FLAC, AAC/M4A, Ogg Vorbis, and Opus.
- Added `LICENSING.md` documenting the intended proprietary-freeware distribution model, paid confidential source access, contribution ownership, dependency notices, and release compliance checklist.

### Quality

- Verified the production documentation build, ESLint, TypeScript application checks, pre-build script checks, and documentation synchronization.
- Recorded a Lighthouse documentation result of 99 accessibility, 100 best practices, and 100 SEO.

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
