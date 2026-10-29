import { expect, open, test } from "nsh/test";

test("navigates between the five Zeedle pages", async () => {
  const app = await open("./App.tsx", {
    width: 900,
    height: 510,
    platform: "windows",
  });
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
  });
  await app.wait(500);
  if (!app.shows("id:song-row-0")) return;
  await app.rightClick("id:song-row-0");
  await app.chooseMenuItem("Edit metadata…");
  await app.waitFor("text:Edit song metadata");
  expect(app.texts()).toContain("Edit song metadata");
  await app.press("id:edit-cancel");
  await app.wait(250);
  await app.rightClick("id:song-row-0");
  await app.chooseMenuItem("Delete song…");
  await app.waitFor("text:Delete this song?");
  expect(app.texts()).toContain("Delete this song?");
  await app.press("id:delete-cancel");
});
