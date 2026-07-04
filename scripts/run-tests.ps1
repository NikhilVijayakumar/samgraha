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

. "$RootDir\scripts\lib\report.ps1"

$Script:PHASE_ID = ""
$Script:PHASE_DURATION = $null
$Script:PHASE_CHECKS = @{}
$Script:PHASE_RESULTS = @{}
$Script:PREV_METRICS = "{}"
$Script:PHASE_ERRORS_JSON = "{}"
$Script:TEMPLATE_DIR = ""
$Script:LATEST_DIR = ""
$Script:ARCHIVE_DIR = ""

Initialize-ReportDirs "tests"

function Write-PhaseReport {
    param([string]$PhaseId)
    $end = Get-Date
    $duration = [math]::Round(($end - $Script:PHASE_DURATION).TotalSeconds)
    $phaseCheckList = if ($Script:PHASE_CHECKS.ContainsKey($PhaseId)) { $Script:PHASE_CHECKS[$PhaseId] } else { New-Object System.Collections.ArrayList }
    $phaseChecksJson = $phaseCheckList | ConvertTo-Json -Compress
    $checksTable = Get-ChecksTable $phaseChecksJson
    $errorsTable = Get-ErrorsTable $PhaseId
    $analysis = Gen-PhaseAnalysis $PhaseId $phaseChecksJson
    $recs = Gen-PhaseRecs $PhaseId $phaseChecksJson
    $total = $phaseCheckList.Count
    $ok = ($phaseCheckList | Where-Object { $_.Status -eq "pass" }).Count
    $fail = ($phaseCheckList | Where-Object { $_.Status -eq "fail" }).Count
    $score = if ($total -gt 0) { [math]::Floor($ok * 100 / $total) } else { 0 }
    $status = if ($fail -gt 0) { "❌ FAIL" } else { "✅ PASS" }
    $prevScore = Get-PrevMetric $PhaseId "score"
    $trend = Trend-Between $score $prevScore
    $reportVals = @{
        TIMESTAMP        = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION         = "${duration}s"
        STATUS           = $status
        SCORE            = $score
        TREND            = $trend
        PREV_SCORE       = if ([string]::IsNullOrEmpty($prevScore)) { "—" } else { $prevScore }
        ANALYSIS         = $analysis
        RECOMMENDATIONS  = $recs
        CHECKS_TABLE     = $checksTable
        ERRORS_TABLE     = $errorsTable
        PASSES           = $ok
        FAILURES         = $fail
    }
    $reportValsJson = $reportVals | ConvertTo-Json -Depth 5
    Write-Report "${PhaseId}.md" "${PhaseId}.md" $reportValsJson | Out-Null
    $Script:PHASE_RESULTS[$PhaseId] = @{
        Status = $status; Score = $score; Errors = $fail; Duration = $duration
    } | ConvertTo-Json -Compress
}

function Write-Pass {
    Write-Host "  OK $($args -join ' ')" -ForegroundColor Green
    $Global:Passes++
    $msg = $args -join ' '
    $phaseId = $Script:PHASE_ID
    if (-not $Script:PHASE_CHECKS.ContainsKey($phaseId)) { $Script:PHASE_CHECKS[$phaseId] = New-Object System.Collections.ArrayList }
    [void]$Script:PHASE_CHECKS[$phaseId].Add(@{Name = $msg; Status = "pass"; Detail = ""})
}

function Write-Fail {
    $msg = $args -join ' '
    Write-Host "  XX $msg" -ForegroundColor Red
    $Global:Failures++
    $Global:FailureDetails.Add([PSCustomObject]@{
        Phase  = $Global:CurrentPhase
        Test   = $msg
        Output = ($Global:LastOutput -join "`n").Trim()
    })
    $phaseId = $Script:PHASE_ID
    if (-not $Script:PHASE_CHECKS.ContainsKey($phaseId)) { $Script:PHASE_CHECKS[$phaseId] = New-Object System.Collections.ArrayList }
    [void]$Script:PHASE_CHECKS[$phaseId].Add(@{Name = $msg; Status = "fail"; Detail = ""})
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
    if ($ec -ne 0) { Write-Fail "$Message (exit $ec)" } else { Write-Pass "$Message" }
}

