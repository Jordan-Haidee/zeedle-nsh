# Repository Guidelines

## Project Structure

`App.tsx` contains the NSH interface and player state; `App.test.tsx` covers app behavior. Native Rust audio and metadata code lives in `src/lib.rs`, with spectrum processing in `src/spectrum.rs` and Windows startup code in `src/startup.rs`. `icons.tsx` maps names to SVGs under `assets/icons/`; `theme.json` defines theme tokens. Cross-platform UI tests are in `tests/`, packaging and startup scripts in `scripts/`, and CI workflows in `.github/workflows/`. Screenshots used by the README live in `docs/screenshots/`.

## Build, Test, and Package

- `nsh run` starts the app with hot reload.
- `nsh check -v` checks the TSX app and Rust crate; `nsh test App.test.tsx` runs app behavior tests.
- `cargo test --locked` runs Rust tests; `cargo fmt --check` checks Rust formatting.
- `nsh test tests/platform-render.test.tsx --target windows` renders core screens on Windows. Replace the target with `linux` or `macos` for those hosts.
- `powershell -File scripts/package-windows.ps1` creates the Windows portable package. Use `nsh package --target linux` or `nsh package --target macos` on the corresponding platform.

App tests use songs and lyric metadata from the default Music folder. The README lists the expected fixture tracks. Screenshot tests can update files under `docs/screenshots/`; review those image changes before including them.

## Coding Style

Use two spaces in TSX and JSON; run `cargo fmt` after Rust edits. Use PascalCase for React components and Rust types, and camelCase for functions and variables. Keep theme colors in `theme.json` and access them through NSH `token()` helpers. Add icons through `icons.tsx` using the existing Lucide SVG assets and kebab-case filenames.

## Tests and Reviews

Name UI tests for observable behavior and exercise interactions through `nsh/test` selectors. Cover light and dark appearance when changing theme behavior. `tests/windows-visual.test.tsx` and README screenshot tests need the documented local music fixtures; `tests/platform-render.test.tsx` uses an empty test library. Inspect visual changes on each affected platform and theme.

## Commits and Pull Requests

Recent history uses concise Conventional Commit subjects, for example `fix(ui): ...`, `docs(readme): ...`, and `ci: ...`. Keep subjects imperative. Pull requests should summarize behavior changes, link relevant issues, list checks run, and include before/after screenshots for UI changes.
