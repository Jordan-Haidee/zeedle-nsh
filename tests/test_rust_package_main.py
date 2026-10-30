import subprocess
import sys
import unittest


class RustPackageMainTests(unittest.TestCase):
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
                self.assertIn("AppSource::Rust", result.stdout)


if __name__ == "__main__":
    unittest.main()
