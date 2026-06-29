param(
    [switch]$Full,
    [switch]$WithMCP,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Continue"
$Global:Failures       = 0
$Global:Passes         = 0
$Global:FailureDetails = [System.Collections.Generic.List[object]]::new()
$Global:LastOutput     = @()
$Global:CurrentPhase   = ""
$RootDir  = Split-Path -Parent $PSScriptRoot
$TestTemp = Join-Path $env:TEMP "samgraha-test-$([System.IO.Path]::GetRandomFileName())"

function Write-Pass { Write-Host "  OK $($args -join ' ')" -ForegroundColor Green; $Global:Passes++ }
function Write-Fail {
    $msg = $args -join ' '
    Write-Host "  XX $msg" -ForegroundColor Red
    $Global:Failures++
    $Global:FailureDetails.Add([PSCustomObject]@{
        Phase  = $Global:CurrentPhase
        Test   = $msg
        Output = ($Global:LastOutput -join "`n").Trim()
    })
}
function Write-Step {
    $Global:CurrentPhase = $args -join ' '
    $Global:LastOutput   = @()
    Write-Host "`n== $Global:CurrentPhase ==" -ForegroundColor Cyan
}
function Write-Info { Write-Host "  .. $($args -join ' ')" -ForegroundColor DarkGray }

function Assert-ExitCodeZero {
    param($Message)
    $ec = $global:LASTEXITCODE
    if ($ec -ne 0) {
        Write-Fail "$Message (exit $ec)"
    } else {
        Write-Pass "$Message"
    }
}

function Assert-FileExists {
    param($Path, $Message)
    if (-not (Test-Path $Path)) {
        Write-Fail "$Message -- file not found: $Path"
    } else {
        Write-Pass "$Message"
    }
}

function Run-Cli {
    param([string[]]$CliArgs)
    $out = & cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- @CliArgs 2>&1
    $Global:LastOutput = $out
    return $out
}

function New-TestFixture {
    param([string]$Path, [string]$RepoId = "test-repo")
    [System.IO.Directory]::CreateDirectory("$Path\docs\architecture") | Out-Null
    [System.IO.Directory]::CreateDirectory("$Path\docs\feature") | Out-Null
    [System.IO.Directory]::CreateDirectory("$Path\docs\engineering") | Out-Null
    [System.IO.File]::WriteAllText("$Path\samgraha.toml",
        "[repository]`nid = `"$RepoId`"`nname = `"$RepoId test`"")
    [System.IO.File]::WriteAllText("$Path\docs\architecture\system-overview.md",
        "# System Overview`n`n## Purpose`n`nText.`n`n## Constraints`n`n- Offline`n- Deterministic")
    [System.IO.File]::WriteAllText("$Path\docs\feature\knowledge-compilation.md",
        "# Compilation`n`n## Purpose`n`nTransform docs.`n`n## Requirements`n`n- FTS`n- Progressive")
    [System.IO.File]::WriteAllText("$Path\docs\engineering\build-system.md",
        "# Build`n`n## Purpose`n`nBuild workflows.`n`n## Toolchain`n`n- Cargo`n- Rust analyzer")
}

function Remove-TestFixture {
    param([string]$Path)
    if (Test-Path $Path) { Remove-Item -Recurse -Force $Path -ErrorAction SilentlyContinue }
}

function Invoke-Phase1a {
    Write-Step "Phase 1a - Unit Tests"
    Push-Location $RootDir
    try {
        Write-Info "Running cargo test -p tests"
        $Global:LastOutput = & cargo test -p tests 2>&1
        $Global:LastOutput | ForEach-Object { Write-Host $_ }
        Assert-ExitCodeZero "cargo test -p tests"
    } finally { Pop-Location }
}

function Invoke-Phase1b {
    Write-Step "Phase 1b - CLI Integration"
    $testDir = Join-Path $TestTemp "p1b"
    New-TestFixture $testDir "test-repo"
    Push-Location $testDir
    try {
        Write-Info "1. compile"
        Run-Cli @("compile") | Out-Null
        Assert-ExitCodeZero "compile exits 0"
        Assert-FileExists ".samgraha\knowledge.db" "knowledge.db"
        Assert-FileExists ".samgraha\manifest.json" "manifest.json"

        $m = Get-Content ".samgraha\manifest.json" | ConvertFrom-Json
        if ($m.revision -ge 1) { Write-Pass "manifest revision >= 1" } else { Write-Fail "revision >= 1" }
        if ($m.audit.status) { Write-Pass "audit status present" } else { Write-Fail "audit status" }
        if ($m.repository.uuid) { Write-Pass "UUID present" } else { Write-Fail "UUID" }

        Write-Info "2. recompile - revision unchanged"
        Run-Cli @("compile") | Out-Null
        $m2 = Get-Content ".samgraha\manifest.json" | ConvertFrom-Json
        if ($m.revision -eq $m2.revision) { Write-Pass "revision unchanged" } else { Write-Fail "revision changed" }

        Write-Info "3. registry register"
        Run-Cli @("registry", "register") | Out-Null
        Assert-ExitCodeZero "register"

        Write-Info "4. registry list"
        $out = Run-Cli @("registry", "list")
        Assert-ExitCodeZero "list"
        if ($out -match "test-repo") { Write-Pass "repo in list" } else { Write-Fail "repo in list" }

        Write-Info "5. registry status"
        $out = Run-Cli @("registry", "status")
        Assert-ExitCodeZero "status"

        Write-Info "6. search"
        Run-Cli @("search", "compilation") | Out-Null
        Assert-ExitCodeZero "search"

        Write-Info "7. sections"
        Run-Cli @("sections", "purpose") | Out-Null
        Assert-ExitCodeZero "sections"

        Write-Info "8. audit"
        Run-Cli @("audit") | Out-Null
        Assert-ExitCodeZero "audit"

        Write-Info "9. info"
        Run-Cli @("info") | Out-Null
        Assert-ExitCodeZero "info"

        Write-Info "10. resolve"
        Run-Cli @("registry", "resolve", "runtime") | Out-Null
        Assert-ExitCodeZero "resolve"

        Write-Info "11. registry sync"
        Run-Cli @("registry", "sync") | Out-Null
        Assert-ExitCodeZero "sync"
    } finally {
        Pop-Location
        Remove-TestFixture $testDir
    }
}

function Invoke-Phase1c {
    Write-Step "Phase 1c - Multi-Repo"
    $rA = Join-Path (Join-Path $TestTemp "p1c") "repo-a"
    $rB = Join-Path (Join-Path $TestTemp "p1c") "repo-b"
    New-TestFixture $rA "repo-a"
    New-TestFixture $rB "repo-b"
    try {
        Push-Location $rA
        Run-Cli @("compile") | Out-Null
        Run-Cli @("registry", "register") | Out-Null
        Pop-Location
        Push-Location $rB
        Run-Cli @("compile") | Out-Null
        Run-Cli @("registry", "register") | Out-Null
        Pop-Location
        $cfg = Get-Content "$rA\samgraha.toml"
        $depPath = $rB -replace '\\', '/'
        $cfg = $cfg + "`n[[repository.dependencies]]`nname = `"repo-b`"`npath = `"$depPath`"`nrequired = true"
        Set-Content -Path "$rA\samgraha.toml" -Value $cfg
        Push-Location $rA
        Write-Info "resolve with dependency"
        Run-Cli @("registry", "resolve", "runtime") | Out-Null
        Assert-ExitCodeZero "resolve with dep"
        Pop-Location
        Remove-TestFixture $rB
        Push-Location $rA
        Write-Info "resolve with missing dep"
        Run-Cli @("registry", "resolve", "runtime") *>$null
        if ($LASTEXITCODE -ne 0) { Write-Pass "missing dep => non-zero exit" } else { Write-Fail "missing dep should fail" }
        Pop-Location
    } finally {
        Remove-TestFixture $rA
        Remove-TestFixture $rB
    }
}

function Invoke-Phase2 {
    Write-Step "Phase 2 - MCP Tests"
    $testDir = Join-Path $TestTemp "p2"
    New-TestFixture $testDir "mcp-test"
    Push-Location $testDir
    try {
        Run-Cli @("compile") | Out-Null
        function RawMcp($json) {
            $out = $json | & cargo run --manifest-path "$RootDir\Cargo.toml" --bin mcp 2>&1
            $Global:LastOutput = $out
            return $out
        }
        Write-Info "tools/list"
        $r = RawMcp '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
        if ($r -match "compile") { Write-Pass "tools/list" } else { Write-Fail "tools/list" }
        Write-Info "tools/call search"
        $r = RawMcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"compilation"}}}'
        if ($LASTEXITCODE -eq 0) { Write-Pass "search" } else { Write-Fail "search" }
        Write-Info "tools/call get_document"
        $r = RawMcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_document","arguments":{"id":"1"}}}'
        if ($LASTEXITCODE -eq 0) { Write-Pass "get_document" } else { Write-Fail "get_document" }
        Write-Info "tools/call nonexistent"
        $r = RawMcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent"}}'
        if ($r -match "error" -or $r -match "not found") { Write-Pass "nonexistent => error" } else { Write-Fail "nonexistent should error" }
    } finally {
        Pop-Location
        Remove-TestFixture $testDir
    }
}

