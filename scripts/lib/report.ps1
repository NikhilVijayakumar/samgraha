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
        $content = [System.IO.File]::ReadAllText($templatePath, [System.Text.Encoding]::UTF8)
    }
    $vals = $ValuesJson | ConvertFrom-Json
    foreach ($prop in $vals.PSObject.Properties) {
        $content = $content.Replace("{{$($prop.Name)}}", [string]$prop.Value)
    }
    [System.IO.File]::WriteAllText($outputPath, $content, [System.Text.Encoding]::UTF8)
    Write-Output $outputPath
}

function Get-ChecksTable {
    param([string]$ChecksJson)
    if ([string]::IsNullOrEmpty($ChecksJson) -or $ChecksJson -eq "null" -or $ChecksJson -eq "[]") { return "| -- | -- | -- | -- |" }
    $items = $ChecksJson | ConvertFrom-Json
    if ($items.Count -eq 0) { return "| -- | -- | -- | -- |" }
    $rows = @("| # | Check | Status | Detail |", "|---|-------|--------|--------|")
    for ($i = 0; $i -lt $items.Count; $i++) {
        $item = $items[$i]
        $icon = if ($item.Status -eq "pass") { "✅ " } elseif ($item.Status -eq "fail") { "❌ " } elseif ($item.Status -eq "warn") { "⚠️ " } else { "⬜ " }
        $detail = if ($item.Detail -and $item.Detail.Length -gt 80) { $item.Detail.Substring(0,80) } else { $item.Detail }
        $rows += "| $($i+1) | $($item.Name) | $icon$($item.Status) | $detail |"
    }
    return $rows -join "`n"
}

function Get-ErrorsTable {
    param([string]$Phase)
    $errsJson = Get-PhaseErrorsJson $Phase
    if ([string]::IsNullOrEmpty($errsJson) -or $errsJson -eq "null" -or $errsJson -eq "[]") { return "✅ No errors" }
    $items = $errsJson | ConvertFrom-Json
    if ($items.Count -eq 0) { return "✅ No errors" }
    $rows = @("| Tool Call | Error | Response |", "|-----------|-------|----------|")
    foreach ($item in $items) {
        $resp = if ($item.Response -and $item.Response.Length -gt 120) { $item.Response.Substring(0,120) } else { $item.Response }
        $rows += "| $($item.Tool) | $($item.Error) | $resp |"
    }
    return $rows -join "`n"
}

function Add-PhaseError {
    param([string]$Tool, [string]$Err, [string]$Resp)
    $snippet = if ($Resp.Length -gt 250) { $Resp.Substring(0, 250) + "..." } else { $Resp }
    $phase = $Script:CURRENT_PHASE
    # Always use ordered hashtable to avoid PSCustomObject vs hashtable inconsistency
    if ([string]::IsNullOrEmpty($Script:PHASE_ERRORS_JSON) -or $Script:PHASE_ERRORS_JSON -eq "{}") {
        $errs = [ordered]@{}
    } else {
        $parsed = $Script:PHASE_ERRORS_JSON | ConvertFrom-Json
        $errs = [ordered]@{}
        foreach ($prop in $parsed.PSObject.Properties) { $errs[$prop.Name] = [System.Collections.ArrayList]($prop.Value) }
    }
    if (-not $errs.Contains($phase)) { $errs[$phase] = [System.Collections.ArrayList]::new() }
    $errs[$phase].Add([PSCustomObject]@{Tool = $Tool; Error = $Err; Response = $snippet}) | Out-Null
    $Script:PHASE_ERRORS_JSON = $errs | ConvertTo-Json -Depth 5 -Compress
}

function Get-PhaseErrorsJson {
    param([string]$Phase)
    if ([string]::IsNullOrEmpty($Script:PHASE_ERRORS_JSON) -or $Script:PHASE_ERRORS_JSON -eq "{}") { return "[]" }
    $errs = $Script:PHASE_ERRORS_JSON | ConvertFrom-Json
    $val = $errs.PSObject.Properties[$Phase]
    if (-not $val -or $null -eq $val.Value) { return "[]" }
    return $val.Value | ConvertTo-Json -Compress
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
    try {
        $data = $Script:PREV_METRICS | ConvertFrom-Json
        $match = $data.phase_scores | Where-Object { $_.phase -eq $Phase } | Select-Object -First 1
        if (-not $match) { return "" }
        return $match.$Metric
    } catch { return "" }
}

function Trend-Between {
    param([string]$Current, [string]$Previous)
    if ([string]::IsNullOrEmpty($Previous)) { return "—" }
    $c = 0.0; $p = 0.0
    if (-not [double]::TryParse($Current, [ref]$c)) { return "—" }
    if (-not [double]::TryParse($Previous, [ref]$p)) { return "—" }
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
    $items = $Checks | ConvertFrom-Json
    if ($items.Count -eq 0) { return "No checks performed for this phase." }
    $total = $items.Count
    $ok = ($items | Where-Object { $_.Status -eq "pass" }).Count
    $fail = ($items | Where-Object { $_.Status -eq "fail" }).Count
    $warn = ($items | Where-Object { $_.Status -eq "warn" }).Count
    $skip = ($items | Where-Object { $_.Status -eq "skip" }).Count
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
    $items = $Checks | ConvertFrom-Json
    $recs = ""
    $failItems = $items | Where-Object { $_.Status -eq "fail" }
    $warnItems = $items | Where-Object { $_.Status -eq "warn" }
    foreach ($f in $failItems) { $recs += "- 🔴 Fix failing check: $($f.Name)`n" }
    foreach ($w in $warnItems) { $recs += "- 🟡 Address warning: $($w.Name)`n" }
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
