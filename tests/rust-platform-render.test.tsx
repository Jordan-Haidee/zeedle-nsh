import { expect, open, test } from "nsh/test";

test("renders Rust UI pages and captures platform screenshots", async () => {
  const app = await open("../", {
    width: 900,
    height: 510,
    colorScheme: "dark",
    storage: {
      "zeedle.music-directory": "target/rust-platform-render-empty-music",
      "zeedle.light-theme": "false",
      "zeedle.show-spectrum": "true",
      "zeedle.language": "en",
    },
  });

  await app.press("id:nav-0");
  await app.waitFor("id:screen-0");
  expect((await app.screenshot("target/rust-platform-render-library.png")).length > 1000).toBe(true);

  await app.press("id:nav-3");
  await app.waitFor("text:Music directory");
  expect((await app.screenshot("target/rust-platform-render-settings.png")).length > 1000).toBe(true);

  await app.press("id:nav-4");
  await app.waitFor("id:about-nsh-link");
  expect((await app.screenshot("target/rust-platform-render-about.png")).length > 1000).toBe(true);
});