function Invoke-Phase25 {
    Write-Step "Phase 2.5 - Protocol"
    $testDir = Join-Path $TestTemp "p25"
    New-TestFixture $testDir "proto-test"
    Push-Location $testDir
    try {
        Run-Cli @("compile") | Out-Null
        function RawMcp($json) {
            $out = $json | & cargo run --manifest-path "$RootDir\Cargo.toml" --bin mcp 2>&1
            $Global:LastOutput = $out
            return $out
        }
        Write-Info "malformed JSON"
        $r = RawMcp "not json"
        if ($r -match "-32700") { Write-Pass "parse error" } else { Write-Fail "expected parse error" }

        Write-Info "unknown method"
        $r = RawMcp '{"jsonrpc":"2.0","id":1,"method":"bogus","params":{}}'
        if ($r -match "-32601") { Write-Pass "method not found" } else { Write-Fail "expected method not found" }

        Write-Info "missing tool name"
        $r = RawMcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{}}'
        if ($r -match "-32602") { Write-Pass "invalid params" } else { Write-Fail "expected invalid params" }

        Write-Info "initialize"
        $r = RawMcp '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
        if ($r -match "samgraha-mcp") { Write-Pass "initialize" } else { Write-Fail "expected serverInfo" }

        Write-Info "tools/list"
        $r = RawMcp '{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}'
        if ($r -match "compile") { Write-Pass "tools/list" } else { Write-Fail "expected tools" }

        Write-Info "rapid calls"
        $ok = $true
        1..5 | ForEach-Object {
            $r = RawMcp '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
            if (-not ($r -match "samgraha-mcp")) { $ok = $false }
        }
        if ($ok) { Write-Pass "rapid calls" } else { Write-Fail "rapid calls" }
    } finally {
        Pop-Location
        Remove-TestFixture $testDir
    }
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
Write-Host "Samgraha Test Runner" -ForegroundColor Cyan
Write-Host "Root: $RootDir" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Step "Building"
    Push-Location $RootDir
    $Global:LastOutput = & cargo build --bin cli 2>&1
    if ($LASTEXITCODE -ne 0) { Write-Fail "build cli"; Pop-Location; exit 1 }
    Write-Pass "cli built"
    if ($WithMCP) {
        $Global:LastOutput = & cargo build --bin mcp 2>&1
        if ($LASTEXITCODE -ne 0) { Write-Fail "build mcp"; Pop-Location; exit 1 }
        Write-Pass "mcp built"
    }
    Pop-Location
}

