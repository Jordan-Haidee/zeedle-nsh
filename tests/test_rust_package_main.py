import subprocess
import sys
import unittest


class RustPackageMainTests(unittest.TestCase):
    def test_windows_rust_runner_waits_until_the_centered_window_is_ready(self):
        result = subprocess.run(
            [sys.executable, "scripts/rust_package_main.py", "--target", "windows", "--print-source"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("AppView::new", result.stdout)
        self.assertIn("set_startup_window", result.stdout)
        self.assertIn("SHOW_READY", result.stdout)
        self.assertIn("view.load(root.into())", result.stdout)
        self.assertIn("ShowWindow(hwnd, SW_SHOWNOACTIVATE)", result.stdout)

    def test_each_platform_runner_uses_the_slint_startup_size(self):
        for target in ("windows", "linux", "macos"):
            with self.subTest(target=target):
                result = subprocess.run(
                    [sys.executable, "scripts/rust_package_main.py", "--target", target, "--print-source"],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("size: (900, 510)", result.stdout)
                self.assertIn('Theme::from_json(include_str!("../../../theme.json"))', result.stdout)
                self.assertIn(".tokens(", result.stdout)
                if target == "windows":
                    self.assertIn("view.load(root.into())", result.stdout)
                else:
                    self.assertIn("AppSource::Rust(root)", result.stdout)

    def test_linux_tsx_runner_sets_window_size_before_loading_the_bundle(self):
        result = subprocess.run(
            [
                sys.executable,
                "scripts/rust_package_main.py",
                "--target",
                "linux",
                "--ui",
                "tsxui",
                "--print-source",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("size: (900, 510)", result.stdout)
        self.assertIn("AppSource::Bundle", result.stdout)
        self.assertIn("Bundle::read_trusted", result.stdout)


if __name__ == "__main__":
    unittest.main()
