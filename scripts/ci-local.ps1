# Windows wrapper for bash CI scripts (Git Bash or WSL).
# Usage from repo root (PowerShell):
#   .\scripts\ci-local.ps1
#   .\scripts\ci-local.ps1 -Full
#   .\scripts\ci-local.ps1 -Full -SkipPostgres

param(
    [switch]$Full,
    [switch]$SkipPostgres,
    [switch]$SkipStaticE2e,
    [switch]$SkipIntegrationMemory,
    [switch]$SkipSdkPython
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Find-Bash {
    $candidates = @(
        "$env:ProgramFiles\Git\bin\bash.exe",
        "$env:ProgramFiles\Git\usr\bin\bash.exe",
        "$env:ProgramFiles(x86)\Git\bin\bash.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) { return $path }
    }
    $wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
    if ($wsl) { return "wsl.exe" }
    throw "Git Bash or WSL not found. Install Git for Windows or enable WSL, then retry."
}

$bash = Find-Bash
$script = if ($Full) { "scripts/ci-local-full.sh" } else { "scripts/ci-local.sh" }

$envVars = @()
if ($SkipPostgres) { $envVars += "CI_SKIP_POSTGRES=1" }
if ($SkipStaticE2e) { $envVars += "CI_SKIP_STATIC_E2E=1" }
if ($SkipIntegrationMemory) { $envVars += "CI_SKIP_INTEGRATION_MEMORY=1" }
if ($SkipSdkPython) { $envVars += "CI_SKIP_SDK_PYTHON=1" }

if ($bash -eq "wsl.exe") {
    $export = ($envVars | ForEach-Object { "export $_;" }) -join " "
    & wsl.exe bash -lc "$export cd '$(($Root -replace '\\', '/'))' && bash $script"
} else {
    foreach ($pair in $envVars) {
        $name, $value = $pair -split "=", 2
        Set-Item -Path "env:$name" -Value $value
    }
    & $bash $script
}
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
