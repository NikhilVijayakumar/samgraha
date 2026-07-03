# ─── Shared Report Utilities (dot-sourced by run-tests.ps1, audit-phase1.ps1) ───

function Get-MetricsJsonPath {
    param([string]$Dir)
    return Join-Path $Dir "metrics.json"
}

function Write-Report {
    param([string]$TemplateName, [string]$OutputName, [string]$ValuesJson)
    $templatePath = Join-Path $Script:TEMPLATE_DIR $TemplateName
    $outputPath   = Join-Path $Script:LATEST_DIR $OutputName
    if (-not (Test-Path $templatePath)) {
        Write-Warning "Template missing: $templatePath -- using inline fallback"
        $content = "# $OutputName`n`n**Status:** {{STATUS}}`n`n{{ERRORS_TABLE}}`n`n{{CHECKS_TABLE}}"
    } else {
        $content = [System.IO.File]::ReadAllText($templatePath)
    }
    $tmpTpl = [System.IO.Path]::GetTempFileName()
    $tmpVals = [System.IO.Path]::GetTempFileName()
    [System.IO.File]::WriteAllText($tmpTpl, $content)
    [System.IO.File]::WriteAllText($tmpVals, $ValuesJson)
    python3 -c @"
import sys, json
with open('$tmpTpl') as f: tpl = f.read()
with open('$tmpVals') as f: vals = json.load(f)
for k, v in vals.items():
    tpl = tpl.replace('{{' + k + '}}', str(v))
with open('$outputPath', 'w') as f: f.write(tpl)
"@
    Remove-Item $tmpTpl, $tmpVals -ErrorAction SilentlyContinue
    Write-Output $outputPath
}

function Get-ChecksTable {
    param([string]$ChecksJson)
    $count = $ChecksJson | jq 'length'
    if ($count -eq 0 -or [string]::IsNullOrEmpty($ChecksJson)) { return "| -- | -- | -- | -- |" }
    return $ChecksJson | jq -r '[
        "| # | Check | Status | Detail |",
        "|---|-------|--------|--------|"
    ] + (to_entries | map(
        "| " + (.key + 1 | tostring) + " | " +
        (.value.Name | gsub("\\|"; "\\|")) + " | " +
        (if .value.Status == "pass" then "✅ " else
         if .value.Status == "fail" then "❌ " else
         if .value.Status == "warn" then "⚠️ " else "⬜ " end end end) +
        (.value.Status) + " | " +
        (.value.Detail[0:80] | gsub("\\|"; "\\|")) + " |"
    )) | join("\n")'
}

function Get-ErrorsTable {
    param([string]$Phase)
    $errs = Get-PhaseErrorsJson $Phase
    $count = $errs | jq 'length'
    if ($count -eq 0) { return "✅ No errors" }
    return $errs | jq -r '[
        "| Tool Call | Error | Response |",
        "|-----------|-------|----------|"
    ] + [.[] | "| " + (.Tool | gsub("\\|"; "\\|")) + " | " +
        (.Error | gsub("\\|"; "\\|")) + " | " +
        (.Response[0:120] | gsub("\\|"; "\\|")) + " |"
    ] | join("\n")'
}

function Add-PhaseError {
    param([string]$Tool, [string]$Err, [string]$Resp)
    $snippet = $Resp
    if ($Resp.Length -gt 250) { $snippet = $Resp.Substring(0, 250) + "..." }
    $prev = $Script:PHASE_ERRORS_JSON
    $Script:PHASE_ERRORS_JSON = $prev | jq -c --arg phase $Script:CURRENT_PHASE --arg tool $Tool --arg err $Err --arg resp $snippet '.[$phase] += [{"Tool": $tool, "Error": $err, "Response": $resp}]'
}

function Get-PhaseErrorsJson {
    param([string]$Phase)
    $Script:PHASE_ERRORS_JSON | jq -c ".[\"$Phase\"] // []"
}

function Load-PreviousMetrics {
    param([string]$ArchiveDir)
    if (-not (Test-Path $ArchiveDir)) { return $false }
    $newest = Get-ChildItem $ArchiveDir -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if (-not $newest) { return $false }
    $mpath = Get-MetricsJsonPath $newest.FullName
    if (-not (Test-Path $mpath)) { return $false }
    $Script:PREV_METRICS = [System.IO.File]::ReadAllText($mpath)
    Write-Host "  Loaded previous metrics from $($newest.FullName)" -ForegroundColor DarkGray
    return $true
}

