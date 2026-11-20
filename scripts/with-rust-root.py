import json
import pathlib
import subprocess
import sys


repo = pathlib.Path(__file__).resolve().parent.parent
manifest_path = repo / "nsh.json"
cargo_manifest_path = repo / "Cargo.toml"
original = manifest_path.read_bytes()
original_cargo = cargo_manifest_path.read_bytes()

try:
    default_features = b"default = []"
    if original_cargo.count(default_features) != 1:
        raise RuntimeError("Could not find the default Cargo feature list to enable rust-ui")
    cargo_manifest_path.write_bytes(
        original_cargo.replace(default_features, b'default = ["rust-ui"]', 1)
    )

    manifest = json.loads(original)
    manifest["root"] = "ui::App"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    result = subprocess.run(sys.argv[1:], cwd=repo, check=False)
finally:
    cargo_manifest_path.write_bytes(original_cargo)
    manifest_path.write_bytes(original)

sys.exit(result.returncode)
