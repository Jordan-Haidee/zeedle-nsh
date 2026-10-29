import { expect, open, test } from "nsh/test";
import { Music } from "zeedle-nsh/native";

test("one pause click stays paused across playback state polls", async () => {
  const app = await open("./App.tsx", { width: 900, height: 510, platform: "windows", storage: { "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios" } });
  await app.waitFor("text:一眼万年 - S.H.E");
  await app.press("id:control-play-pause");
  await app.waitFor("label:Pause");
  await app.press("id:control-play-pause");
  await app.waitUntil(() => JSON.parse(Music.state()).paused === true);
  for (let poll = 0; poll < 6; poll++) {
    await app.wait(120);
    expect(app.shows("label:Play")).toBe(true);
    expect(JSON.parse(Music.state()).paused).toBe(true);
  }
});

test("About links are accepted by the native browser opener only over HTTP", () => {
  expect(Music.isBrowserUrl("https://nsh.dev")).toBe(true);
  expect(Music.isBrowserUrl("http://example.com")).toBe(true);
  expect(Music.isBrowserUrl("file:///C:/secret.txt")).toBe(false);
  expect(Music.isBrowserUrl("javascript:alert(1)")).toBe(false);
});

test("native browser opener rejects unsafe schemes without launching", async () => {
  expect(await Music.openUrl("javascript:alert(1)")).toBe("Only HTTP and HTTPS links can be opened");
});

test("repeat and shuffle replace each other and toggle back to order", async () => {
  const app = await open("./App.tsx", { width: 900, height: 510, platform: "windows" });
  await app.wait(100);
  for (const [button, expected] of [["control-repeat", "repeat"], ["control-repeat", "order"], ["control-shuffle", "random"], ["control-repeat", "order"], ["control-shuffle", "random"]]) {
    await app.press(`id:${button}`);
    await app.waitUntil(() => app.storage()["zeedle.play-mode"] === expected);
  }
});

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
    await app.waitFor(`id:screen-${index}`);
    expect(app.shows(`id:screen-${index}`)).toBe(true);
  }

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

test("song rows start playback only on double click", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });

  await app.waitFor("text:万疆");
  await app.waitFor("text:一眼万年 - S.H.E");
  await app.press("text:万疆");
  await app.wait(100);
  expect(app.texts()).toContain("一眼万年 - S.H.E");
  expect(app.shows("text:万疆 - 李玉刚")).toBe(false);
  await app.press("text:万疆");
  await app.wait(100);
  expect(app.texts()).toContain("万疆 - 李玉刚");
});

test("pressing a lyric seeks the playing song", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "C:/Users/dell/Music",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });

  await app.waitFor("text:Love Story");
  await app.press("text:Love Story");
  await app.press("text:Love Story");
  await app.waitFor("text:Love Story - Taylor Swift");
  await app.wait(200);
  const positionBeforeSeek = (JSON.parse(Music.state()) as { position: number }).position;
  await app.press("id:nav-2");
  await app.waitFor("text:We were both young when I first saw you");
  await app.press("text:We were both young when I first saw you");
  await app.wait(150);
  const seekState = JSON.parse(Music.state()) as { position: number; paused: boolean };
  expect(seekState.position > positionBeforeSeek + 10).toBe(true);
  await app.press("id:nav-0");
  await app.wait(120);
  expect(app.texts().some((text) => text.startsWith("00:16 /"))).toBe(true);
});

test("lyric seek is retained before playback starts", async () => {
  const directory = "C:/Users/dell/Music";
  const songs = JSON.parse(await Music.scan(directory)) as { song_name: string; singer: string; song_path: string }[];
  const song = songs.find((item) => item.song_name === "Deadman");
  if (!song) throw new Error("Deadman seek fixture was not found");
  const lines = JSON.parse(await Music.lyrics(song.song_path)) as { time: number; text: string }[];
  const target = lines.find((line) => line.time >= 10);
  if (!target) throw new Error("Deadman seek fixture has no timestamped lyrics");

  Music.stop();
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": directory,
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });
  await app.waitFor(`text:${song.song_name} - ${song.singer}`);
  await app.press("id:nav-2");
  await app.waitFor(`text:${target.text}`);
  await app.press(`text:${target.text}`);
  await app.wait(200);
  const state = JSON.parse(Music.state()) as { position: number };
  expect(state.position >= target.time - 1).toBe(true);
});

test("playback keeps the active lyric in the visible list", async () => {
  const directory = "C:/Users/dell/Music";
  const songs = JSON.parse(await Music.scan(directory)) as { song_name: string; song_path: string }[];
  const song = songs.find((item) => item.song_name === "Love Story");
  if (!song) throw new Error("Love Story fixture was not found");
  const lyricItems = JSON.parse(await Music.lyrics(song.song_path)) as { time: number }[];
  const targetTime = Math.max(...lyricItems.filter((line) => line.time <= 90).map((line) => line.time));
  const targetIndex = lyricItems.reduce((last, line, index) => line.time === targetTime ? index : last, -1);
  const steps = new Set(lyricItems.filter((line) => line.time > 0 && line.time <= targetTime).map((line) => line.time)).size;
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": directory,
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });

  await app.waitFor("text:Love Story");
  await app.press("text:Love Story");
  await app.press("text:Love Story");
  await app.waitFor("text:Love Story - Taylor Swift");
  await app.press("id:nav-2");
  for (let index = 0; index < steps; index++) await app.keyboard.press("right");
  await app.wait(120);
  expect(app.shows("id:lyric-active")).toBe(true);
});

test("the About page shows the NSH browser link", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: { "zeedle.light-theme": "false" },
  });
  await app.wait(100);
  await app.press("id:nav-4");
  await app.waitFor("text:NSH");
  expect(app.shows("id:about-nsh-link")).toBe(true);
});

test("clicking the progress bar seeks the playing song", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });

  await app.waitFor("text:一眼万年 - S.H.E");
  await app.press("text:一眼万年");
  await app.press("text:一眼万年");
  await app.wait(200);
  const started = JSON.parse(Music.state()) as { position: number; duration: number };
  await app.press("id:progress-seek");
  await app.wait(150);
  const tapped = (JSON.parse(Music.state()) as { position: number }).position;
  expect(tapped > started.position + started.duration * 0.4).toBe(true);
});

test("dragging the progress bar does not seek", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "false",
    },
  });

  await app.waitFor("text:一眼万年 - S.H.E");
  await app.press("text:一眼万年");
  await app.press("text:一眼万年");
  await app.wait(150);
  const before = (JSON.parse(Music.state()) as { position: number }).position;
  await app.drag("id:progress-seek", { x: 150, y: 0 });
  await app.wait(150);
  const after = (JSON.parse(Music.state()) as { position: number }).position;
  expect(after < before + 5).toBe(true);
});

test("mounted settings restyle when the system appearance changes", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
    colorScheme: "dark",
  });

  await app.wait(100);
  if (app.storage()["zeedle.light-theme"] !== undefined) {
    await app.press("text:Follow system theme");
    await app.waitUntil(() => app.storage()["zeedle.light-theme"] === undefined);
  }
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
