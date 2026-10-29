# NSH Theme and Slint Visual Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make theme changes consistent across mounted NSH views and match the original Slint palette, page alignment, and Lucide button-icon sizes.

**Architecture:** Add a `theme.json` NSH theme with standard and Zeedle-specific light/dark tokens. Read tokens reactively from `App.tsx`; keep `setColorScheme()` as the single appearance controller, correct the system-follow preference transitions, and remove theme-dependent values cached at SettingsPage creation. Keep layout changes in the existing TSX tree and use the Slint source and `assets/p1.png`–`p5.png` as visual references.

**Tech Stack:** NSH TSX, NSH `token()` and `setColorScheme()` APIs, `nsh/test`, GPUI offscreen screenshots and headless web screenshots, Slint 1.17.1 reference source.

---

### Task 1: Add failing theme regression tests

**Files:**
- Modify: `App.test.tsx`
- Test: `App.test.tsx`

- [x] **Step 1: Add tests for mounted Settings and system-follow transitions**

```tsx
test("mounted settings restyle when the system appearance changes", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
  });

  await app.wait(100);
  await app.press("id:nav-3");
  await app.wait(250);
  const darkPlayback = app.spanStyle("text:Playback").color;
  await app.setColorScheme("light");
  const lightPlayback = app.spanStyle("text:Playback").color;
  expect(lightPlayback === darkPlayback).toBe(false);
  await app.setColorScheme("dark");
  expect(app.spanStyle("text:Playback").color).toBe(darkPlayback);
});

test("turning off system-follow pins the current appearance", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: { "zeedle.light-theme": "false" },
  });

  await app.wait(100);
  await app.press("id:nav-3");
  await app.press("text:Follow system theme");
  await app.setColorScheme("light");
  const lightText = app.spanStyle("text:Music directory").color;
  await app.press("text:Follow system theme");
  await app.wait(100);
  expect(app.storage()["zeedle.light-theme"]).toBe("true");
  const pinnedText = app.spanStyle("text:Music directory").color;
  await app.setColorScheme("dark");
  await app.wait(100);
  expect(pinnedText).toBe(lightText);
  expect(app.spanStyle("text:Music directory").color).toBe(pinnedText);
});
```

- [x] **Step 2: Run the UI tests and confirm both new tests fail for the current bug**

Run: `rtk nsh test App.test.tsx`

Expected: the first new test reports that the mounted `Playback` text has not changed to the light color; the second reports that `Music directory` changed to dark after system-follow was disabled. Existing navigation and metadata tests remain unchanged.

### Task 2: Define the NSH light and dark theme tokens

**Files:**
- Create: `theme.json`
- Modify: `nsh.json`

- [x] **Step 1: Add the theme file with standard and Zeedle-specific tokens**

Use NSH's `cssVars` form. Put `font-sans: "Microsoft YaHei UI"` in `cssVars.theme`; put standard colors and Zeedle-specific tokens in `cssVars.light` and `cssVars.dark`. Define `page`, `panel`, `raised`, `hover`, `song-hover`, `pressed`, `text-secondary`, `text-tertiary`, `divider`, `input-border`, `button`, `checkbox-border`, `danger`, `danger-surface`, `danger-text`, `shadow`, `track`, and `track-thumb-border` in both appearances. Map surface/text/accent roles to `background`, `foreground`, `primary`, `primary-foreground`, `border`, `input`, and the sidebar tokens so NSH components use the same appearance.

Use `ui/theme.slint` and sampled Slint screenshots as the color authority. Preserve the measured dark values: page `#141414`, panel `#272727`, hover/input `#323232`, panel border `#3B3B3B`, input border `#454545`, divider `#464646`, text `#FFFFFF`, secondary/tertiary text `#C4C4C4`, and accent `#6CC4FF`. For light mode use page `#FFFFFF`, panel `#F5F5F6`, hover/raised `#E8E9EC`, border `#DEDFE2`, text `#1A1A1A`, secondary `#666666`, tertiary `#999999`, and accent `#005FB8`. Keep dark accent-soft `#20354E` and danger `#EF7378`.

- [x] **Step 2: Point `nsh.json` at `theme.json`**

