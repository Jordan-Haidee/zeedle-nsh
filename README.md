<p align="center">
  <img src="assets/cover.svg" alt="Zeedle logo" width="88">
</p>

<h1 align="center">Zeedle</h1>

<p align="center">
  <strong>A Native Shell (NSH) port of the <a href="https://github.com/Jordan-Haidee/zeedle">original Zeedle music player</a>.</strong><br>
  Music at the speed of a blink, designed for pure simplicity.
</p>

<p align="center">
  Built with <a href="https://nsh.dev">Native Shell</a>, Rust and Rodio
</p>

---

Zeedle is a lightweight desktop music player focused on your local library. This project brings the original Zeedle experience to [Native Shell](https://nsh.dev), with a TSX interface and a Rust audio and metadata backend.

## Features

- Browse and search local MP3, FLAC, WAV and OGG files.
- Follow synchronized lyrics and seek by clicking a lyric line.
- View a live audio spectrum while music plays.
- Edit song metadata, album art and lyrics from the library.
- Choose playback order, repeat or shuffle; use keyboard shortcuts for playback and navigation.
- Switch between English, Chinese, Spanish, French, German and Russian, and follow the system theme.

## Build and run

Install the [NSH CLI](https://nsh.dev/docs/) and a Rust toolchain, then run:

```sh
nsh run
```

To check the app and run its tests:

```sh
nsh check -v
nsh test App.test.tsx
```

Playback integration tests use songs from the default Music folder, including **一眼万年**, **万疆**, **Deadman** and **Love Story**. Keep those tracks and their lyric metadata available when running the full suite; no copyrighted audio is included in this repository.

## Package for Windows

```powershell
powershell -File scripts/package-windows.ps1
```

The script builds the NSH bundle and Zeedle's Windows launcher. The launcher waits until the music library and first UI are ready, then opens centered with the Zeedle icon. Distribute the complete `.nsh/out/Zeedle` folder or `.nsh/out/Zeedle.zip`.

For Windows rendering and startup checks:

```sh
nsh test tests/windows-visual.test.tsx --target windows
powershell -File scripts/check-windows-startup.ps1
```

## Project structure

- `App.tsx` — screens, navigation and player controls.
- `src/lib.rs` and `src/spectrum.rs` — audio playback, metadata, browser links and spectrum analysis.
- `assets/icons/` — Lucide SVG interface icons.
- `theme.json` — light and dark theme tokens.

The original Zeedle project is maintained at [Jordan-Haidee/zeedle](https://github.com/Jordan-Haidee/zeedle). This repository is its NSH port; it is not the original Slint application.

## Screenshots

The following full-window captures show the Windows app in English. Search, lyrics and metadata editing are shown with “Love Story” by Taylor Swift.

### Music library

![Zeedle music library](docs/screenshots/gallery.png)

### Search

![Search results for Love Story](docs/screenshots/search.png)

### Synchronized lyrics

![Synchronized lyrics for Love Story](docs/screenshots/lyrics.png)

### Settings

![Zeedle settings](docs/screenshots/settings.png)

### About

![About Zeedle and NSH](docs/screenshots/about.png)

### Edit song metadata

![Edit metadata for Love Story](docs/screenshots/metadata-editor.png)
