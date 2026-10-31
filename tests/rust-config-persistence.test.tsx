import { expect, open, test } from "nsh/test";

const legacySettings = {
  "zeedle.music-directory": "C:/legacy-music",
  "zeedle.volume": "0.62",
  "zeedle.play-mode": "repeat",
  "zeedle.show-spectrum": "false",
  "zeedle.language": "zh",
  "zeedle.light-theme": "true",
};

test("Rust UI loads Slint TOML, clears legacy keys, and saves current UI state on dispose", async () => {
  const app = await open("../", {
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
});