Add the manifest field:

```json
"theme": "theme.json"
```

- [x] **Step 3: Run NSH type generation and validation**

Run: `rtk nsh check -v`

Expected: the theme parses, `.nsh/types/nsh/tokens.d.ts` includes the custom token names, the app has no type errors, and the Rust crate builds.

### Task 3: Make all app colors reactive and repair appearance transitions

**Files:**
- Modify: `App.tsx`
- Test: `App.test.tsx`

- [x] **Step 1: Import `token` and replace hard-coded theme color helpers**

```tsx
import { Image, List, Scroll, Storage, Text, View, colorScheme, keymap, mount, openUrl, setColorScheme, signal, token } from "nsh";

const textColor = () => token("foreground");
const secondaryColor = () => token("text-secondary");
const mutedColor = () => token("text-tertiary");
const panelColor = () => token("panel");
const raisedColor = () => token("raised");
const lineColor = () => token("border");
const pageColor = () => token("page");
```

Set `textStyle`'s `fontFamily` to `"Microsoft YaHei UI"`, the Windows family used by the source Slint application.

Replace every remaining UI color literal or `light()` ternary that represents a theme role with its token accessor, including navigation, player controls, settings, text inputs, metadata dialogs, error surfaces, and the spectrum. Keep literal colors only for image content or states that are deliberately invariant across the two themes.

- [x] **Step 2: Remove the cached Settings style values**

Delete the hoisted `cardStyle`, `sectionStyle`, and `divider` values in `SettingsPage`. Put their token reads directly in each rendered JSX style object so `token()` is read by a reactive binding. Use outer padding 24, inter-card gap 20, card padding 16, and a 6-point directory label/input group; set the card's NSH gap to 10 points to match the rendered Slint 12-point spacing.

- [x] **Step 3: Fix manual and system theme state transitions**

When no `zeedle.light-theme` value exists, set `followSystem` to `true` and call `setColorScheme("system")`, matching `Config::default` in the Slint app. When a saved value exists, set `followSystem` to `false` and call `setColorScheme(savedTheme === "true" ? "light" : "dark")`. Turning system-follow on calls `setColorScheme("system")` and removes the saved override. Turning it off reads `colorScheme()`, sets that explicit scheme with `setColorScheme`, and stores whether it is light. The title-bar theme button is disabled and dimmed while system-follow is enabled; otherwise it toggles and persists the explicit scheme.

- [x] **Step 4: Run the UI tests and confirm both theme tests pass**

Run: `rtk nsh test App.test.tsx`

Expected: all existing tests and the two new theme transition tests pass.

### Task 4: Match the Slint component spacing and Lucide icon sizes

**Files:**
- Modify: `App.tsx`
- Test: `App.test.tsx`

- [x] **Step 1: Match the search page**

Set the search field height to 40 points, remove its leading search icon, retain the 14-point clear icon, and set the empty state to a 44-point search icon with 14-point text. Place a 24-point input underline inside the 40-point field, with 12-point horizontal inset and no second outer margin.

- [x] **Step 2: Match the Settings and metadata editor surfaces**

Align the Settings card bounds and directory grouping to the source screenshots. Keep the metadata dialog at 520×450 points, use an 18-point title, 14-point form labels, 200-point cover/lyrics panels with 6-point inner spacing, and remove the dimming layer behind it, as shown in `assets/p5.png`. Match the Slint About page's 120-point cover, 14-point text, source description, author, version, and Github link.

- [x] **Step 3: Match Lucide glyph sizes to their Slint components**

Keep sidebar navigation, sidebar toggle, and title theme glyphs at 18 points; set the selected sidebar marker to 3×16 points; set sort arrows to 12 points (the `TitleBarItem` arrow box), and title-bar close to 14 points. Size Lucide minus to 18 points so its painted path matches the source 12×2 shape; size the square to 12 points so its painted outline matches 10×10. Give the repeat, previous, play, next, and shuffle groups 6% of the player row each; size repeat/shuffle to 18 points, skip/play to 26 points, and volume to 20 points. Use 33% table columns with text at 40% of each column, as `SongItem` and `SortColumn` do.

