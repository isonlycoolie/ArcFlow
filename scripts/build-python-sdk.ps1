# Build arcflow-sdk wheel (+ sdist) into sdk-python/dist/
$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptDir
$SdkDir = Join-Path $Root "sdk-python"

if (-not (Test-Path (Join-Path $SdkDir "pyproject.toml"))) {
    Write-Error "Expected sdk-python/pyproject.toml under $Root"
}

$name = (Select-String -Path (Join-Path $SdkDir "pyproject.toml") -Pattern '^name = "(.+)"' | ForEach-Object { $_.Matches.Groups[1].Value })
if ($name -ne "arcflow-sdk") {
    Write-Warning "pyproject name is '$name' (expected arcflow-sdk for PyPI)"
}

Write-Host "Building $name from $SdkDir ..."
Push-Location $SdkDir
try {
    python -m pip install -q maturin
    python -m maturin build --release --out dist
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

$Dist = Join-Path $SdkDir "dist"
$wheelPrefix = $name -replace '-', '_'
$wheels = @(Get-ChildItem -Path $Dist -File -Filter "$wheelPrefix-*.whl")
if ($wheels.Count -eq 0) {
    Write-Host "dist contents:"
    Get-ChildItem -Path $Dist -File | ForEach-Object { Write-Host "  $($_.Name)" }
    Write-Error "No wheel produced in $Dist"
}

Write-Host "Built:"
$wheels | ForEach-Object { Write-Host "  $($_.FullName)" }
