import json
import pathlib
import subprocess
import sys


repo = pathlib.Path(__file__).resolve().parent.parent
manifest_path = repo / "nsh.json"
original = manifest_path.read_bytes()

try:
    manifest = json.loads(original)
    manifest["root"] = "ui::App"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    result = subprocess.run(sys.argv[1:], cwd=repo, check=False)
finally:
    manifest_path.write_bytes(original)

sys.exit(result.returncode)
