# Zeedle

A Native Shell music player with a TSX interface and a Rust audio/tag backend. The five screens cover the music gallery, search, lyrics, settings and app information.

Interface icons are the Lucide SVG set (`0.468.0`, ISC license), kept in `assets/icons/` and tinted by the TSX components.

## Run

```sh
nsh run
nsh check
nsh package --target windows
```

The gallery scans the user's `Music` folder on first launch. Choose another directory from **Settings**. Supported files are MP3, FLAC, WAV and OGG.

Rust uses rodio for playback and Lofty for embedded tags, lyrics, album art and title/artist/album edits. The spectrum view is calculated with rustfft. TSX files live in `App.tsx`; the app's native module is in `src/lib.rs`.
