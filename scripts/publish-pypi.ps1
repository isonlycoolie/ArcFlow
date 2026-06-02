# Build (optional) and upload arcflow-sdk to PyPI with your API token.
#
# Token (first match wins):
#   1. -Token "pypi-...."
#   2. $env:PYPI_API_TOKEN
#   3. scripts/.pypi-token  (one line, gitignored — copy from .pypi-token.example)
#   4. Secure prompt (use -SaveToken to write scripts/.pypi-token for next time)
#
# Examples:
#   .\scripts\publish-pypi.ps1
#   .\scripts\publish-pypi.ps1 -SkipBuild
#   .\scripts\publish-pypi.ps1 -Token "pypi-AgEIcHlwaS5vcmcCJ..." -SkipBuild

param(
    [string]$Token,
    [switch]$SkipBuild,
    [switch]$SaveToken
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TokenFile = Join-Path $ScriptDir ".pypi-token"

function Resolve-PypiToken {
    param([string]$FromParam)

    if ($FromParam) { return $FromParam.Trim() }
    if ($env:PYPI_API_TOKEN) { return $env:PYPI_API_TOKEN.Trim() }
    if (Test-Path $TokenFile) {
        foreach ($rawLine in Get-Content $TokenFile) {
            $line = $rawLine.Trim()
            if (-not $line -or $line.StartsWith("#")) { continue }
            if ($line.StartsWith("pypi-")) { return $line }
        }
    }

    $secure = Read-Host "PyPI API token (pypi-...)" -AsSecureString
    $plain = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
        [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    )
    if (-not $plain) { throw "No token entered." }

    if ($SaveToken) {
        Set-Content -Path $TokenFile -Value $plain.Trim() -NoNewline -Encoding utf8
        Write-Host "Saved token to $TokenFile (gitignored)."
    }

    return $plain.Trim()
}

$resolved = Resolve-PypiToken -FromParam $Token
if (-not $resolved.StartsWith("pypi-")) {
    Write-Warning "PyPI tokens usually start with pypi-; continuing anyway."
}

$env:PYPI_API_TOKEN = $resolved
$localArgs = @{}
if ($SkipBuild) { $localArgs.SkipBuild = $true }
& (Join-Path $ScriptDir "publish-pypi-local.ps1") @localArgs
