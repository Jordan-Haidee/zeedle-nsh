import { expect, open, test } from "nsh/test";

test("captures full Zeedle screens for the public README", async () => {
  const app = await open("../App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: { "zeedle.light-theme": "false", "zeedle.show-spectrum": "true", "zeedle.language": "en" },
  });

  await app.waitFor("text:Deadman - 蔡徐坤");
  for (let track = 0; track < 8; track++) await app.press("label:Next track");
  await app.waitFor("text:Love Story - Taylor Swift");
  await app.press("id:nav-0");
  await app.keyboard.press("right");
  await app.wait(120);
  await app.screenshot("docs/screenshots/gallery.png");

  await app.press("id:nav-1");
  await app.type("id:song-search", "Love Story");
  await app.waitFor("text:1 result(s) found");
  await app.screenshot("docs/screenshots/search.png");
  await app.press("id:nav-0");
  await app.press("id:progress-seek");
  await app.wait(150);
  await app.press("id:nav-2");
  await app.waitFor("id:lyric-active");
  await app.screenshot("docs/screenshots/lyrics.png");

  await app.press("id:nav-4");
  await app.waitFor("id:about-nsh-link");
  await app.screenshot("docs/screenshots/about.png");

  await app.press("id:nav-0");
  await app.rightClick("text:Love Story");
  await app.chooseMenuItem("Edit metadata…");
  await app.waitFor("text:Edit song metadata");
  await app.screenshot("docs/screenshots/metadata-editor.png");

  const settings = await open("../App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "Your Music Folder",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
      "zeedle.language": "en",
    },
  });
  await settings.press("id:nav-3");
  await settings.waitFor("text:Music directory");
  await settings.screenshot("docs/screenshots/settings.png");
  expect(settings.shows("id:screen-3")).toBe(true);
});
