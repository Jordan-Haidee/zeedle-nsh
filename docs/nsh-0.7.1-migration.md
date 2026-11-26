# NSH 0.7.1 migration

This project now builds against `nsh-core` 0.7.1 and CI installs the matching
0.7.1 CLI. NSH 0.7.1 is a patch release on 0.7.0; the compatibility review
therefore covers both releases since 0.6.1.

## Changes used by Zeedle

- The custom desktop header now uses `nsh/base`'s `TitleBar`. It owns window
  dragging, its minimum height, and dynamically measured insets around system
  controls. The app keeps its title, theme control, and platform-specific
  minimize, maximize, and close actions.
- Keyboard shortcuts remain `keymap.bind` commands guarded by `notEditing`.
  The 0.7 focus and key routing changes are covered by an app test that checks
  Space toggles playback from content and remains text in the search field.
- NSH 0.7.1 also fixes the Windows title-bar accent color leaking into client
  content, which applies to the app's hidden title bar.

## Breaking-change review

| NSH change since 0.6.1 | Effect here |
| --- | --- |
| Desktop windows close independently; default quit behavior is platform-specific. | The app handles `closePress` itself: close-to-tray hides the window, otherwise it calls `Application.quit()`. Tray Exit also quits explicitly, so the platform default does not decide the app's existing behavior. |
| Pointer presses no longer focus controls on macOS/iOS; raw keys with no focused view start at the window root and bubble. Escape routing also changed around focused content and popovers. | App shortcuts use `keymap`, not raw key handlers. Page navigation explicitly blurs search and focuses the content pane. The keyboard test covers playback and focused search behavior. |
| New fields were added to several public Rust structs, requiring `..Default::default()` in struct literals. | The app does not construct the affected NSH structs directly. `nsh check` and the Rust test suite validate the upgraded API use. |
| `Speech` moved to `nsh-extra`; clipboard writes and some storage/update APIs now report failures; `nsh-extra` versions follow NSH. | Zeedle does not use Speech, those clipboard/storage/update APIs, or `nsh-extra`; no migration was needed. |
| `nsh/base` parts are loaded on demand for web bundles; CLI module dependencies are locked in `nsh.lock`. | This is a native Rust-root app and does not install external NSH modules or target web; its own crate stays in the existing `rust` entry. |

The full upstream change list is in the [NSH 0.7.0 release notes](https://github.com/longbridge/nsh/releases/tag/v0.7.0)
and [NSH 0.7.1 release notes](https://github.com/longbridge/nsh/releases/tag/v0.7.1).
