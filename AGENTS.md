# Repository Guidelines

## Project Structure

`App.tsx` is the NSH entry point; `ui/` contains the TSX interface, UI components, icons and their SVG assets, translations, and view types. `App.test.tsx` covers app behavior. Native Rust audio and metadata code lives in `src/lib.rs`, `src/audio.rs`, and `src/library.rs`, with spectrum processing in `src/spectrum.rs`. `theme.json` defines theme tokens. Cross-platform UI tests are in `tests/`, platform validation scripts in `scripts/`, and CI workflows in `.github/workflows/`. Screenshots used by the README live in `docs/screenshots/`.

## Build, Test, and Package

- `nsh run` starts the app with hot reload.
- `nsh check -v` checks the TSX app and Rust crate; `nsh test App.test.tsx` runs app behavior tests.
- `bun run fmt` formats all TypeScript and TSX files with Oxfmt; `bun run fmt:check` checks formatting without changing files.
- `bun run lint` lints TypeScript and TSX files with Oxlint.
- The repository pre-commit hook checks formatting and lint for staged TypeScript/TSX files, then runs `nsh check`. Enable it once per clone with `git config core.hooksPath .githooks`.
- `bun run scripts/check-tailwind-lsp.ts` runs the Tailwind language server over all project TS/TSX using the repository-owned `tailwind-lsp.config.json` rules.
- `bun run scripts/test_config_persistence.ts` runs the isolated config-persistence integration test and its temporary WAV fixture.
- `cargo test --locked` runs Rust tests; `cargo fmt --check` checks Rust formatting.
- For GUI screenshots, run the app entry through the platform test host, for example `nsh test App.tsx --target windows`, and send `await app.screenshot("target/screen.png")` through stdin. This captures the app surface through NSH; do not capture the shared desktop. If an offscreen renderer is unavailable for a change, record the limitation instead of switching to an on-screen capture.
- `nsh package App.tsx --target windows` creates the Windows package and, with Inno Setup 6 installed, a `*-setup.exe` installer. Use `nsh package --target linux` or `nsh package --target macos` on the corresponding platform.

### Offscreen verification only

Use NSH's test host for all screenshots and UI checks in this repository. Run GUI behavior tests with `nsh test App.test.tsx`; run GUI screenshots with the app-entry form of `nsh test`, and save them under the ignored `target/` directory through `app.screenshot()`. For example:

```powershell
@'
await app.waitFor("id:screen-0");
await app.screenshot("target/gallery-dark.png");
await app.setColorScheme("light");
await app.screenshot("target/gallery-light.png");
'@ | nsh test App.tsx --target windows
```

This captures the app window through the test host; never capture the shared desktop or bring the app to the foreground. Do not use TUI output as a substitute for GUI appearance checks. If the offscreen test host cannot render a change, record the limitation rather than switching to an on-screen capture.

App tests use songs and lyric metadata from the default Music folder. The README lists the expected fixture tracks. Screenshot tests can update files under `docs/screenshots/`; review those image changes before including them.

## Coding Style

Use two spaces in TSX and JSON; run `cargo fmt` after Rust edits. `tsconfig.json` keeps unused locals/parameters, missing returns, and switch fallthrough as errors. Use PascalCase for React components and Rust types, and camelCase for functions and variables. Keep theme colors in `theme.json` and access them through NSH `token()` helpers. Keep app UI TSX/TS modules and UI-only SVG assets under `ui/`; map icons in `ui/icons.tsx` using kebab-case filenames.

## Tests and Reviews

Name UI tests for observable behavior and exercise interactions through `nsh/test` selectors. Cover light and dark appearance when changing theme behavior. `tests/windows-visual.test.tsx` and README screenshot tests need the documented local music fixtures; `tests/platform-render.test.tsx` uses an empty test library. Capture through the offscreen test host on each affected platform and theme; never capture the shared desktop.

## Commits and Pull Requests

Recent history uses concise Conventional Commit subjects, for example `fix(ui): ...`, `docs(readme): ...`, and `ci: ...`. Keep subjects imperative. Pull requests should summarize behavior changes, link relevant issues, list checks run, and include before/after screenshots for UI changes.