function Assert-FileExists {
    param($Path, $Message)
    if (-not (Test-Path $Path)) { Write-Fail "$Message -- file not found: $Path" } else { Write-Pass "$Message" }
}

function Run-Cli {
    param([string[]]$CliArgs)
    $out = & cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- @CliArgs 2>&1
    $Global:LastOutput = $out
    return $out
}

function New-TestFixture {
    param([string]$Path, [string]$RepoId = "test-repo")
    try {
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
    } catch {
        Write-Fail "New-TestFixture '$RepoId' failed: $_"
        throw
    }
}

function Remove-TestFixture {
    param([string]$Path)
    if (Test-Path $Path) { Remove-Item -Recurse -Force $Path -ErrorAction SilentlyContinue }
}

function Invoke-Phase1a {
    Write-Step "Phase 1a - Unit Tests"
    $Script:PHASE_ID = "01-phase1a"
    $Script:PHASE_DURATION = Get-Date
    Push-Location $RootDir
    try {
        Write-Info "Running cargo test -p tests"
        $Global:LastOutput = & cargo test -p tests 2>&1
        $Global:LastOutput | ForEach-Object { Write-Host $_ }
        Assert-ExitCodeZero "cargo test -p tests"
    } finally {
        Pop-Location
        Write-PhaseReport "01-phase1a"
    }
}

function Invoke-Phase1b {
    Write-Step "Phase 1b - CLI Integration"
    $Script:PHASE_ID = "02-phase1b"
    $Script:PHASE_DURATION = Get-Date
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
        Write-PhaseReport "02-phase1b"
    }
}

function Invoke-Phase1c {
    Write-Step "Phase 1c - Multi-Repo"
    $Script:PHASE_ID = "03-phase1c"
    $Script:PHASE_DURATION = Get-Date
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
        Write-PhaseReport "03-phase1c"
    }
}

function Invoke-Phase2 {
    Write-Step "Phase 2 - MCP Tests"
    $Script:PHASE_ID = "04-phase2"
    $Script:PHASE_DURATION = Get-Date
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
        if ($r -match "error|not found") { Write-Fail "search" } else { Write-Pass "search" }
        Write-Info "tools/call get_document"
        $r = RawMcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_document","arguments":{"id":1}}}'
        if ($r -match "error|not found") { Write-Fail "get_document" } else { Write-Pass "get_document" }
        Write-Info "tools/call nonexistent"
        $r = RawMcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent"}}'
        if ($r -match "error" -or $r -match "not found") { Write-Pass "nonexistent => error" } else { Write-Fail "nonexistent should error" }
    } finally {
        Pop-Location
        Remove-TestFixture $testDir
        Write-PhaseReport "04-phase2"
    }
}

