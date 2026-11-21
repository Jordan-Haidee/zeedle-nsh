# Design Notes

## UI motion inventory

| Change | Motion | Reduced motion |
| --- | --- | --- |
| Main content surface appears below the title bar | None; its radius is static geometry copied from the original 10 pt corner | No change |
| Lyric row hover | Quiet `song-hover` wash, native hover state, no layout movement | No change |
| Lyrics and album metadata change with the current song | None; retain the original immediate content update | No change |
| Lyric typography and cover sizing | None; static styles follow the original scale | No change |
| Seek changes the active lyric | Immediate selection of the single latest timestamp at or before playback position | No change |
| Lyrics pane narrows | Artwork scales down to preserve its 16px side inset; lyric text keeps its 12px inset | No change |
| Search field loses focus on page change or blank-area press | Immediate focus handoff; no motion | No change |
| Gallery song row click | No pressed background; the row only has its existing pointer hover wash | No change |

## Keyboard behavior

- Sidebar navigation and its expand control are pointer-only; they do not take keyboard focus or bind Enter.
- App shortcuts are suppressed while a text input has focus. Leaving Search through the sidebar or clicking its blank content returns focus to the main content.
