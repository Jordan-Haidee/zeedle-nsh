# Repository Guidelines

## Project Structure & Module Organization

`App.tsx` contains the NSH interface, screen components, navigation, and player state. `App.test.tsx` holds UI behavior tests. `icons.tsx` maps icon names to the Lucide SVGs in `assets/icons/`; keep additions in that set. `assets/cover.svg` is the fallback artwork. The Rust audio and metadata bridge is in `src/lib.rs`, with spectrum processing in `src/spectrum.rs`. App metadata and theme tokens live in `nsh.json` and `theme.json`. Design notes and implementation plans are under `docs/superpowers/`.

## Build, Test, and Development Commands

- `nsh run` launches the app in a development host with hot reload.
- `nsh check -v` type-checks the TSX and checks/builds the Rust crate.
- `nsh test App.test.tsx` runs the navigation, metadata, and theme regression tests.
- `nsh package --target windows` creates the Windows application under `.nsh/out/`.
- `cargo fmt --check` verifies Rust formatting; run `cargo fmt` after Rust edits.

## Coding Style & Naming Conventions

Use two-space indentation in TSX and JSON. Name screen components and React-style components in PascalCase; use camelCase for functions, signals, and variables. Keep UI colors in `theme.json` and read them through NSH `token()` helpers so mounted screens respond to appearance changes. Preserve Lucide SVG paths and kebab-case asset filenames.

## Testing Guidelines

Name tests for user-visible behavior, for example `test("mounted settings restyle when the system appearance changes", ...)`. Exercise real NSH interactions through `nsh/test` selectors and verify both light and dark transitions when changing theme behavior. For visual changes, render and inspect every affected screen in both themes. Prefer GPUI offscreen or headless browser captures; do not force a desktop window to the top for screenshots.

## Commit & Pull Request Guidelines

Use Conventional Commit subjects, matching the history: `feat(ui): ...`, `fix(theme): ...`, or `docs: ...`. Keep subjects concise and imperative. Pull requests should summarize behavior changes, list checks run, link relevant issues, and include before/after screenshots for UI changes.
