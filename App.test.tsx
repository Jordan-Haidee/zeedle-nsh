import { expect, open, test } from "nsh/test";

test("navigates between the five Zeedle pages", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: { "zeedle.light-theme": "false" },
  });
  await app.wait(100);
  expect(app.shows("id:screen-0")).toBe(true);

  for (const index of [1, 2, 3, 4, 0] as const) {
    await app.press(`id:nav-${index}`);
    await app.wait(250);
    expect(app.shows(`id:screen-${index}`)).toBe(true);
  }

  await app.keyboard.press("f2");
  expect(app.shows("id:screen-1")).toBe(true);
  await app.keyboard.press("f3");
  expect(app.shows("id:screen-2")).toBe(true);
});

test("metadata actions open editable and confirmation dialogs", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    storage: { "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios" },
  });
  await app.waitFor("text:一眼万年");
  await app.wait(300);
  await app.rightClick("text:一眼万年");
  await app.wait(100);
  await app.chooseMenuItem("Edit metadata…");
  await app.waitFor("text:Edit song metadata");
  expect(app.texts()).toContain("Edit song metadata");
  await app.press("id:edit-cancel");
  await app.wait(250);
  await app.rightClick("text:一眼万年");
  await app.wait(100);
  await app.chooseMenuItem("Delete song…");
  await app.waitFor("text:Delete this song?");
  expect(app.texts()).toContain("Delete this song?");
  await app.press("id:delete-cancel");
});

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
  await app.wait(250);
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