try {
    Invoke-Phase1a
    Invoke-Phase1b
    if ($Full) { Invoke-Phase1c }
    if ($WithMCP) { Invoke-Phase2; Invoke-Phase25 }
} finally {
    Remove-TestFixture $TestTemp
}

$sw.Stop()
$s = "{0:F1}s" -f $sw.Elapsed.TotalSeconds
Write-Host "Passed: $Global:Passes  Failed: $Global:Failures  Time: $s" -ForegroundColor Cyan

# Save report to docs/report/ (gitignored)
$reportDir = Join-Path $RootDir "docs\report\manual-audit"
if (-not (Test-Path $reportDir)) { New-Item -ItemType Directory -Force $reportDir | Out-Null }
$ts        = Get-Date -Format "yyyyMMdd-HHmmss"
$modeParts = @(); if ($Full) { $modeParts += "full" }; if ($WithMCP) { $modeParts += "mcp" }
$mode      = if ($modeParts.Count) { $modeParts -join "-" } else { "default" }
$reportPath = Join-Path $reportDir "$ts-$mode.md"

$sb = [System.Text.StringBuilder]::new()
[void]$sb.AppendLine("# Samgraha Test Report")
[void]$sb.AppendLine("")
[void]$sb.AppendLine("**Date:** $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')")
[void]$sb.AppendLine("**Mode:** $mode")
[void]$sb.AppendLine("**Result:** $(if ($Global:Failures) { 'FAIL' } else { 'PASS' }) -- $Global:Passes passed, $Global:Failures failed")
[void]$sb.AppendLine("**Duration:** $s")
[void]$sb.AppendLine("")

if ($Global:FailureDetails.Count) {
    [void]$sb.AppendLine("## Failure Summary")
    [void]$sb.AppendLine("")
    [void]$sb.AppendLine("| # | Phase | Test |")
    [void]$sb.AppendLine("|---|-------|------|")
    $i = 1
    foreach ($f in $Global:FailureDetails) {
        [void]$sb.AppendLine("| $i | $($f.Phase) | $($f.Test) |")
        $i++
    }
    [void]$sb.AppendLine("")
    [void]$sb.AppendLine("## Failure Details")
    [void]$sb.AppendLine("")
    $i = 1
    foreach ($f in $Global:FailureDetails) {
        [void]$sb.AppendLine("### $i. $($f.Phase): $($f.Test)")
        [void]$sb.AppendLine("")
        if ($f.Output) {
            [void]$sb.AppendLine('```')
            [void]$sb.AppendLine($f.Output)
            [void]$sb.AppendLine('```')
            [void]$sb.AppendLine("")
        }
        $i++
    }
}

[System.IO.File]::WriteAllText($reportPath, $sb.ToString(), [System.Text.Encoding]::UTF8)
Write-Host "Report: $reportPath" -ForegroundColor Cyan

if ($Global:Failures -gt 0) { exit 1 } else { exit 0 }
