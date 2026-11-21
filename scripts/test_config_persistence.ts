const childProcessModule = "node:child_process";
const fsModule = "node:fs/promises";
const osModule = "node:os";
const pathModule = "node:path";
const processModule = "node:process";
const bufferModule = "node:buffer";
const urlModule = "node:url";

const { spawnSync } = await import(childProcessModule);
const fs = await import(fsModule);
const os = await import(osModule);
const path = (await import(pathModule)).default;
const process = (await import(processModule)).default;
const { Buffer } = await import(bufferModule);
const { fileURLToPath } = await import(urlModule);

function silentWave(frameCount: number) {
  const channels = 1;
  const sampleRate = 8000;
  const bytesPerSample = 2;
  const dataBytes = frameCount * channels * bytesPerSample;
  const wave = Buffer.alloc(44 + dataBytes);
  wave.write("RIFF", 0, "ascii");
  wave.writeUInt32LE(36 + dataBytes, 4);
  wave.write("WAVE", 8, "ascii");
  wave.write("fmt ", 12, "ascii");
  wave.writeUInt32LE(16, 16);
  wave.writeUInt16LE(1, 20);
  wave.writeUInt16LE(channels, 22);
  wave.writeUInt32LE(sampleRate, 24);
  wave.writeUInt32LE(sampleRate * channels * bytesPerSample, 28);
  wave.writeUInt16LE(channels * bytesPerSample, 32);
  wave.writeUInt16LE(bytesPerSample * 8, 34);
  wave.write("data", 36, "ascii");
  wave.writeUInt32LE(dataBytes, 40);
  return wave;
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tempRoot = path.resolve(os.tmpdir());
const tempDirectory = await fs.mkdtemp(path.join(tempRoot, "zeedle-config-"));
const configPath = path.join(tempDirectory, "config.toml");
const musicDirectory = path.join(tempDirectory, "Music");
const musicPath = musicDirectory.replaceAll("\\", "/");
const songPath = path.join(musicDirectory, "state-fixture.wav").replaceAll("\\", "/");

try {
  await fs.mkdir(musicDirectory);
  await fs.writeFile(path.join(musicDirectory, "state-fixture.wav"), silentWave(240_000));
  await fs.writeFile(configPath, `song_dir = "${musicPath}"
current_song_path = "${songPath}"
progress = 12.5
play_mode = "Random"
sort_key = "BySinger"
sort_ascending = false
lang = "zh"
light_ui = true
follow_system_theme = false
volume = 0.42
show_spectrum = false
current_page = 4
sidebar_expanded = false
gallery_viewport_y = 33.0
`, "utf8");

  const result = spawnSync("nsh", ["test", "tests/config-persistence.test.tsx"], {
    cwd: root,
    env: { ...process.env, ZEEDLE_CONFIG_PATH: configPath },
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`nsh test exited with status ${result.status}`);
} finally {
  const tempPrefix = `${tempRoot}${path.sep}`;
  const resolvedTemp = path.resolve(tempDirectory);
  if (!resolvedTemp.startsWith(tempPrefix)) throw new Error(`Refusing to remove unexpected temp path: ${resolvedTemp}`);
  await fs.rm(resolvedTemp, { recursive: true, force: true });
}
