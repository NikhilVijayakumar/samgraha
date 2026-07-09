<#
.SYNOPSIS
    Runs unit + e2e tests with coverage and writes a TestRunReport JSON
    (see crates/schemas/src/test_run.rs, docs/raw/audit/test-report.schema.json)
    for Coverage Audit (CV6) / Implementation Audit (I8) to read via the
    repo's [pipelines.test] contract.

.DESCRIPTION
    Single instrumented `cargo llvm-cov --workspace --json` run covers both
    lib unit tests and the `tests` crate's e2e_*.rs integration tests in one
    pass; stderr carries the normal cargo-test log (parsed for pass/fail),
    stdout carries the coverage JSON (parsed for line coverage %).
#>
param(
    [string]$ReportPath = "docs/report/test-results.json"
)

$ErrorActionPreference = "Continue"
$RootDir = Split-Path -Parent $PSScriptRoot
$stdoutFile = Join-Path $env:TEMP "samgraha-llvmcov-stdout-$PID.json"
$stderrFile = Join-Path $env:TEMP "samgraha-llvmcov-stderr-$PID.log"

function Parse-TestLog {
    param([string[]]$Lines)
    $unit = @{ passed = 0; failed = 0; skipped = 0; failures = @() }
    $e2e  = @{ passed = 0; failed = 0; skipped = 0; failures = @() }
    $bucket = $null
    foreach ($line in $Lines) {
        if ($line -match '^\s*Running unittests\b') { $bucket = $unit; continue }
        if ($line -match '^\s*Running tests[\\/]') { $bucket = $e2e; continue }
        if ($null -eq $bucket) { continue }
        if ($line -match '^test (\S+) \.\.\. FAILED') {
            # ponytail: name only, no scraped panic message - cargo's
            # default pretty format doesn't put the message next to the
            # FAILED line (it's in a separate "failures:" section keyed by
            # test name with its own multi-line backtrace); reliably
            # correlating the two needs a second parsing pass. Upgrade path:
            # add that pass if fix-plan quality on the message text matters.
            $bucket.failures += @{ name = $Matches[1]; message = "" }
            continue
        }
        if ($line -match 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;') {
            $bucket.passed  += [int]$Matches[1]
            $bucket.failed  += [int]$Matches[2]
            $bucket.skipped += [int]$Matches[3]
        }
    }
    return @{ unit = $unit; e2e = $e2e }
}

Push-Location $RootDir
try {
    Write-Host "Running unit+e2e tests under cargo-llvm-cov..."
    & cargo llvm-cov --workspace --json 1>$stdoutFile 2>$stderrFile
    $testExitCode = $LASTEXITCODE
    Get-Content $stderrFile | ForEach-Object { Write-Host $_ }

    $parsed = Parse-TestLog (Get-Content $stderrFile)
    $unit = $parsed.unit
    $e2e = $parsed.e2e
    $unit.total = $unit.passed + $unit.failed + $unit.skipped
    $e2e.total = $e2e.passed + $e2e.failed + $e2e.skipped

    $coveragePercent = $null
    try {
        $covData = Get-Content $stdoutFile -Raw | ConvertFrom-Json
        $coveragePercent = [math]::Round($covData.data[0].totals.lines.percent, 1)
    } catch {
        Write-Host "Warning: could not parse coverage JSON - coverage_percent will be null ($_)"
    }

    $report = @{
        unit = @{
            total = $unit.total; passed = $unit.passed; failed = $unit.failed
            skipped = $unit.skipped; failures = @($unit.failures)
        }
        e2e = @{
            total = $e2e.total; passed = $e2e.passed; failed = $e2e.failed
            skipped = $e2e.skipped; failures = @($e2e.failures)
        }
        coverage_percent = $coveragePercent
    }

    $fullReportPath = Join-Path $RootDir $ReportPath
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $fullReportPath) | Out-Null
    # Windows PowerShell 5.1's `Set-Content -Encoding utf8` writes a UTF-8
    # BOM, which serde_json (and most JSON parsers) treat as invalid leading
    # bytes rather than skipping - write BOM-less UTF-8 explicitly instead.
    $reportJson = $report | ConvertTo-Json -Depth 5
    [System.IO.File]::WriteAllText($fullReportPath, $reportJson, [System.Text.UTF8Encoding]::new($false))

    Write-Host "Wrote $fullReportPath"
    Write-Host "unit: $($unit.passed)/$($unit.total) passed, e2e: $($e2e.passed)/$($e2e.total) passed, coverage: $coveragePercent%"
} finally {
    Pop-Location
    Remove-Item -Force -ErrorAction SilentlyContinue $stdoutFile, $stderrFile
}

if ($unit.failed -gt 0 -or $e2e.failed -gt 0) { exit 1 } else { exit 0 }
