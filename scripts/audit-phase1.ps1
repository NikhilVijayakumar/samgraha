param(
    [switch]$Keep,
    [switch]$Restore
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $PSScriptRoot
$BackupPath = Join-Path $RootDir "samgraha.toml.phase1bak"

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

Initialize-ReportDirs "audit"
Load-PreviousMetrics $Script:ARCHIVE_DIR

$Script:PHASE_ID = "01-phase1"
$Script:PHASE_DURATION = Get-Date
$phaseChecks = New-Object System.Collections.ArrayList

Push-Location $RootDir
try {
    if ($Restore) {
        if (-not (Test-Path $BackupPath)) {
            Write-Host "No backup found at $BackupPath" -ForegroundColor Red
            exit 1
        }
        Move-Item $BackupPath "samgraha.toml" -Force
        Write-Host "Config restored from backup" -ForegroundColor Green
        [void]$phaseChecks.Add(@{Name = "Config restored"; Status = "pass"; Detail = ""})
        [void]$phaseChecks.Add(@{Name = "Phase 1 audit completed"; Status = "pass"; Detail = ""})
        $pStatus = "✅ PASS"
        $pScore = 100
    } else {
        if (Test-Path $BackupPath) {
            Write-Host "WARN stale backup found -- restoring first" -ForegroundColor Yellow
            Move-Item $BackupPath "samgraha.toml" -Force
        }
        Copy-Item "samgraha.toml" $BackupPath
        Write-Host "Config backed up -> samgraha.toml.phase1bak" -ForegroundColor Green
        [void]$phaseChecks.Add(@{Name = "Config backed up"; Status = "pass"; Detail = ""})
        [void]$phaseChecks.Add(@{Name = "Phase 1 audit started"; Status = "pass"; Detail = ""})
        $pStatus = "⬜ STARTED"
        $pScore = 0
        Write-Host "Run Phase 1 commands, then: .\scripts\audit-phase1.ps1 -Restore" -ForegroundColor Cyan
    }
} finally {
    Pop-Location
}

$Script:PHASE_CHECKS["01-phase1"] = $phaseChecks

# Write phase report
$end = Get-Date
$duration = [math]::Round(($end - $Script:PHASE_DURATION).TotalSeconds)
$phaseChecksJson = $phaseChecks | ConvertTo-Json -Compress
$checksTable = Get-ChecksTable $phaseChecksJson
$errorsTable = Get-ErrorsTable "01-phase1"
$analysis = Gen-PhaseAnalysis "01-phase1" $phaseChecksJson
$recs = Gen-PhaseRecs "01-phase1" $phaseChecksJson
$total = $phaseChecks.Count
$ok = ($phaseChecks | Where-Object { $_.Status -eq "pass" }).Count
$fail = ($phaseChecks | Where-Object { $_.Status -eq "fail" }).Count
$prevScore = Get-PrevMetric "01-phase1" "score"
$trend = Trend-Between $pScore $prevScore

$reportVals = @{
    TIMESTAMP        = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    DURATION         = "${duration}s"
    STATUS           = $pStatus
    SCORE            = $pScore
    TREND            = $trend
    PREV_SCORE       = if ([string]::IsNullOrEmpty($prevScore)) { "—" } else { $prevScore }
    ANALYSIS         = $analysis
    RECOMMENDATIONS  = $recs
    CHECKS_TABLE     = $checksTable
    ERRORS_TABLE     = $errorsTable
    PASSES           = $ok
    FAILURES         = $fail
    BACKUP_PATH      = $BackupPath
}
Write-Report "01-phase1.md" "01-phase1.md" ($reportVals | ConvertTo-Json -Depth 5) | Out-Null

if ($Restore) {
    # Save metrics
    $phaseScores = @(@{phase = "01-phase1"; score = $pScore; status = $pStatus; errors = $fail; duration = $duration})
    $metrics = @{
        timestamp    = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        phase_scores = $phaseScores
        total_score  = $pScore
        metrics      = @{passes = $ok; failures = $fail; duration = $duration}
    } | ConvertTo-Json -Depth 5
    [System.IO.File]::WriteAllText((Get-MetricsJsonPath $Script:LATEST_DIR), $metrics, [System.Text.Encoding]::UTF8)

    # Summary
    $phaseRow = "| 01-phase1 | ${pScore}/100 | $pStatus | $fail | ${duration}s |"
    $summaryVals = @{
        TIMESTAMP          = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        STATUS             = $pStatus
        DURATION           = "${duration}s"
        SCORE              = $pScore
        TREND              = $trend
        PREV_SCORE         = if ([string]::IsNullOrEmpty($prevScore)) { "—" } else { $prevScore }
        ANALYSIS           = $analysis
        RECOMMENDATIONS    = $recs
        PHASE_RESULTS_ROWS = $phaseRow
        FAILED_PHASES      = "—"
        PASSES             = $ok
        FAILURES           = $fail
    }
    Write-Report "00-summary.md" "00-summary.md" ($summaryVals | ConvertTo-Json -Depth 5) | Out-Null

    Write-Host "Report files:" -ForegroundColor Cyan
    foreach ($f in (Get-ChildItem "$($Script:LATEST_DIR)\*.md" | Sort-Object Name)) {
        Write-Host "  $($f.FullName)"
    }
}

if ($Keep -and (Test-Path $BackupPath)) {
    Remove-Item $BackupPath -Force
}