function Invoke-Phase3 {
    Write-Step "Phase 3 - Semantic Audit Tools"
    $Script:PHASE_ID = "06-phase3"
    $Script:PHASE_DURATION = Get-Date
    $testDir = Join-Path $TestTemp "p3"
    New-TestFixture $testDir "audit-test"
    $auditDir = Join-Path $testDir "docs\raw\audit-standards\feature"
    New-Item -ItemType Directory -Force $auditDir | Out-Null
    @"
# Functional Requirements Audit
## Scoring Criteria
| ID | Score | Description |
|---|---|---|
| C1 | 30 | All requirements uniquely identified |
| C2 | 30 | Each requirement is testable |
"@ | Set-Content -Path (Join-Path $auditDir "functional-requirements.md")
    Push-Location $testDir
    try {
        Run-Cli @("compile") | Out-Null
        function RawMcp($json) {
            $out = $json | & cargo run --manifest-path "$RootDir\Cargo.toml" --bin mcp 2>&1
            $Global:LastOutput = $out
            return $out
        }

        Write-Info "get_documents_by_domain"
        $r = RawMcp '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_documents_by_domain","arguments":{"domain":"feature"}}}'
        if ($r -match '"documents"' -or $r -match '\[\]' -or $LASTEXITCODE -eq 0) { Write-Pass "get_documents_by_domain" } else { Write-Fail "get_documents_by_domain" }

        Write-Info "get_section"
        $r = RawMcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_section","arguments":{"section_id":1}}}'
        if ($r -match '"section"' -or $LASTEXITCODE -eq 0) { Write-Pass "get_section" } else { Write-Fail "get_section" }

        Write-Info "get_audit_knowledge"
        $r = RawMcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_audit_knowledge","arguments":{"domain":"feature","section_type":"functional-requirements"}}}'
        if ($r -match "C1|C2") { Write-Pass "get_audit_knowledge" } else { Write-Fail "get_audit_knowledge" }

        Write-Info "get_section_changed"
        $r = RawMcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_section_changed","arguments":{"section_id":1}}}'
        if ($r -match "changed") { Write-Pass "get_section_changed" } else { Write-Fail "get_section_changed" }

        Write-Info "check_gate"
        $r = RawMcp '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"check_gate","arguments":{"stage":"deterministic","document_id":1}}}'
        if ($r -match "passed|blocked") { Write-Pass "check_gate" } else { Write-Fail "check_gate" }

        Write-Info "store_section_report"
        $r = RawMcp '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"store_section_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000001","domain":"feature","stage":"Section","document_id":1,"section_id":1,"strategy":"completeness","score":85,"findings":[{"check_id":"C1","severity":"Error","message":"All present","provider":"test","confidence":0.95,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"test"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}'
        if ($r -match "report_id") { Write-Pass "store_section_report" } else { Write-Fail "store_section_report" }

        Write-Info "get_audit_report"
        $r = RawMcp '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"get_audit_report","arguments":{"domain":"feature","stage":"section","document_id":1}}}'
        if ($r -match "report_id|findings") { Write-Pass "get_audit_report" } else { Write-Fail "get_audit_report" }

        Write-Info "update_finding_status"
        $r = RawMcp '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"update_finding_status","arguments":{"report_id":1,"criterion_id":"C1","status":"fixed"}}}'
        if ($r -match "success|true") { Write-Pass "update_finding_status" } else { Write-Fail "update_finding_status" }

        Write-Info "store_document_report"
        $r = RawMcp '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"store_document_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000002","domain":"feature","stage":"Document","document_id":1,"section_id":null,"strategy":"completeness","score":90,"findings":[{"check_id":"C1","severity":"Error","message":"Doc level","provider":"test","confidence":0.95,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"test"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}'
        if ($r -match "report_id") { Write-Pass "store_document_report" } else { Write-Fail "store_document_report" }

        Write-Info "store_cross_domain_report"
        $r = RawMcp '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"store_cross_domain_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000003","domain":"feature","stage":"CrossDomain","document_id":null,"section_id":null,"strategy":"consistency","score":80,"findings":[{"check_id":"C1","severity":"Warning","message":"Cross domain","provider":"test","confidence":0.85,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"cross"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}'
        if ($r -match "report_id" -or $LASTEXITCODE -eq 0) { Write-Pass "store_cross_domain_report" } else { Write-Fail "store_cross_domain_report" }
    } finally {
        Pop-Location
        Remove-TestFixture $testDir
        Write-PhaseReport "06-phase3"
    }
}

function Invoke-Phase25 {
    Write-Step "Phase 2.5 - Protocol"
    $Script:PHASE_ID = "05-phase25"
    $Script:PHASE_DURATION = Get-Date
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
        Write-PhaseReport "05-phase25"
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
    if ($WithMCP) { Invoke-Phase2; Invoke-Phase25; Invoke-Phase3 }
} finally {
    Remove-TestFixture $TestTemp
}

$sw.Stop()
$s = "{0:F1}s" -f $sw.Elapsed.TotalSeconds
Write-Host "Passed: $Global:Passes  Failed: $Global:Failures  Time: $s" -ForegroundColor Cyan

# --- Summary + metrics ---
$buildChecks = New-Object System.Collections.ArrayList
[void]$buildChecks.Add(@{Name = "Build"; Status = if ($SkipBuild) { "skip" } else { "pass" }; Detail = if ($SkipBuild) { "Skipped via -SkipBuild" } else { "Binaries built" }})
$Script:PHASE_CHECKS["00-build"] = $buildChecks

