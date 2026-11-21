# NSH 0.6.1 UI and backend refactor

## Requirements
- Preserve Zeedle's existing visual output, theme behavior, navigation, shortcuts, playback flow, library actions, settings, persistence, and error handling.
- Fix progress seeking so click and drag update the controlled slider, with audio seek committed on `onChangeEnd`.
- Make UI and Rust responsibilities readable across focused modules while keeping `zeedle-nsh/native` exports and behavior stable.
- Use APIs available in the checked-in NSH 0.6.1 declarations; avoid adopting newer APIs.

## Work sequence
1. Update the progress drag regression test first and confirm it fails against the existing press-only progress view.
2. Replace the custom progress view with `nsh/base` Slider, preserving current track, fill, thumb, label, value and click seeking; update the drag test to pass.
3. Extract TSX domain types/formatters and reusable player/row components into focused modules without changing their props' behavior or styling.
4. Split Rust audio state/worker and music metadata operations from the native module facade; retain the same registry and TypeScript API.
5. Run formatting, crate checks/tests, NSH TSX check/build/tests and review light/dark screenshots and interaction behavior on available targets.

## Verification
- The regression test must fail before the Slider implementation and pass after it.
- Existing app tests, Rust tests, formatting and NSH checks must pass.
- Compare screenshots and observable flows against the unchanged baseline, including light/dark, progress click/drag, navigation, settings, lyrics and song actions. Report any unavailable visual/platform verification explicitly.
