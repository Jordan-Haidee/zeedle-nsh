import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIRECTORY = ROOT / "tests" / "fixtures" / "rust-context-menu"


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="zeedle-rust-context-menu-") as directory:
        config_path = Path(directory) / "config.toml"
        config_path.write_text(
            "\n".join(
                [
                    f"song_dir = {json.dumps(FIXTURE_DIRECTORY.as_posix())}",
                    'current_song_path = ""',
                    "progress = 0.0",
                    'play_mode = "InOrder"',
                    'sort_key = "BySongName"',
                    "sort_ascending = true",
                    'lang = ""',
                    "light_ui = false",
                    "follow_system_theme = false",
                    "volume = 1.0",
                    "show_spectrum = false",
                    "current_page = 0",
                    "sidebar_expanded = false",
                    "gallery_viewport_y = 0.0",
                    "",
                ]
            ),
            encoding="utf-8",
        )

        env = os.environ.copy()
        env["ZEEDLE_CONFIG_PATH"] = str(config_path)
        command = [
            sys.executable,
            "scripts/with-rust-root.py",
            "nsh",
            "test",
            "tests/rust-context-menu.test.tsx",
            *sys.argv[1:],
        ]
        return subprocess.run(command, cwd=ROOT, env=env, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
