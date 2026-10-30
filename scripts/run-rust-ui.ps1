[CmdletBinding()]
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$NshArguments = @()
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot 'nsh.json'
$originalManifest = [System.IO.File]::ReadAllBytes($manifestPath)
$restoreManifest = $false
$exitCode = 0

Push-Location $repoRoot
try {
  $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding utf8 | ConvertFrom-Json -AsHashtable
  $manifest['root'] = 'ui::App'
  $updatedManifest = ConvertTo-Json -InputObject $manifest -Depth 100

  # Restore the original bytes even when NSH exits with an error or Ctrl-C.
  $restoreManifest = $true
  [System.IO.File]::WriteAllText(
    $manifestPath,
    $updatedManifest + [Environment]::NewLine,
    [System.Text.UTF8Encoding]::new($false)
  )

  $hasWindowSize = $false
  foreach ($argument in $NshArguments) {
    if ($argument -eq '--window-size' -or $argument.StartsWith('--window-size=')) {
      $hasWindowSize = $true
      break
    }
  }

  if ($hasWindowSize) {
    & nsh run @NshArguments
  } else {
    & nsh run --window-size 900x510 @NshArguments
  }
  if ($null -ne $LASTEXITCODE) { $exitCode = $LASTEXITCODE }
} finally {
  try {
    if ($restoreManifest) {
      [System.IO.File]::WriteAllBytes($manifestPath, $originalManifest)
    }
  } finally {
    Pop-Location
  }
}

exit $exitCode
