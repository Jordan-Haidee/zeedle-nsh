# NSH theme and Slint visual alignment

## Problem

The supplied screenshot shows the title bar and page background in light mode while the settings cards and section labels retain dark-mode colors. In `App.tsx`, `SettingsPage` hoists `cardStyle`, `sectionStyle`, and the divider element into values computed once when the page mounts. They therefore keep the previous appearance after the app changes `colorScheme`. The “Follow system theme” off branch also stores the current mode without changing the app appearance from `system` to a fixed light or dark scheme.

The current UI also uses a hand-maintained set of dark colors and hard-coded light-mode values, while the original Slint UI defines one palette for each appearance. Some page dimensions and Lucide glyph sizes differ from the Slint references.

## Design

Use NSH’s theme tokens as the sole source for app colors. Add an app theme file referenced by `nsh.json`, with light and dark token values mapped from the original Slint palette and a small set of Zeedle-specific tokens for page, panel, raised surface, hover, pressed, secondary text, and spectrum/accent colors. Read these tokens with NSH’s reactive `token()` accessor in the view bindings so a scheme change updates mounted pages and dialogs.

Keep `colorScheme()` / `setColorScheme()` as the appearance source. Manual changes set an explicit light or dark appearance and persist it. With “Follow system theme” enabled, set the appearance to `system` and remove the saved override; when disabled, keep the current appearance explicit and persist it. Match the original Slint default of following the system when no user preference exists. Disable the title-bar theme button while system-following is active, as Slint does.

Align the five reference states against `D:/rs-projects/zeedle-slint/assets/p1.png` through `p5.png`, and match the About page to `app.slint` / `get_about_info`: a 120-point cover and the source description, author, version, and Github link. Use the Windows font `Microsoft YaHei UI`, preserve the shared window/sidebar/player geometry, and lay out song columns at 33% each with text starting 40% into each column. Use 24-point Settings content padding, a 20-point inter-card gap, 16-point card padding, and a 6-point directory label/input group; a 10-point NSH card gap gives the same rendered bounds as Slint's 12-point spacing. Restore the 40-point search field without a leading search glyph and place its 24-point input underline inside the field; use the 44-point empty-search icon and 14-point empty-search text; and match the metadata editor's 520×450-point bounds without dimming its backdrop. Use the same Lucide glyphs and sizes as the Slint components: 18-point navigation/theme icons, 12-point sort arrows, 14-point close, a 12-point square that paints a 10×10 glyph, an 18-point minus that paints a 12×2 glyph, and the player controls' 18/26/20-point source sizes.

## Tests and visual review

Add regression coverage for changing light/dark while Settings is already mounted, for the default system-follow appearance, and for enabling/disabling system following. Run the app checks and UI tests. Capture every app page in both appearances with GPUI's Windows offscreen renderer, which reads the local music folder, and use the headless web target to check empty states and the custom Lucide title-bar icons. Compare the results against the Slint screenshots; do not use an always-on-top capture method.

## Scope

Expected code changes are limited to `App.tsx`, `App.test.tsx`, `nsh.json`, and the new theme file. Rust audio and metadata behavior are out of scope.
