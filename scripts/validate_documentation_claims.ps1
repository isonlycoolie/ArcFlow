# Validate documentation claims (PowerShell)
# Lightweight smoke checks for documentation/ content.
$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$DocRoot = Join-Path $Root "documentation"
$Fail = 0

if (-not (Test-Path $DocRoot)) {
    Write-Error "Missing documentation/ at $DocRoot"
    exit 1
}

$mdCount = (Get-ChildItem -Path $DocRoot -Recurse -Filter "*.md").Count
Write-Host "Found $mdCount markdown files under documentation/"
if ($mdCount -lt 140) {
    Write-Host "WARN: expected ~150+ pages after curriculum enrichment"
}

# Em dash (U+2014)
$emDash = Get-ChildItem -Path $DocRoot -Recurse -Filter "*.md" | Select-String -Pattern [char]0x2014
if ($emDash) {
    Write-Host "FAIL: em dash found in documentation/"
    $emDash | Select-Object -First 5 | ForEach-Object { Write-Host $_.Path }
    $Fail = 1
}

# Horizontal rules on own line
$hr = Get-ChildItem -Path $DocRoot -Recurse -Filter "*.md" | Select-String -Pattern '^---$'
if ($hr) {
    Write-Host "FAIL: horizontal rule (---) found in documentation/"
    $hr | Select-Object -First 5 | ForEach-Object { Write-Host $_.Path }
    $Fail = 1
}

# Getting-started curriculum dirs
$requiredDirs = @(
    "getting-started\fundamentals",
    "getting-started\writing-agents",
    "getting-started\writing-workflows",
    "getting-started\tools",
    "getting-started\memory",
    "getting-started\rag",
    "getting-started\integrating",
    "getting-started\paths"
)
foreach ($d in $requiredDirs) {
    if (-not (Test-Path (Join-Path $DocRoot $d))) {
        Write-Host "MISSING: $d"
        $Fail = 1
    }
}

$requiredFiles = @(
    "getting-started\README.md",
    "getting-started\paths\static-site-chatbot.md",
    "getting-started\fundamentals\01-how-arcflow-thinks.md"
)
foreach ($f in $requiredFiles) {
    if (-not (Test-Path (Join-Path $DocRoot $f))) {
        Write-Host "MISSING: $f"
        $Fail = 1
    }
}

# Core SDK exports
$sdkInit = Join-Path $Root "sdk-python\arcflow\__init__.py"
if (Test-Path $sdkInit) {
    foreach ($sym in @("Agent", "Workflow", "TraceResult", "OpenAI", "Anthropic", "Gemini")) {
        if (-not (Select-String -Path $sdkInit -Pattern $sym -Quiet)) {
            Write-Host "MISSING: $sym in sdk-python __init__"
            $Fail = 1
        }
    }
}

# Server routes documented
$serverDir = Join-Path $DocRoot "server"
if (Test-Path $serverDir) {
    $serverMd = Get-ChildItem -Path $serverDir -Filter "*.md" -Recurse | ForEach-Object { $_.FullName }
    foreach ($route in @('POST /v1/runs', 'GET /v1/runs', '/health', '/ready')) {
        if (-not (Select-String -Path $serverMd -SimpleMatch -Pattern $route -Quiet)) {
            Write-Host "MISSING: server doc reference to $route"
            $Fail = 1
        }
    }
}

# Primary example scripts on disk (current branch names)
foreach ($rel in @(
    "examples\personal\blog_pipeline.py",
    "examples\rag\document_qa.py",
    "examples\external\playwright_stub_callback.py",
    "examples\langchain\migration_demo.py"
)) {
    if (-not (Test-Path (Join-Path $Root $rel))) {
        Write-Host "MISSING example on disk: $rel"
        $Fail = 1
    }
}

if ($Fail -eq 0) {
    Write-Host "validate_documentation_claims: OK"
    exit 0
}
exit 1
