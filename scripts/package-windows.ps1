$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
  $generatedBuild = Join-Path (Get-Location) '.nsh/rust/windows-app/build.rs'
  if (Test-Path -LiteralPath $generatedBuild) { Remove-Item -LiteralPath $generatedBuild }
  nsh package --target windows
  if ($LASTEXITCODE -ne 0) { throw 'NSH packaging failed' }
  Copy-Item -LiteralPath assets/cover.ico -Destination .nsh/out/Zeedle/nsh/assets/assets/cover.ico -Force
  Copy-Item -LiteralPath scripts/windows-main.rs -Destination .nsh/rust/windows-app/main.rs -Force
  Copy-Item -LiteralPath scripts/windows-build.rs -Destination .nsh/rust/windows-app/build.rs -Force
  Copy-Item -LiteralPath assets/cover.ico -Destination .nsh/rust/windows-app/cover.ico -Force
  Add-Content -LiteralPath .nsh/rust/windows-app/Cargo.toml -Value "`n[build-dependencies]`nwinresource = '=0.1.23'"
  # Copy-Item preserves mtime; force Cargo to notice the launcher replacement.
  (Get-Item -LiteralPath .nsh/rust/windows-app/main.rs).LastWriteTimeUtc = [DateTime]::UtcNow
  cargo build --offline --release --manifest-path .nsh/rust/windows-app/Cargo.toml --target-dir target
  if ($LASTEXITCODE -ne 0) { throw 'Windows startup host build failed' }
  Copy-Item -LiteralPath target/release/nsh-app-nsh-app-zeedle-nsh-windows-app.exe -Destination .nsh/out/Zeedle/Zeedle.exe -Force
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $zipPath = Join-Path (Get-Location) '.nsh/out/Zeedle.zip'
  if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath }
  [IO.Compression.ZipFile]::CreateFromDirectory((Join-Path (Get-Location) '.nsh/out/Zeedle'), $zipPath, [IO.Compression.CompressionLevel]::Optimal, $true)
  Write-Output "Packaged Zeedle with deferred centered startup: $zipPath"
} finally { Pop-Location }
