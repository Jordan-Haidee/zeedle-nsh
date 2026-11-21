import { afterAll } from "nsh/test";
import { AppConfig } from "zeedle-nsh/native";

const originalConfig = AppConfig.load();

afterAll(() => {
  const error = AppConfig.save(originalConfig);
  if (error) throw new Error(`could not restore app config after test: ${error}`);
});
