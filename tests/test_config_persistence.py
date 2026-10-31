import os
import subprocess
import sys
import tempfile
import tomllib
import wave
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TEST_COMMANDS = [
    ["nsh", "test", "tests/config-persistence.test.tsx", "--target", "windows"],
    [
        sys.executable,
        "scripts/with-rust-root.py",
        "nsh",
        "test",
        "tests/rust-config-persistence.test.tsx",
        "--target",
        "windows",
    ],
]


def main() -> int:
    for command in TEST_COMMANDS:
        with tempfile.TemporaryDirectory(prefix="zeedle-config-") as directory:
            env = os.environ.copy()
            fixture_directory = Path(directory)
            config_path = fixture_directory / "config.toml"
            music_directory = fixture_directory / "Music"
            music_directory.mkdir()
            music_path = music_directory.as_posix()
            song_path = (music_directory / "state-fixture.wav").as_posix()
            with wave.open(song_path, "wb") as audio_file:
                audio_file.setparams((1, 2, 8000, 0, "NONE", "not compressed"))
                audio_file.writeframes(b"\x00\x00" * 240_000)
            config_path.write_text(
                f'''song_dir = "{music_path}"
current_song_path = "{song_path}"
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
''',
                encoding="utf-8",
            )
            env["ZEEDLE_CONFIG_PATH"] = str(config_path)
            result = subprocess.run(command, cwd=ROOT, env=env, check=False)
            if result.returncode:
                return result.returncode
            with config_path.open("rb") as config_file:
                config = tomllib.load(config_file)
            assert config["song_dir"].replace("\\", "/") == music_path
            assert config["current_song_path"].replace("\\", "/") == song_path
            assert abs(config["progress"] - 12.5) < 0.1, config["progress"]
            assert config["volume"] == 0.42
            assert config["play_mode"] == "Random"
            assert config["show_spectrum"] is False
            assert config["lang"] == "zh"
            assert config["light_ui"] is True
            assert config["follow_system_theme"] is False
            assert config["sort_key"] == "BySinger"
            assert config["sort_ascending"] is False
            assert config["current_page"] == 4
            assert config["sidebar_expanded"] is True
            assert config["gallery_viewport_y"] == 33.0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
