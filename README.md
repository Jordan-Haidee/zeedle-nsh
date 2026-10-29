# Zeedle

A Native Shell music player with a TSX interface and a Rust audio/tag backend. The five screens cover the music gallery, search, lyrics, settings and app information.

Interface icons are the Lucide SVG set (`0.468.0`, ISC license), kept in `assets/icons/` and tinted by the TSX components.

## Run

```sh
nsh run
nsh check
powershell -File scripts/package-windows.ps1
```

The gallery scans the user's `Music` folder on first launch. Choose another directory from **Settings**. Supported files are MP3, FLAC, WAV and OGG.

The Windows packaging script runs `nsh package --target windows`, then builds the project's Windows launcher. This launcher keeps the window hidden until the initial library and UI are ready, centers it, and presents the prepared first frame. Use this script for release packages; the default NSH launcher displays its window immediately. Distribute the complete `.nsh/out/Zeedle` directory, including its `nsh` subdirectory, or `Zeedle.zip`.

Check native Windows rendering with `nsh test tests/windows-visual.test.tsx --target windows`. After packaging, `powershell -File scripts/check-windows-startup.ps1` launches its own instance, checks hidden startup and screen centering, saves a `PrintWindow` screenshot without raising the window, and closes that instance.

Rust uses rodio for playback and Lofty for embedded tags, lyrics, album art and title/artist/album edits. The spectrum view is calculated with rustfft. TSX files live in `App.tsx`; the app's native module is in `src/lib.rs`.