- [x] **Step 4: Run the UI tests and static checks**

Run: `rtk nsh test App.test.tsx`

Run: `rtk nsh check -v`

Expected: all UI tests pass, no TypeScript errors appear, and the Rust crate builds.

### Task 5: Render and review all reference pages in both appearances

**Files:**
- Create temporarily under ignored output: `target/slint-align.test.tsx`
- Create temporarily under ignored output: `target/slint-align-web.test.tsx`
- Verify: `target/slint-align-*.png` (ignored screenshots)
- Compare: `zeedle-slint/assets/p1.png` through `p5.png`

- [x] **Step 1: Create a temporary screenshot test under `target/`**

```tsx
import { open, test } from "nsh/test";

test("captures Zeedle pages in both themes", async () => {
  const app = await open("../App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "~/Music",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "true",
    },
  });
  await app.waitFor("text:/^Love Story$/");
  await app.press("text:/^Love Story$/");
  await app.wait(300);
  await app.screenshot("target/slint-align-dark-gallery.png");

  await app.press("id:nav-1");
  await app.type("id:song-search", "lo");
  await app.screenshot("target/slint-align-dark-search.png");
  await app.press("id:nav-2");
  await app.screenshot("target/slint-align-dark-lyrics.png");
  await app.press("id:nav-3");
  await app.screenshot("target/slint-align-dark-settings.png");
  await app.press("id:nav-4");
  await app.screenshot("target/slint-align-dark-about.png");

  await app.press("id:nav-0");
  await app.rightClick("text:/^Love Story$/");
  await app.chooseMenuItem("Edit metadata…");
  await app.screenshot("target/slint-align-dark-editor.png");
  await app.press("id:edit-cancel");

  await app.press("id:theme-toggle");
  await app.screenshot("target/slint-align-light-gallery.png");
  await app.press("id:nav-1");
  await app.screenshot("target/slint-align-light-search.png");
  await app.press("id:nav-2");
  await app.screenshot("target/slint-align-light-lyrics.png");
  await app.press("id:nav-3");
  await app.screenshot("target/slint-align-light-settings.png");
  await app.press("id:nav-4");
  await app.screenshot("target/slint-align-light-about.png");

  await app.press("id:nav-0");
  await app.rightClick("text:/^Love Story$/");
  await app.chooseMenuItem("Edit metadata…");
  await app.screenshot("target/slint-align-light-editor.png");
});
```

- [x] **Step 2: Capture all content pages with GPUI's offscreen Windows renderer**

Run: `rtk nsh test target/slint-align.test.tsx --target gpui`

The test saves gallery, search, lyrics, settings, about, and metadata-editor screenshots in both appearances. The Windows GPUI host uses `Window::render_to_image` for these captures; it does not change window z-order.

- [x] **Step 3: Capture the custom title bar and no-results search state in headless Chromium**

Run: `rtk proxy powershell -NoProfile -Command '$env:CARGO_TARGET_DIR = "D:\rs-projects\zeedle-nsh\target"; rtk nsh test target/slint-align-web.test.tsx --target web'`

The web test captures all five pages in both appearances; use it to inspect the custom Lucide window controls and empty search state. The GPUI test supplies the screenshots with local songs, lyrics, and metadata. It uses a cached shared target to avoid rebuilding the web runtime's Windows-incompatible WASI dependency.

- [x] **Step 4: Inspect and compare the screenshots**

Open the twelve GPUI screenshots and ten web screenshots with `view_image`. Compare title bar/sidebar colors, page and card backgrounds, text contrast, search field position/height, table columns, settings card edges and spacing, modal presentation, and icon sizes against the five Slint references. Correct each measured mismatch in `App.tsx` or `theme.json`, then repeat Steps 1–4 until all five pages match in both appearances.

- [x] **Step 5: Verify the final worktree**

Run: `rtk git diff --check`

Run: `rtk git status --short`

Expected: there are no whitespace errors; implementation changes are limited to `App.tsx`, `App.test.tsx`, `nsh.json`, and `theme.json`, the approved spec and plan remain under `docs/superpowers/`, and screenshot artifacts remain ignored under `target/`.