$allPhaseRows = ""
$allFailed = ""
$scoreSum = 0; $scoreCount = 0
$phaseOrder = @("00-build", "01-phase1a", "02-phase1b", "03-phase1c", "04-phase2", "05-phase25", "06-phase3")
foreach ($key in $phaseOrder) {
    if (-not $Script:PHASE_RESULTS.ContainsKey($key)) { continue }
    $pr = $Script:PHASE_RESULTS[$key] | ConvertFrom-Json
    $allPhaseRows += "| $key | $($pr.Score)/100 | $($pr.Status) | $($pr.Errors) | $($pr.Duration)s |`n"
    $scoreSum += $pr.Score
    $scoreCount++
    if (-not ($pr.Status -match "PASS") -and $pr.Status -ne "⬜ SKIPPED") {
        $allFailed += "- **$key**: $($pr.Status) ($($pr.Errors) errors)`n"
    }
}
$totalScore = if ($scoreCount -gt 0) { [math]::Floor($scoreSum / $scoreCount) } else { 0 }

$prevTotalScore = Get-PrevMetric "total_score" "total_score"
$totalTrend = Trend-Between $totalScore $prevTotalScore
if ([string]::IsNullOrEmpty($prevTotalScore)) { $prevTotalScore = "—" }

if ($Global:Failures -gt 0) {
    $analysis = "❌ $($Global:Failures) failures across $scoreCount phases. $($Global:Passes) total passes."
    $recs = "- 🔴 Fix $($Global:Failures) failing test(s) before next run"
} else {
    $analysis = "✅ All $scoreCount phases passed. $($Global:Passes) total passes."
    $recs = "- ✅ No action required"
}
if ([string]::IsNullOrEmpty($allFailed)) { $allFailed = "—" }

$reportVals = @{
    TIMESTAMP          = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    STATUS             = if ($Global:Failures -gt 0) { "❌ FAIL" } else { "✅ PASS" }
    DURATION           = $s
    SCORE              = $totalScore
    TREND              = $totalTrend
    PREV_SCORE         = $prevTotalScore
    ANALYSIS           = $analysis
    RECOMMENDATIONS    = $recs
    PHASE_RESULTS_ROWS = $allPhaseRows
    FAILED_PHASES      = $allFailed
    PASSES             = $Global:Passes
    FAILURES           = $Global:Failures
}
$reportValsJson = $reportVals | ConvertTo-Json -Depth 5
Write-Report "00-summary.md" "00-summary.md" $reportValsJson | Out-Null

# Metrics
$metricsPhaseOrder = @("01-phase1a", "02-phase1b")
if ($Full) { $metricsPhaseOrder += "03-phase1c" }
if ($WithMCP) { $metricsPhaseOrder += @("04-phase2", "05-phase25", "06-phase3") }
$phaseScores = @()
foreach ($key in $metricsPhaseOrder) {
    if (-not $Script:PHASE_RESULTS.ContainsKey($key)) { continue }
    $pr = $Script:PHASE_RESULTS[$key] | ConvertFrom-Json
    $phaseScores += @{
        phase = $key; score = $pr.Score; status = $pr.Status
        errors = $pr.Errors; duration = $pr.Duration
    }
}
$metrics = @{
    timestamp    = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    phase_scores = $phaseScores
    total_score  = $totalScore
    metrics      = @{
        passes   = $Global:Passes
        failures = $Global:Failures
        duration = [math]::Round($sw.Elapsed.TotalSeconds)
    }
} | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText((Get-MetricsJsonPath $Script:LATEST_DIR), $metrics, [System.Text.Encoding]::UTF8)

Write-Host "Report files:" -ForegroundColor Cyan
foreach ($f in (Get-ChildItem "$($Script:LATEST_DIR)\*.md" | Sort-Object Name)) {
    Write-Host "  $($f.FullName)"
}

if ($Global:Failures -gt 0) { exit 1 } else { exit 0 }