function Get-PrevMetric {
    param([string]$Phase, [string]$Metric = "score")
    if ([string]::IsNullOrEmpty($Script:PREV_METRICS) -or $Script:PREV_METRICS -eq "{}") { return "" }
    $val = $Script:PREV_METRICS | jq -r --arg p $Phase --arg m $Metric '(.phase_scores // []) | .[] | select(.phase == $p) | .[$m] // ""' 2>$null
    if ([string]::IsNullOrEmpty($val) -or $val -eq "null") { return "" }
    return $val
}

function Trend-Between {
    param([string]$Current, [string]$Previous)
    if ([string]::IsNullOrEmpty($Previous)) { return "—" }
    $c = [double]::Parse($Current)
    $p = [double]::Parse($Previous)
    if ($c -gt $p) { return "↑" }
    elseif ($c -lt $p) { return "↓" }
    else { return "→" }
}

function Format-ScoreLine {
    param([string]$Label, [int]$Score, [string]$Prev, [string]$Status, [int]$Errors)
    $trend = Trend-Between $Score $Prev
    $prevStr = if ([string]::IsNullOrEmpty($Prev)) { "—" } else { "$Prev/100" }
    return "| $Label | $Score/100 | $prevStr | $trend | $Status | $Errors |"
}

function Gen-PhaseAnalysis {
    param([string]$Phase, [string]$Checks)
    if ([string]::IsNullOrEmpty($Checks) -or $Checks -eq "null") { return "No checks data available." }
    $total = $Checks | jq 'length // 0'
    $ok = $Checks | jq '[.[] | select(.Status == "pass")] | length'
    $fail = $Checks | jq '[.[] | select(.Status == "fail")] | length'
    $warn = $Checks | jq '[.[] | select(.Status == "warn")] | length'
    $skip = $Checks | jq '[.[] | select(.Status == "skip")] | length'
    if ($total -eq 0) { return "No checks performed for this phase." }
    if ($fail -gt 0) { $msg = "❌ $fail/$total checks failed. " }
    elseif ($warn -gt 0) { $msg = "⚠️ $warn/$total checks have warnings. " }
    elseif ($skip -gt 0) { $msg = "ℹ️ $skip/$total checks skipped, $ok passed. " }
    else { $msg = "✅ All $total checks passed. " }
    $msg += "$ok passed, $warn warnings, $fail failures."
    if ($skip -gt 0) { $msg += " $skip skipped." }
    return $msg
}

function Gen-PhaseRecs {
    param([string]$Phase, [string]$Checks)
    if ([string]::IsNullOrEmpty($Checks) -or $Checks -eq "null") { return "- No data to generate recommendations." }
    $recs = ""
    $failItems = $Checks | jq -r '[.[] | select(.Status == "fail") | .Name] | join("│")'
    $warnItems = $Checks | jq -r '[.[] | select(.Status == "warn") | .Name] | join("│")'
    if (-not [string]::IsNullOrEmpty($failItems)) {
        foreach ($f in ($failItems -split "│")) {
            $recs += "- 🔴 Fix failing check: $f`n"
        }
    }
    if (-not [string]::IsNullOrEmpty($warnItems)) {
        foreach ($w in ($warnItems -split "│")) {
            $recs += "- 🟡 Address warning: $w`n"
        }
    }
    if ([string]::IsNullOrEmpty($recs)) { $recs = "- ✅ No action required" }
    return $recs.TrimEnd("`n")
}

function Initialize-ReportDirs {
    param([string]$ReportSubdir, [string]$ReportRootDir = "docs\report\manual-audit")
    $Script:TEMPLATE_DIR = Join-Path $Script:RootDir "scripts\templates\$ReportSubdir"
    $Script:LATEST_DIR   = Join-Path $Script:RootDir "$ReportRootDir\$ReportSubdir\latest"
    $Script:ARCHIVE_DIR  = Join-Path $Script:RootDir "$ReportRootDir\$ReportSubdir\archive"
    if (-not (Test-Path $Script:TEMPLATE_DIR)) { New-Item -ItemType Directory -Force $Script:TEMPLATE_DIR | Out-Null }
    if (-not (Test-Path $Script:ARCHIVE_DIR)) { New-Item -ItemType Directory -Force $Script:ARCHIVE_DIR | Out-Null }
    if (Test-Path $Script:LATEST_DIR) {
        $ts = Get-Date -Format "yyyy-MM-dd_HHmmss"
        $archivePath = Join-Path $Script:ARCHIVE_DIR $ts
        Move-Item $Script:LATEST_DIR $archivePath
        Write-Host "Archived previous run → $archivePath"
    }
    New-Item -ItemType Directory -Force $Script:LATEST_DIR | Out-Null
}
