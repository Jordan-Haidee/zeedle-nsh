# Zeedle interface notes

Zeedle keeps its existing compact desktop layout, theme tokens, typography, spacing, icon family and interaction states. `theme.json` remains the source for app colors. UI code and UI-only SVGs live under `ui/`: `AppView.tsx` composes screens, `components/` contains shared controls, `icons.tsx` maps `icons/`, and `translations.ts`, `types.ts` and `format.ts` hold shared UI resources.

## Player panel motion inventory

| Change | Motion and reduced-motion behavior |
| --- | --- |
| Transport control hover and press | Existing 120 ms color transition; `motion-reduce:transition-none` makes it immediate. |
| Play / pause icon | Existing opacity swap; immediate in both motion settings. |
| Repeat / shuffle selection | Existing icon color and glyph update; immediate in both motion settings. |
| Volume hover and level | Hover reveals the vertical slider; its preview follows the pointer while dragging and commits once on release. No added transition, so reduced motion has no effect. |
| Playback progress drag | The NSH 0.6.1 `Slider` thumb and fill follow the pointer during drag. Release commits one seek through `onChangeEnd`; no extra transition, so reduced motion has no effect. |
| Playback time and spectrum | Follow the audio state continuously; no added easing or per-frame UI animation. |
| Tray click and show-window action | The native window restores and focuses immediately; the operating system owns any window transition. |
| Window close request | Closing hides the window and leaves playback and UI state alive; the tray Exit item quits the app. |

The player now previews volume changes while dragging and commits them on release. The tray uses native window and menu behavior without introducing additional visual controls into the player layout.
