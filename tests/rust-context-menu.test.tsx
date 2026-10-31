import { expect, open, test } from "nsh/test";

test("Rust song context menu follows the selected row and dismisses outside", async () => {
  const app = await open("../", {
    width: 900,
    height: 510,
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "tests/fixtures/missing-legacy-library",
      "zeedle.show-spectrum": "true",
      "zeedle.language": "zh",
    },
  });

  await app.waitFor("text:AlphaTone");
  await app.waitFor("text:BetaTone");
  expect(Object.keys(app.storage()).filter((key) => key.startsWith("zeedle."))).toEqual([]);

  await app.rightClick("text:AlphaTone");
  await app.waitFor("id:song-action-menu");
  expect(app.texts().filter((text) => text === "AlphaTone").length).toBe(2);
  await app.press("id:action-menu-backdrop");
  expect(app.shows("id:song-action-menu")).toBe(false);
  await app.screenshot("target/rust-context-menu-dismissed.png");

  await app.rightClick("text:BetaTone");
  await app.waitFor("id:song-action-menu");
  expect(app.texts().filter((text) => text === "BetaTone").length).toBe(2);
  await app.screenshot("target/rust-context-menu-selected-beta.png");
  await app.press("id:action-menu-backdrop");
  expect(app.shows("id:song-action-menu")).toBe(false);
});
