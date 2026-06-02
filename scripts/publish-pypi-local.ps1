# Build (optional) and upload arcflow-sdk to PyPI.
# Usage:
#   .\scripts\publish-pypi-local.ps1              # build + upload
#   .\scripts\publish-pypi-local.ps1 -SkipBuild   # upload only
#   $env:PYPI_API_TOKEN = "pypi-...."             # required for upload

param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Resolve-Path (Join-Path $ScriptDir "..")
$SdkDir = Join-Path $Root "sdk-python"
$Dist = Join-Path $SdkDir "dist"

if (-not $SkipBuild) {
    & (Join-Path $ScriptDir "build-python-sdk.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-Path $Dist)) {
    Write-Error "Missing $Dist - run without -SkipBuild or rebuild via scripts/build-python-sdk.ps1"
}

# Wheel tag uses underscore (PEP 427): arcflow-sdk -> arcflow_sdk-*.whl
$artifactPaths = [System.Collections.Generic.List[string]]::new()
foreach ($pattern in @("arcflow_sdk-*.whl", "arcflow_sdk-*.tar.gz")) {
    Get-ChildItem -Path $Dist -File -Filter $pattern -ErrorAction SilentlyContinue | ForEach-Object {
        $artifactPaths.Add($_.FullName)
    }
}

if ($artifactPaths.Count -eq 0) {
    Write-Host "Contents of $Dist :"
    Get-ChildItem -Path $Dist -File -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "  $($_.Name)"
    }
    Write-Error "No arcflow_sdk-*.whl found. Build failed or wrong package name in pyproject.toml."
}

if (-not $env:PYPI_API_TOKEN) {
    Write-Error 'Set PYPI_API_TOKEN before upload (PyPI token scoped to arcflow-sdk).'
}

Write-Host "Uploading to PyPI (project arcflow-sdk):"
foreach ($p in $artifactPaths) {
    Write-Host "  $([System.IO.Path]::GetFileName($p))"
}

python -m pip install -q twine
$env:TWINE_USERNAME = "__token__"
$env:TWINE_PASSWORD = $env:PYPI_API_TOKEN
& python -m twine upload --non-interactive @($artifactPaths.ToArray())
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host 'Upload OK. Verify: pip install "arcflow-sdk==0.3.3"'
