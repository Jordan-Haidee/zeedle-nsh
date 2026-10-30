# Rust UI for Zeedle

## Goal

Add a Rust implementation of Zeedle's NSH interface and make it the default app entry point. Keep the existing TSX interface runnable as a comparison and fallback. Both interfaces use the existing `Music` audio and metadata implementation in `src/lib.rs`, so playback behavior and device state remain shared by the same backend.

## User experience and scope

The Rust UI must reproduce the TSX app's window geometry, navigation, visual styling, responsive states, and user-visible behavior across the Library, Search, Lyrics, Settings, and About pages. This includes the metadata editor and delete confirmation overlays, playback controls, keyboard shortcuts, theme selection/system following, settings persistence, language selection, song sorting/search, lyrics seeking and synchronization, spectrum display, directory selection, metadata editing, and deleting tracks. Component spacing, colors, typography, and icon appearance follow `App.tsx` and `theme.json`.

`nsh run` and packaging the app directory mount the Rust root. Running `nsh run App.tsx` continues to launch the TSX implementation. Both entry points remain in the same app crate and use the same persisted settings keys and `Music` backend.

## Architecture

- Add Rust UI modules under `src/ui/`, with an `App` root component and focused page/component modules where useful.
- Set `"root": "ui::App"` in `nsh.json`; keep the existing Rust crate and native module registration. Keep `App.tsx` as the TSX entry and do not duplicate the audio engine.
- The Rust view calls the existing public `Music` operations from `src/lib.rs` for scanning, playback, seeking, volume, spectrum state, lyrics, covers, file pickers, metadata updates, and deletion. The current `OnceLock` backend remains the sole playback state and audio engine.
- Use NSH Rust components/signals, timers, keyboard bindings, theme accessors, and desktop window services. Reuse `theme.json`, the existing SVG assets, and the app's current window/sidebar/player dimensions.
- Share persisted preference key names with TSX so either entry point sees the same directory, theme, playback mode, spectrum, volume, and language settings.

## Behavior and error handling

Rust UI state mirrors the TSX view state while treating `Music` as the authority for audio playback. Poll `Music::state()` on the same cadence needed for progress, spectrum, end-of-track handling, and lyrics synchronization. Route returned errors into the existing in-app error presentation. Preserve current empty-library and empty-search states, history-based previous playback, repeat/random modes, and metadata validation.

The UI must preserve English, Simplified Chinese, Spanish, French, German, and Russian labels. Theme selection follows the system by default, supports explicit light/dark overrides, and updates mounted content reactively.

## Verification

- Run `nsh check -v` and `cargo fmt --check` after implementation.
- Render all five pages and the metadata/delete overlays in light and dark appearances. Compare the Rust renders with the TSX screenshots and `docs/screenshots/` to verify page layout, spacing, colors, and controls.
- Confirm `nsh run` selects the Rust root and `nsh run App.tsx` still starts the TSX interface.

## Out of scope

Do not replace, fork, or redesign the Rodio audio engine, metadata pipeline, or platform startup code. Do not remove the TSX interface.
