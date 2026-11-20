[CmdletBinding()]
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$NshArguments = @()
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot 'nsh.json'
$cargoManifestPath = Join-Path $repoRoot 'Cargo.toml'
$originalManifest = [System.IO.File]::ReadAllBytes($manifestPath)
$originalCargoManifest = [System.IO.File]::ReadAllBytes($cargoManifestPath)
$restoreManifest = $false
$restoreCargoManifest = $false
$exitCode = 0

Push-Location $repoRoot
try {
  $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding utf8 | ConvertFrom-Json -AsHashtable
  $manifest['root'] = 'ui::App'
  $updatedManifest = ConvertTo-Json -InputObject $manifest -Depth 100

  $cargoManifest = [System.Text.Encoding]::UTF8.GetString($originalCargoManifest)
  if (-not $cargoManifest.Contains('default = []')) {
    throw 'Could not find the default Cargo feature list to enable rust-ui'
  }
  $updatedCargoManifest = $cargoManifest.Replace('default = []', 'default = ["rust-ui"]')

  # Restore the original bytes even when NSH exits with an error or Ctrl-C.
  $restoreManifest = $true
  $restoreCargoManifest = $true
  [System.IO.File]::WriteAllText(
    $manifestPath,
    $updatedManifest + [Environment]::NewLine,
    [System.Text.UTF8Encoding]::new($false)
  )
  [System.IO.File]::WriteAllText(
    $cargoManifestPath,
    $updatedCargoManifest,
    [System.Text.UTF8Encoding]::new($false)
  )

  & nsh run @NshArguments
  if ($null -ne $LASTEXITCODE) { $exitCode = $LASTEXITCODE }
} finally {
  try {
    if ($restoreManifest) {
      [System.IO.File]::WriteAllBytes($manifestPath, $originalManifest)
    }
    if ($restoreCargoManifest) {
      [System.IO.File]::WriteAllBytes($cargoManifestPath, $originalCargoManifest)
    }
  } finally {
    Pop-Location
  }
}

exit $exitCode
