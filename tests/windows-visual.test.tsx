import { expect, open, test } from "nsh/test";

test("Windows transport controls stay centered and pause remains stable", async () => {
  const app = await open("../App.tsx", { width: 900, height: 510, platform: "windows", colorScheme: "dark", storage: { "zeedle.music-directory": "D:/rs-projects/zeedle-slint/audios", "zeedle.light-theme": "false", "zeedle.show-spectrum": "false" } });
  await app.waitFor("text:一眼万年 - S.H.E");
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
  await app.press("id:control-shuffle");
  await app.screenshot("target/windows-mode-random.png");
  await app.press("id:control-repeat");
  await app.screenshot("target/windows-mode-order.png");
  await app.press("id:theme-toggle");
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
});
