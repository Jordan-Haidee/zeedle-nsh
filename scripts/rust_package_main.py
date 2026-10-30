import argparse
import re
import shutil
import subprocess
import tarfile
import zipfile
from pathlib import Path


def main_source(target: str) -> str:
    asset_dir = (
        'app_dir.parent().expect("Contents directory").join("Resources/nsh")'
        if target == "macos"
        else 'app_dir.join("nsh")'
    )
    return_code = (
        "if nsh_platform_linux::run_app(source, options) == nsh_platform_linux::gtk4::glib::ExitCode::SUCCESS {\n"
        "        std::process::ExitCode::SUCCESS\n"
        "    } else {\n"
        "        std::process::ExitCode::FAILURE\n"
        "    }"
        if target == "linux"
        else f"nsh_platform_{target}::run_app(source, options)"
    )
    return f'''#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(target_os = "{target}")]
fn main() -> std::process::ExitCode {{
    use nsh_platform_{target}::{{AppOptions, AppSource, AssetBase, Limits, Root}};

    let executable = std::env::current_exe().expect("current executable path");
    let app_dir = executable.parent().expect("executable directory");
    let asset_dir = {asset_dir};

    nsh_platform_{target}::set_ui_sets(vec![
        nsh_base::registry(),
        app_nsh_app_zeedle_nsh::registry(),
    ]);
    let options = AppOptions {{
        title: "Zeedle".into(),
        app_id: "dev.zeedle.music-player".into(),
        size: (900, 510),
        asset_base: Some(AssetBase::Directory(asset_dir.join("assets"))),
        limits: Limits::standalone(),
        ..AppOptions::default()
    }};
    let source = AppSource::Rust(Root::of::<app_nsh_app_zeedle_nsh::ui::App>());
    {return_code}
}}
'''


def tsx_main_source() -> str:
    return '''#![cfg(target_os = "linux")]

fn main() -> std::process::ExitCode {
    use nsh_platform_linux::{AppOptions, AppSource, AssetBase, Bundle, Limits};

    let executable = std::env::current_exe().expect("current executable path");
    let app_dir = executable.parent().expect("executable directory");
    let nsh_dir = app_dir.join("nsh");
    // SAFETY: the bytecode is shipped inside this app's own package.
    let bundle = match unsafe { Bundle::read_trusted(&nsh_dir.join("bundle.nshc")) } {
        Ok(bundle) => bundle,
        Err(error) => {
            eprintln!("cannot read the app bundle: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };

    nsh_platform_linux::set_ui_sets(vec![
        nsh_base::registry(),
        app_nsh_app_zeedle_nsh::registry(),
    ]);
    let options = AppOptions {
        title: "Zeedle".into(),
        app_id: "dev.zeedle.music-player".into(),
        size: (900, 510),
        asset_base: Some(AssetBase::Directory(nsh_dir.join("assets"))),
        limits: Limits::standalone(),
        ..AppOptions::default()
    };
    if nsh_platform_linux::run_app(AppSource::Bundle(bundle), options)
        == nsh_platform_linux::gtk4::glib::ExitCode::SUCCESS
    {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
'''


def install(target: str, ui: str = "rustui") -> None:
    if ui == "tsxui" and target != "linux":
        raise ValueError("the custom TSX runner is only needed for Linux")
    app_source = Path(".nsh/rust") / f"{target}-app"
    manifest = app_source / "Cargo.toml"
    main = app_source / "main.rs"
    source = tsx_main_source() if ui == "tsxui" else main_source(target)
    main.write_text(source, encoding="utf-8", newline="\n")
    app_crate = f"nsh-{target}-app"
    cargo_toml = manifest.read_text(encoding="utf-8")
    cargo_toml = re.sub(rf"^{re.escape(app_crate)}\s*=.*\n", "", cargo_toml, flags=re.M)
    if target == "windows":
        if "winresource" not in cargo_toml:
            cargo_toml = cargo_toml.rstrip() + "\n\n[build-dependencies]\nwinresource = '=0.1.23'\n"
    manifest.write_text(cargo_toml, encoding="utf-8", newline="\n")

    package_name = re.search(r'^name\s*=\s*"([^"]+)"', manifest.read_text(encoding="utf-8"), re.M)
    if package_name is None:
        raise RuntimeError(f"No Cargo package name in {manifest}")
    binary = Path("target/release") / package_name.group(1)
    if target == "windows":
        binary = binary.with_suffix(".exe")

    command = ["cargo", "build", "--release", "--manifest-path", str(manifest), "--target-dir", "target", "--offline"]
    result = subprocess.run(command, check=False)
    if result.returncode:
        result = subprocess.run(command[:-1], check=False)
    if result.returncode:
        raise SystemExit(result.returncode)

    package = Path(".nsh/out") / ui
    if target == "windows":
        app = package / "Zeedle"
        destination = app / "Zeedle.exe"
    elif target == "linux":
        app = package / "Zeedle"
        destination = app / "Zeedle"
    else:
        app = package / "Zeedle.app"
        destination = app / "Contents/MacOS/Zeedle"
    shutil.copy2(binary, destination)

    if target == "windows":
        archive = package / "Zeedle.zip"
        archive.unlink(missing_ok=True)
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
            for file in app.rglob("*"):
                if file.is_file():
                    output.write(file, Path(app.name) / file.relative_to(app))
    elif target == "linux":
        archive = package / "Zeedle.tar.gz"
        archive.unlink(missing_ok=True)
        with tarfile.open(archive, "w:gz") as output:
            output.add(app, arcname=app.name)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=("windows", "linux", "macos"), required=True)
    parser.add_argument("--ui", choices=("rustui", "tsxui"), default="rustui")
    parser.add_argument("--print-source", action="store_true")
    parser.add_argument("--install", action="store_true")
    arguments = parser.parse_args()
    if arguments.print_source:
        if arguments.ui == "tsxui" and arguments.target != "linux":
            parser.error("the custom TSX runner is only needed for Linux")
        source = tsx_main_source() if arguments.ui == "tsxui" else main_source(arguments.target)
        print(source, end="")
    elif arguments.install:
        install(arguments.target, arguments.ui)
    else:
        parser.error("choose --print-source or --install")
