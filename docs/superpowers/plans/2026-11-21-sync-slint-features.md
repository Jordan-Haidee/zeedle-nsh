# Sync Recent Slint Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use native execution. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Defer volume changes until slider release and synchronize recent Slint tray and adjacent-LRC behavior into NSH.

**Architecture:** Use the existing `nsh/base` Slider for volume preview and release commit. Keep sidecar lyric lookup in the native library layer. Configure the app-owned desktop tray from the TSX root using NSH desktop APIs.

**Tech Stack:** NSH 0.6.1 TSX, Rust, rodio, lofty.

**Spec:** User request and recent functional commits in `D:\rs-projects\zeedle-slint` (`09b762e`, `19ad2d9`, `5d65566`).

## Global Constraints

- Preserve the existing NSH app architecture, theme tokens, and compact desktop design.
- Test UI through the NSH test host and capture screenshots offscreen only.
- Keep lyric loading in Rust and use same-stem `.lrc` files only as fallback.

## Review Focus

- Volume must not reach the audio backend while the pointer is held; release submits the final value once.
- Embedded lyrics keep precedence; unreadable or absent sidecars return no lyrics.
- Tray icon click restores the window; menu actions show, rescan, and exit.
- Tray unavailability must not prevent normal app startup.

### Task 1: Release-committed volume slider

**Files:** `ui/components/PlayerControls.tsx`, `App.test.tsx`, `docs/DESIGN.md`.

- [x] Add an integration test that holds the volume slider, verifies backend volume is unchanged, then drops and verifies it changes.
- [x] Run the test and confirm the expected failure.
- [x] Replace click-only level setting with a vertical Slider preview and one `onChangeEnd` commit.
- [x] Run the focused test and full UI suite.
- [x] Update the player motion inventory and inspect light/dark screenshots.

### Task 2: Sidecar LRC fallback

**Files:** `src/library.rs` and its tests.

- [x] Add a test that writes a minimal WAV and adjacent `.lrc`, then verifies parsed lyric text and timestamp.
- [x] Run it and confirm the expected failure.
- [x] Parse embedded lyrics first, then the same-stem sidecar with existing LRC parser.
- [x] Run Rust tests and formatting checks.

### Task 3: Desktop tray

**Files:** `ui/AppView.tsx`, `App.test.tsx`, `docs/DESIGN.md`.

- [x] Add app-level coverage for tray setup/actions where the test host exposes tray state.
- [x] Configure show, rescan, exit menu actions and primary-click window restore via `nsh/desktop`.
- [x] Verify build, tests, and offscreen screenshots in light and dark.

### Task 4: Keep the app alive when the window closes

**Files:** `ui/AppView.tsx`, `App.test.tsx`, `README.md`, `docs/DESIGN.md`.

- [x] Test native and custom window close behavior, including preserved app state.
- [x] Set `closeOnPress: false`, hide on native `closePress`, and hide from the custom title bar close button.
- [x] Keep the tray Exit item on NSH's native `quit` role and verify all UI tests.
