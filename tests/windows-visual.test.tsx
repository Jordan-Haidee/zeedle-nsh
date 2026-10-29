import { expect, open, test } from "nsh/test";
import { Music } from "zeedle-nsh/native";

test("Windows transport controls stay centered and pause remains stable", async () => {
  const app = await open("../App.tsx", { width: 900, height: 510, platform: "windows", colorScheme: "dark", storage: { "zeedle.light-theme": "false", "zeedle.show-spectrum": "false" } });
  await app.keyboard.press("f1");
  await app.waitFor("id:control-play-pause");
  const songs = JSON.parse(await Music.scan(Music.defaultDirectory())) as { id: number; song_name: string; singer: string; song_path: string }[];
  if (!songs.length) throw new Error("The default Music folder needs songs for Windows screenshots");
  const loveStory = songs.find((song) => song.song_name === "Love Story");
  if (!loveStory) throw new Error("The default Music folder needs Love Story for Windows screenshots");
  await app.waitFor(`text:${songs[0].song_name} - ${songs[0].singer}`);
  await app.press(`id:song-row-${loveStory.id}`);
  await app.press(`id:song-row-${loveStory.id}`);
  await app.waitFor("text:Love Story - Taylor Swift");
  await app.keyboard.press("right");
  await app.wait(150);
  await app.screenshot("docs/screenshots/gallery.png");
  for (const id of ["control-play-pause", "control-previous", "control-next"]) {
    await app.hover(`id:${id}`);
    await app.wait(180);
    await app.screenshot(`target/windows-${id}.png`);
  }
  await app.press("id:control-play-pause");
  await app.waitFor("label:Pause");
  await app.hover("id:control-play-pause");
  await app.screenshot("target/windows-pause.png");
  await app.press("id:control-play-pause");
  await app.waitFor("label:Play");
  for (let poll = 0; poll < 6; poll++) {
    await app.wait(120);
    expect(app.shows("label:Play")).toBe(true);
  }
  await app.screenshot("target/windows-paused-stable.png");
  await app.press("id:control-play-pause");
  await app.waitFor("label:Pause");
  Music.seek(130);
  await app.press("id:nav-2");
  await app.waitFor("id:lyric-active");
  await app.screenshot("docs/screenshots/lyrics.png");

  await app.press("id:nav-1");
  await app.type("id:song-search", "lo");
  await app.waitFor("text:2 result(s) found");
  await app.screenshot("docs/screenshots/search.png");

  await app.press("id:control-shuffle");
  await app.screenshot("target/windows-mode-random.png");
  await app.press("id:control-repeat");
  await app.screenshot("target/windows-mode-order.png");
  await app.press("id:nav-4");
  await app.waitFor("id:about-nsh-link");
  expect(app.shows("id:about-nsh-link")).toBe(true);
  await app.press("id:about-nsh-link");
  expect(app.shows("id:screen-4")).toBe(true);
  await app.screenshot("docs/screenshots/about.png");
  await app.press("id:theme-toggle");
  await app.press("id:nav-0");
  await app.waitFor("id:control-play-pause");
  for (const id of ["control-play-pause", "control-previous", "control-next"]) {
    await app.hover(`id:${id}`);
    await app.wait(180);
    await app.screenshot(`target/windows-light-${id}.png`);
  }
  await app.press("id:control-play-pause");
  await app.waitFor("label:Pause");
  await app.hover("id:control-play-pause");
  await app.screenshot("target/windows-light-pause.png");
  await app.press("id:control-play-pause");
  await app.waitFor("label:Play");

  await app.press("id:nav-0");
  await app.rightClick(`id:song-row-${loveStory.id}`);
  await app.chooseMenuItem("Edit metadata…");
  await app.waitFor("text:Edit song metadata");
  await app.screenshot("docs/screenshots/metadata-editor.png");

  const settings = await open("../App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: { "zeedle.music-directory": "Your Music Folder", "zeedle.light-theme": "false", "zeedle.show-spectrum": "false" },
  });
  await settings.press("id:nav-3");
  await settings.waitFor("text:Music directory");
  await settings.screenshot("docs/screenshots/settings.png");
});
