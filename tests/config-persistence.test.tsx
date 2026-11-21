import { expect, open, test } from "nsh/test";
import { AppConfig } from "zeedle-nsh/native";
import "./restore-app-config";

type PersistedConfig = {
  song_dir: string;
  current_song_path: string | null;
  progress: number;
  play_mode: string;
  sort_key: string;
  sort_ascending: boolean;
  lang: string;
  light_ui: boolean;
  follow_system_theme: boolean;
  volume: number;
  show_spectrum: boolean;
  current_page: number;
  sidebar_expanded: boolean;
  gallery_viewport_y: number;
};

const originalConfig = JSON.parse(AppConfig.load()) as PersistedConfig;

const legacySettings = {
  "zeedle.music-directory": "C:/legacy-music",
  "zeedle.volume": "0.62",
  "zeedle.play-mode": "repeat",
  "zeedle.show-spectrum": "false",
  "zeedle.language": "zh",
  "zeedle.light-theme": "true",
};

test("loads Slint TOML, clears legacy keys, and saves current UI state on dispose", async () => {
  const app = await open("../App.tsx", {
    width: 900,
    height: 510,
    colorScheme: "dark",
    storage: legacySettings,
  });

  await app.waitFor("id:screen-4");
  await new Promise((resolve) => setTimeout(resolve, 250));
  expect(Object.keys(app.storage()).filter((key) => key.startsWith("zeedle."))).toEqual([]);
  await app.press("id:sidebar-toggle");
  expect(app.texts()).toContain("曲库");
  await app.wait(600);
  app.dispose();
  await new Promise((resolve) => setTimeout(resolve, 150));

  const saved = JSON.parse(AppConfig.load()) as PersistedConfig;
  const normalizePath = (value: string | null) => value?.replaceAll("\\", "/") ?? null;
  for (const key of [
    "progress",
    "play_mode",
    "sort_key",
    "sort_ascending",
    "lang",
    "light_ui",
    "follow_system_theme",
    "volume",
    "show_spectrum",
    "current_page",
    "gallery_viewport_y",
  ] as const) {
    expect(saved[key]).toBe(originalConfig[key]);
  }
  expect(normalizePath(saved.song_dir)).toBe(normalizePath(originalConfig.song_dir));
  expect(normalizePath(saved.current_song_path)).toBe(
    normalizePath(originalConfig.current_song_path),
  );
  expect(saved.sidebar_expanded).toBe(!originalConfig.sidebar_expanded);
});
