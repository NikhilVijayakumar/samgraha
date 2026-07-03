<#
.SYNOPSIS
  Discover and test all MCP tools by walking the domain-document-section
  hierarchy. Produces templated reports in docs/report/manual-audit/mcp/latest/.

.DESCRIPTION
  Pipeline: Bootstrap → Domain Scan → Doc Discovery → Doc Verify →
  Cross-Section → Section Verify → Search → Audit → Gaps → Registry
  → Summary report.

  Reports are rendered from markdown templates in
  scripts/templates/mcp/.
  Previous run is rotated to archive/{timestamp}/.
  Scores and metrics are persisted for trend analysis across runs.

.PARAMETER Build
  Run cargo build --bin mcp before scanning.

.PARAMETER BinaryPath
  Path to pre-built mcp.exe. Default: target\debug\mcp.exe (then release).

.PARAMETER ReportDir
  Report output root. Default: docs/report/manual-audit/mcp

.PARAMETER Domain
  Only scan named domains. Default: all.

.PARAMETER MaxDocs
  Cap docs per domain (0 = unlimited).

.PARAMETER MaxSections
  Cap sections per doc (0 = unlimited).

.PARAMETER NoSectionContent
  Skip get_document_section calls (saves ~400 calls / ~20s).

.PARAMETER NoAudit
  Skip audit/report/gate calls (saves ~100 calls / ~5s).

.PARAMETER PassThru
  Print report paths to console on completion.

.EXAMPLE
  .\scripts\mcp-discover.ps1 -Build -PassThru

.EXAMPLE
  .\scripts\mcp-discover.ps1 -Domain engineering,feature -MaxDocs 3 -NoSectionContent
#>

param(
    [switch]$Build,
    [string]$BinaryPath = "",
    [string]$ReportDir = "",
    [string[]]$Domain = @(),
    [int]$MaxDocs = 0,
    [int]$MaxSections = 0,
    [switch]$NoSectionContent,
    [switch]$NoAudit,
    [switch]$PassThru
)

#Requires -Version 5.1

# ─── Setup ────────────────────────────────────────────────────────────────────
$ErrorActionPreference = "Stop"
$Script:RootDir = Split-Path -Parent $PSScriptRoot

. "$Script:RootDir\scripts\lib\report.ps1"

if (-not (Get-Command jq -ErrorAction SilentlyContinue)) { throw "jq is required but not found on PATH" }
if (-not (Get-Command python3 -ErrorAction SilentlyContinue)) { throw "python3 is required but not found on PATH" }

# Binary resolution
if ($Build) {
    Write-Host "Building mcp binary..." -ForegroundColor Cyan
    Push-Location $Script:RootDir
    cargo build --bin mcp 2>&1 | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Pop-Location
    $Script:BinaryPath = Join-Path $Script:RootDir "target\debug\mcp.exe"
} elseif ($BinaryPath) {
    $Script:BinaryPath = $BinaryPath
} else {
    $dbg = Join-Path $Script:RootDir "target\debug\mcp.exe"
    $rel = Join-Path $Script:RootDir "target\release\mcp.exe"
    if (Test-Path $dbg) { $Script:BinaryPath = $dbg }
    elseif (Test-Path $rel) { $Script:BinaryPath = $rel }
    else { throw "No mcp.exe found. Use -Build or -BinaryPath." }
}

if (-not (Test-Path $Script:BinaryPath)) {
    throw "Binary not found: $Script:BinaryPath"
}

Write-Host "MCP binary: $Script:BinaryPath" -ForegroundColor DarkGray

# Override template dir to match bash version's convention
$Script:TEMPLATE_DIR = Join-Path $Script:RootDir "scripts\templates\mcp"

# Report directory setup
if (-not $ReportDir) { $ReportDir = "docs/report/manual-audit/mcp" }
$Script:ReportDir = Join-Path $Script:RootDir $ReportDir

Initialize-ReportDirs "mcp"
$Script:ArchivePath = $null

# Ensure template and archive dirs exist
if (-not (Test-Path $Script:TEMPLATE_DIR)) { New-Item -ItemType Directory -Force $Script:TEMPLATE_DIR | Out-Null }
if (-not (Test-Path $Script:ARCHIVE_DIR)) { New-Item -ItemType Directory -Force $Script:ARCHIVE_DIR | Out-Null }

# Rotate previous latest → archive/{timestamp}/
if (Test-Path $Script:LATEST_DIR) {
    $ts = Get-Date -Format "yyyy-MM-dd_HHmmss"
    $archivePath = Join-Path $Script:ARCHIVE_DIR $ts
    Move-Item $Script:LATEST_DIR $archivePath
    $Script:ArchivePath = $archivePath
    Write-Host "Archived previous run → $archivePath" -ForegroundColor DarkGray
}
New-Item -ItemType Directory -Force $Script:LATEST_DIR | Out-Null

Load-PreviousMetrics $Script:ARCHIVE_DIR

# ─── Global State ─────────────────────────────────────────────────────────────
$Script:NEXT_ID = 1
$Script:TOTAL_CALLS = 0
$Script:CURRENT_PHASE = ""
$Script:PHASE_ERRORS_JSON = "{}"
$Script:PREV_METRICS = "{}"
$Script:PHASE_RESULTS = @{}
$Script:PHASE_ID = ""
$Script:PHASE_DURATION = $null
$Script:PHASE_CHECKS = @{}
$Script:AllResults = @{
    Protocol = $null
    Tools = @()
    Runtime = $null
    Domains = @{}
    TotalDocs = 0
    AllDocs = @()
    TotalSections = 0
    SectionIds = @{}
    SectionsByType = @{}
}

# ─── Core Functions ───────────────────────────────────────────────────────────

function Get-Id {
    $id = $Script:NEXT_ID
    $Script:NEXT_ID++
    return $id
}

function Invoke-McpDirect {
    <#
    .SYNOPSIS
      Send JSON-RPC directly (method at top level). Used for initialize, tools/list.
    #>
    param([string]$Method, $Params = @{}, [int]$Id = -1)

    if ($Id -eq -1) { $Id = Get-Id }
    $Script:TOTAL_CALLS++

    $request = @{
        jsonrpc = "2.0"
        id = $Id
        method = $Method
        params = $Params
    } | ConvertTo-Json -Compress -Depth 10

    try {
        $raw = $request | & $Script:BinaryPath 2>$null
        if ([string]::IsNullOrEmpty("$raw")) {
            Add-PhaseError -Tool $request -Err "Empty response" -Resp ""
            return $null
        }
        if ($raw -is [array]) { $raw = $raw[0] }
        $parsed = $raw | ConvertFrom-Json
        if ($parsed.PSObject.Properties['error'] -and $parsed.error) {
            Add-PhaseError -Tool $request -Err "$($parsed.error.code): $($parsed.error.message)" -Resp $raw
            return $null
        }
        return $parsed.result
    } catch {
        Add-PhaseError -Tool $request -Err $_.Exception.Message -Resp ""
        return $null
    }
}

function Invoke-McpTool {
    <#
    .SYNOPSIS
      Send JSON-RPC via tools/call. Returns result object or $null on error.
    #>
    param(
        [string]$Name,
        $Arguments = @{},
        [int]$Id = -1,
        [switch]$Quiet   # skip error logging (for expected-failure tests)
    )

    if ($Id -eq -1) { $Id = Get-Id }
    $Script:TOTAL_CALLS++

    $request = @{
        jsonrpc = "2.0"
        id = $Id
        method = "tools/call"
        params = @{
            name = $Name
            arguments = $Arguments
        }
    } | ConvertTo-Json -Compress -Depth 10

    try {
        $raw = $request | & $Script:BinaryPath 2>$null
        if ([string]::IsNullOrEmpty("$raw")) {
            if (-not $Quiet) { Add-PhaseError -Tool $request -Err "Empty response" -Resp "" }
            return $null
        }
        if ($raw -is [array]) { $raw = $raw[0] }
        $parsed = $raw | ConvertFrom-Json
        if ($parsed.PSObject.Properties['error'] -and $parsed.error) {
            if (-not $Quiet) { Add-PhaseError -Tool $request -Err "$($parsed.error.code): $($parsed.error.message)" -Resp $raw }
            return $null
        }
        return $parsed.result
    } catch {
        if (-not $Quiet) { Add-PhaseError -Tool $request -Err $_.Exception.Message -Resp "" }
        return $null
    }
}

function Invoke-McpToolAll {
    <#
    .SYNOPSIS
      Paginated variant of Invoke-McpTool. Follows has_more with offset.
      $CollectionKey = name of array field in response (e.g. "documents", "sections").
    #>
    param(
        [string]$Name,
        $Arguments = @{},
        [string]$CollectionKey,
        [int]$PageSize = 100,
        [int]$Id = -1
    )

    $all = [System.Collections.ArrayList]::new()
    $offset = 0
    $id = if ($Id -eq -1) { Get-Id } else { $Id }
    $hasMore = $true

    while ($hasMore) {
        $args = @{} + $Arguments
        $args.limit = $PageSize
        $args.offset = $offset

        $result = Invoke-McpTool -Name $Name -Arguments $args -Id $id
        if ($null -eq $result) { break }

        $items = $result.$CollectionKey
        if ($items) {
            foreach ($item in $items) { $all.Add($item) | Out-Null }
        }

        $hasMore = $result.has_more -eq $true
        $offset += $PageSize
        $id++

        if ($items -and $items.Count -lt $PageSize) { $hasMore = $false }
    }

    return $all.ToArray()
}

function Build-PhaseScoresJson {
    $order = @("01-tool-health", "02-domain-catalog", "03-document-audit", "04-section-integrity", "05-search-results", "06-audit-findings", "07-coverage-gaps", "08-registry-state")
    $arr = '[]'
    foreach ($key in $order) {
        if ($Script:PHASE_RESULTS.ContainsKey($key)) {
            $pr = $Script:PHASE_RESULTS[$key]
            $score = $pr | jq -r '.Score // 0'
            $status = $pr | jq -r '.Status // "?"'
            $errors = $pr | jq -r '.Errors // 0'
            $dur = $pr | jq -r '.Duration // 0'
            $arr = $arr | jq -c --arg key "$key" --argjson score $score --arg status "$status" --argjson errors $errors --argjson dur $dur '. += [{phase: $key, score: $score, status: $status, errors: $errors, duration: $dur}]'
        }
    }
    return $arr
}

# ─── Phase 1: Bootstrap ──────────────────────────────────────────────────────

function Phase-1-Bootstrap {
    $Script:CURRENT_PHASE = "01-tool-health"
    Write-Host "Phase 1: Bootstrap..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    # initialize
    $initResult = Invoke-McpDirect -Method "initialize" -Params @{
        protocolVersion = "2025-03-26"
        capabilities = @{}
        clientInfo = @{ name = "mcp-discover"; version = "1.0" }
    }
    if ($initResult) {
        $pv = $initResult.protocolVersion
        $Script:AllResults.Protocol = $pv
        $checks = $checks | jq -c --arg pv "$pv" '. += [{"Name": "Initialize", "Status": "pass", "Detail": "Protocol \($pv)"}]'
    } else {
        $checks = $checks | jq -c '. += [{"Name": "Initialize", "Status": "fail", "Detail": "No response"}]'
    }

    # tools/list
    $toolsResult = Invoke-McpDirect -Method "tools/list"
    $tools = @()
    if ($toolsResult -and $toolsResult.tools) {
        $tools = $toolsResult.tools
        $tc = $tools.Count
        $Script:AllResults.Tools = $tools
        $checks = $checks | jq -c --argjson tc $tc '. += [{"Name": "Tools/List", "Status": "pass", "Detail": "\($tc) tools"}]'
    } else {
        $checks = $checks | jq -c '. += [{"Name": "Tools/List", "Status": "fail", "Detail": "No tools returned"}]'
    }

    # info via tools/call
    $infoResult = Invoke-McpTool -Name "info"
    if ($infoResult) {
        $dc = $infoResult.document_count
        $Script:AllResults.Runtime = $infoResult
        $checks = $checks | jq -c --arg dc "$dc" '. += [{"Name": "Info", "Status": "pass", "Detail": "\($dc) docs"}]'
    } else {
        $checks = $checks | jq -c '. += [{"Name": "Info", "Status": "fail", "Detail": "No response"}]'
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "01-tool-health"
    $errorCount = [int]($errorJson | jq 'length')
    $hasFail = [int]($checks | jq '[.[] | select(.Status == "fail")] | length')
    $status = if ($hasFail -gt 0) { "❌ FAIL" } else { "✅ PASS" }

    # Build tool table for template
    $toolRows = [System.Collections.ArrayList]::new()
    $ti = 0
    foreach ($t in $tools) {
        $ti++
        $name = $t.name
        $req = if ($t.inputSchema -and $t.inputSchema.required) { $t.inputSchema.required -join ", " } else { "none" }
        $errorsJson = Get-PhaseErrorsJson "01-tool-health"
        $hasErr = $errorsJson | jq --arg n "$name" 'any(.[]; .Tool | contains($n))'
        $toolStatus = if ($hasErr -eq "true") { "⚠️" } else { "✅" }
        $toolRows.Add("| $ti | `"$name`" | $req | $toolStatus |") | Out-Null
    }

    $standards = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) {
        $Script:AllResults.Runtime.standards -join ", "
    } else { "--" }
    $standardCount = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) { $Script:AllResults.Runtime.standards.Count } else { 0 }
    $services = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.services) {
        $Script:AllResults.Runtime.services -join ", "
    } else { "--" }
    $policy = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.policy) {
        ($Script:AllResults.Runtime.policy | ConvertTo-Json -Compress -Depth 2)
    } else { "--" }
    $registryPath = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.registry_path } else { "--" }
    $repositoryName = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.repository } else { "--" }
    $docCount = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.document_count } else { "?" }
    $healthyToolCount = $tools.Count

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "01-tool-health") -join "`n"

    # Score: % of checks passing, penalized by errors
    $tc = [int]($checks | jq 'length')
    $passC = [int]($checks | jq '[.[] | select(.Status == "pass")] | length')
    $score = 0
    if ($tc -gt 0) { $score = [int]($passC * 100 / $tc) }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 5 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "01-tool-health" $checks
    $recommendations = Gen-PhaseRecs "01-tool-health" $checks
    $prevScore = Get-PrevMetric "01-tool-health" "score"
    $trend = Trend-Between $score $prevScore
    $prevDocCount = Get-PrevMetric "01-tool-health" "prev_doc_count"
    if ([string]::IsNullOrEmpty($prevDocCount)) { $prevDocCount = "" }
    $docTrend = Trend-Between $docCount $prevDocCount

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg TOOLS_TABLE "$($toolRows -join "`n")" `
        --arg DOC_COUNT "$docCount" `
        --arg STANDARDS_LIST "$standards" `
        --arg STANDARD_COUNT $standardCount `
        --arg REGISTRY_PATH "$registryPath" `
        --arg REPOSITORY "$repositoryName" `
        --arg SERVICES "$services" `
        --arg POLICY "$policy" `
        --arg TOOL_COUNT $healthyToolCount `
        --arg TOOL_ERROR_COUNT $errorCount `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        --arg PROTOCOL_VERSION "$($Script:AllResults.Protocol)" `
        --arg HEALTHY_TOOL_COUNT $healthyToolCount `
        --arg PREV_DOC_COUNT "$prevDocCount" `
        --arg DOC_TREND "$docTrend" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, TOOLS_TABLE: $TOOLS_TABLE, DOC_COUNT: $DOC_COUNT, STANDARDS_LIST: $STANDARDS_LIST, STANDARD_COUNT: $STANDARD_COUNT, REGISTRY_PATH: $REGISTRY_PATH, REPOSITORY: $REPOSITORY, SERVICES: $SERVICES, POLICY: $POLICY, TOOL_COUNT: $TOOL_COUNT, TOOL_ERROR_COUNT: $TOOL_ERROR_COUNT, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE, PROTOCOL_VERSION: $PROTOCOL_VERSION, HEALTHY_TOOL_COUNT: $HEALTHY_TOOL_COUNT, PREV_DOC_COUNT: $PREV_DOC_COUNT, DOC_TREND: $DOC_TREND}'
    Write-Report "01-tool-health.md" "01-tool-health.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["01-tool-health"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "01-tool-health.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($duration`s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 2: Domain Catalog ──────────────────────────────────────────────────

function Phase-2-DomainScan {
    $Script:CURRENT_PHASE = "02-domain-catalog"
    Write-Host "Phase 2: Domain Scan..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    $domainsResult = Invoke-McpTool -Name "list_domains" -Arguments @{}
    $domains = @()
    $allDomainNames = @()
    if ($domainsResult -and $domainsResult.domains) {
        $allDomains = $domainsResult.domains
        if ($Domain.Count -gt 0) {
            $domains = $allDomains | Where-Object { $_ -in $Domain }
        } else {
            $domains = $allDomains
        }
        $allDomainNames = $domains
        $Script:AllResults.Domains = @{}
        foreach ($d in $domains) {
            $Script:AllResults.Domains[$d] = @{
                name = $d
                docs = @()
                sectionTypes = @{}
                sectionIds = @()
                docCount = 0
            }
        }
        $cnt = $domains.Count
        $checks = $checks | jq -c --argjson cnt $cnt '. += [{"Name": "List Domains", "Status": "pass", "Detail": "\($cnt) domains"}]'
    } else {
        $checks = $checks | jq -c '. += [{"Name": "List Domains", "Status": "fail", "Detail": "No domains"}]'
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "02-domain-catalog"
    $errorCount = [int]($errorJson | jq 'length')

    $status = if ($allDomainNames.Count -gt 0) { "✅ PASS" } else { "❌ FAIL" }

    # Score: 100 if any domains found, proportional to count, min 20 per domain
    $dc = $allDomainNames.Count
    $score = 0
    if ($dc -gt 0) { $score = [Math]::Min($dc * 25, 100) }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 10 }
    if ($score -lt 0) { $score = 0 }

    $prevScore = Get-PrevMetric "02-domain-catalog" "score"
    $trend = Trend-Between $score $prevScore

    $Script:PHASE_RESULTS["02-domain-catalog"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "02-domain-catalog.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($duration`s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 3: Document Discovery ─────────────────────────────────────────────

function Phase-3-DocDiscover {
    $Script:CURRENT_PHASE = "03-document-audit"
    Write-Host "Phase 3: Document Discovery..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    $domains = $Script:AllResults.Domains.Keys
    $totalDocs = 0
    $allDocCount = 0
    $domainNames = @($domains)

    foreach ($d in $domains) {
        Write-Host "  Fetching docs for '$d'..." -ForegroundColor DarkGray
        $docResult = Invoke-McpToolAll -Name "get_documents_by_domain" -Arguments @{ domain = $d } -CollectionKey "documents"

        if ($docResult -and $docResult.Count -gt 0) {
            $docs = $docResult
            $Script:AllResults.Domains[$d].docs = $docs
            $Script:AllResults.Domains[$d].docCount = $docs.Count
            $totalDocs += $docs.Count
            $dc = $docs.Count
            $cn = "Docs in '$d'"
            $checks = $checks | jq -c --arg n "$cn" --argjson dc $dc '. += [{"Name": $n, "Status": "pass", "Detail": "\($dc) docs"}]'
        } else {
            $Script:AllResults.Domains[$d].docCount = 0
            $cn = "Docs in '$d'"
            $checks = $checks | jq -c --arg n "$cn" '. += [{"Name": $n, "Status": "skip", "Detail": "0 docs (or error)"}]'
        }
        $allDocCount += $Script:AllResults.Domains[$d].docCount
    }

    $Script:AllResults.TotalDocs = $allDocCount

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "03-document-audit"
    $errorCount = [int]($errorJson | jq 'length')
    $hasFailChecks = [int]($checks | jq '[.[] | select(.Status == "fail")] | length')
    $status = if ($hasFailChecks -gt 0) { "⚠️ PARTIAL" } elseif ($allDocCount -gt 0) { "✅ PASS" } else { "❌ FAIL" }

    # Generate domain catalog report with actual doc counts
    $dcDomainRows = [System.Collections.ArrayList]::new()
    $dcDocCountRows = [System.Collections.ArrayList]::new()
    $dcDi = 0
    foreach ($d in $domains) {
        $dcDi++
        $cnt = $Script:AllResults.Domains[$d].docCount
        $dcDomainRows.Add("| $dcDi | `"$d`" | $cnt | $cnt |") | Out-Null
        $dcDocCountRows.Add("| $d | $cnt |") | Out-Null
    }
    $dcStandards = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) {
        ($Script:AllResults.Runtime.standards | ForEach-Object { "- $_" }) -join "`n"
    } else { "--" }
    $sc = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) { $Script:AllResults.Runtime.standards.Count } else { 0 }

    # Build all checks from both phase 2 and 3
    $allChecks = $checks | jq -c --argjson dc $domainNames.Count '. += [{"Name": "List Domains", "Status": "pass", "Detail": "\($dc) domains"}]'
    $allChecksTable = (Get-ChecksTable $allChecks) -join "`n"

    $phase2Errors = Get-PhaseErrorsJson "02-domain-catalog"
    $p2ErrorRows = ($phase2Errors | jq -r 'if length == 0 then "No errors" else (["| Tool Call | Error | Response |", "|-----------|-------|----------|"] + (.[] | "| " + (.Tool | gsub("\\|"; "\\|")) + " | " + (.Error | gsub("\\|"; "\\|")) + " | " + (.Response[0:120] | gsub("\\|"; "\\|")) + " |")) | join("\n") end') -join "`n"

    # Phase 2 score from stored results
    $p2Obj = $Script:PHASE_RESULTS["02-domain-catalog"]
    $p2Score = if ($p2Obj) { [int]($p2Obj | jq -r '.Score // 0') } else { 0 }
    $p2Prev = Get-PrevMetric "02-domain-catalog" "score"
    $p2Trend = Trend-Between $p2Score $p2Prev
    $p2Analysis = Gen-PhaseAnalysis "02-domain-catalog" $allChecks
    $p2Recs = Gen-PhaseRecs "02-domain-catalog" $allChecks

    $prevDocCount = Get-PrevMetric "03-document-audit" "prev_doc_count"
    if ([string]::IsNullOrEmpty($prevDocCount)) { $prevDocCount = "" }
    $docTrend = Trend-Between $allDocCount $prevDocCount
    $prevDomainCount = Get-PrevMetric "02-domain-catalog" "prev_domain_count"
    if ([string]::IsNullOrEmpty($prevDomainCount)) { $prevDomainCount = "" }
    $domainTrend = Trend-Between $domainNames.Count $prevDomainCount

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "✅ PASS" `
        --arg CHECKS_TABLE "$allChecksTable" `
        --arg ERRORS_TABLE "$p2ErrorRows" `
        --arg DOMAINS_TABLE "$($dcDomainRows -join "`n")" `
        --arg STANDARDS_LIST "$dcStandards" `
        --arg STANDARD_COUNT $sc `
        --arg DOC_COUNTS_TABLE "$($dcDocCountRows -join "`n")" `
        --arg DOMAIN_COUNT $domainNames.Count `
        --arg DOCUMENT_COUNT $allDocCount `
        --arg SCORE $p2Score `
        --arg TREND "$p2Trend" `
        --arg ANALYSIS "$p2Analysis" `
        --arg RECOMMENDATIONS "$p2Recs" `
        --arg PREV_SCORE "$p2Prev" `
        --arg PREV_DOC_COUNT "$prevDocCount" `
        --arg DOC_TREND "$docTrend" `
        --arg PREV_DOMAIN_COUNT "$prevDomainCount" `
        --arg DOMAIN_TREND "$domainTrend" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, DOMAINS_TABLE: $DOMAINS_TABLE, STANDARDS_LIST: $STANDARDS_LIST, STANDARD_COUNT: $STANDARD_COUNT, DOC_COUNTS_TABLE: $DOC_COUNTS_TABLE, DOMAIN_COUNT: $DOMAIN_COUNT, DOCUMENT_COUNT: $DOCUMENT_COUNT, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE, PREV_DOC_COUNT: $PREV_DOC_COUNT, DOC_TREND: $DOC_TREND, PREV_DOMAIN_COUNT: $PREV_DOMAIN_COUNT, DOMAIN_TREND: $DOMAIN_TREND}'
    Write-Report "02-domain-catalog.md" "02-domain-catalog.md" $reportVals | Out-Null

    # Score for doc-discover phase
    $domainsWithDocs = [int]($Script:AllResults.Domains.Keys | Where-Object { $Script:AllResults.Domains[$_].docCount -gt 0 } | Measure-Object).Count
    $expected = [Math]::Max($domainsWithDocs * 3, 1)
    $score = 0
    if ($allDocCount -ge $expected) { $score = 100 }
    elseif ($allDocCount -gt 0) { $score = [int]($allDocCount * 100 / $expected) }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 5 }
    if ($score -lt 0) { $score = 0 }

    $prevScore = Get-PrevMetric "03-document-audit" "score"
    $trend = Trend-Between $score $prevScore

    $Script:PHASE_RESULTS["03-doc-discover"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($allDocCount docs discovered)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 4: Document Verification ──────────────────────────────────────────

function Phase-4-DocVerify {
    $Script:CURRENT_PHASE = "03-document-audit"
    Write-Host "Phase 4: Document Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    $domains = $Script:AllResults.Domains.Keys
    $domainNames = @($domains)
    $allDocs = @()
    $allSectionsTotal = 0
    $qualityRows = [System.Collections.ArrayList]::new()
    $domainDocsSectionsParts = [System.Collections.ArrayList]::new()
    $issues = [System.Collections.ArrayList]::new()
    $issueCount = 0
    $sectDist = @{ "0" = 0; "1-3" = 0; "4-7" = 0; "8-15" = 0; "16+" = 0 }
    $emptySects = 0
    $missingSects = 0

    foreach ($d in $domains) {
        $docs = $Script:AllResults.Domains[$d].docs
        $totalSects = 0
        $emptySectsD = 0
        $missingSectsD = 0
        $docParts = [System.Collections.ArrayList]::new()
        $docParts.Add("### $d") | Out-Null
        $docParts.Add("") | Out-Null
        $docParts.Add("| Doc ID | Title | Sections | Coverage | Issues |") | Out-Null
        $docParts.Add("|--------|-------|----------|----------|--------|") | Out-Null

        foreach ($doc in $docs) {
            $docId = $doc.id
            $title = $doc.title
            $quality = $doc.quality
            $body = $doc.body

            $sectCount = if ($quality -and $quality.total_section_count -ne $null) { $quality.total_section_count } else { 0 }
            $emptyCount = if ($quality -and $quality.empty_section_count -ne $null) { $quality.empty_section_count } else { 0 }
            $missingCount = if ($quality -and $quality.missing_section_count -ne $null) { $quality.missing_section_count } else { 0 }
            $coverage = if ($quality -and $quality.coverage -ne $null) { $quality.coverage } else { 0 }
            $requiredCount = if ($quality -and $quality.required_section_count -ne $null) { $quality.required_section_count } else { 0 }

            $totalSects += $sectCount
            if ($emptyCount -gt 0) { $emptySectsD += $emptyCount; $emptySects += $emptyCount }
            if ($missingCount -gt 0) { $missingSectsD += $missingCount; $missingSects += $missingCount }

            # Section distribution
            if ($sectCount -eq 0) { $sectDist["0"]++ }
            elseif ($sectCount -le 3) { $sectDist["1-3"]++ }
            elseif ($sectCount -le 7) { $sectDist["4-7"]++ }
            elseif ($sectCount -le 15) { $sectDist["8-15"]++ }
            else { $sectDist["16+"]++ }

            # Collect section types per domain
            if ($body -and $body.PSObject.Properties) {
                $bodyVal = $null
                foreach ($prop in $body.PSObject.Properties) {
                    $bodyVal = $prop.Value
                    break
                }
                if ($bodyVal -and $bodyVal.sections) {
                    foreach ($section in $bodyVal.sections) {
                        $st = $section.semantic_type
                        if ($st) { $Script:AllResults.Domains[$d].sectionTypes[$st] = ($Script:AllResults.Domains[$d].sectionTypes[$st] -or 0) + 1 }
                        $allSectionsTotal++
                    }
                }
            }

            # Issues
            $docIssues = @()
            if ($emptyCount -gt 0) { $docIssues += "$emptyCount empty sections"; $issueCount++ }
            if ($missingCount -gt 0) { $docIssues += "$missingCount missing sections"; $issueCount++ }
            if ($coverage -lt 0.5) { $docIssues += "low coverage ($coverage)"; $issueCount++ }
            if (-not $doc.hash) { $docIssues += "no hash"; $issueCount++ }

            $issueStr = if ($docIssues.Count -gt 0) { $docIssues -join "; " } else { "✅" }
            $coverageStr = if ($coverage -ne $null) { "$([math]::Round($coverage * 100, 0))%" } else { "?" }
            $docParts.Add("| $docId | `"$title`" | $sectCount | $coverageStr | $issueStr |") | Out-Null

            if ($docIssues.Count -gt 0) {
                $issues.Add("| $docId | `"$title`" | $d | $($docIssues -join '; ') |") | Out-Null
            }

            $allDocs += @{ doc = $doc; domain = $d }
        }

        $docParts.Add("") | Out-Null
        $domainDocsSectionsParts.Add($docParts -join "`n") | Out-Null

        $avgSects = if ($docs.Count -gt 0) { [math]::Round($totalSects / $docs.Count, 1) } else { 0 }
        $qualityRows.Add("| $d | $($docs.Count) | $avgSects | $emptySectsD | $missingSectsD | -- |") | Out-Null
    }

    $Script:AllResults.AllDocs = $allDocs
    $Script:AllResults.TotalSections = $allSectionsTotal

    $allDocsCount = $allDocs.Count
    $checks = $checks | jq -c --argjson dc $allDocsCount --argjson dm $domainNames.Count '. += [{"Name": "Document verification", "Status": "pass", "Detail": "\($dc) docs across \($dm) domains"}]'
    $checks = $checks | jq -c --argjson ts $allSectionsTotal '. += [{"Name": "Section count", "Status": "pass", "Detail": "\($ts) sections total"}]'

    $distRows = [System.Collections.ArrayList]::new()
    foreach ($k in @("0", "1-3", "4-7", "8-15", "16+")) {
        if ($sectDist[$k] -gt 0) {
            $distRows.Add("| $k sections | $($sectDist[$k]) docs |") | Out-Null
        }
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "03-document-audit"
    $errorCount = [int]($errorJson | jq 'length')
    $status = if ($issueCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "03-document-audit") -join "`n"

    $issuesTable = "No issues found"
    if ($issues.Count -gt 0) {
        $issuesTable = "| Doc ID | Title | Domain | Issues |`n|--------|-------|--------|--------|`n$($issues -join "`n")"
    }

    if ($distRows.Count -eq 0) { $distRows.Add("| -- | -- |") | Out-Null }

    # Score: weighted by coverage, empty sections penalty, missing sections penalty
    $score = 0
    if ($allSectionsTotal -gt 0 -or $allDocsCount -gt 0) {
        $covScore = 50
        $maxIssues = [Math]::Max($allDocsCount * 3, 1)
        $issueRatio = [int]($issueCount * 100 / $maxIssues)
        $covScore = [Math]::Max(50 - [int]($issueRatio / 2), 0)
        $emptyPenalty = $emptySects * 5
        $missingPenalty = $missingSects * 5
        $score = $covScore + 50 - $emptyPenalty - $missingPenalty
    }
    if ($score -lt 0) { $score = 0 }
    if ($score -gt 100) { $score = 100 }

    $analysis = Gen-PhaseAnalysis "03-document-audit" $checks
    $recommendations = Gen-PhaseRecs "03-document-audit" $checks
    $prevScore = Get-PrevMetric "03-document-audit" "score"
    $trend = Trend-Between $score $prevScore
    $prevSectCount = Get-PrevMetric "03-document-audit" "prev_sect_count"
    if ([string]::IsNullOrEmpty($prevSectCount)) { $prevSectCount = "" }
    $sectTrend = Trend-Between $allSectionsTotal $prevSectCount

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg DOMAIN_DOCS_SECTIONS "$($domainDocsSectionsParts -join "`n`n")" `
        --arg QUALITY_TABLE "$($qualityRows -join "`n")" `
        --arg ISSUES_LIST "$issuesTable" `
        --arg SECTION_DIST_TABLE "$($distRows -join "`n")" `
        --arg TOTAL_DOCS $allDocsCount `
        --arg DOMAIN_COUNT $domainNames.Count `
        --arg TOTAL_SECTIONS $allSectionsTotal `
        --arg ISSUE_COUNT $issueCount `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        --arg PREV_SECT_COUNT "$prevSectCount" `
        --arg SECT_TREND "$sectTrend" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, DOMAIN_DOCS_SECTIONS: $DOMAIN_DOCS_SECTIONS, QUALITY_TABLE: $QUALITY_TABLE, ISSUES_LIST: $ISSUES_LIST, SECTION_DIST_TABLE: $SECTION_DIST_TABLE, TOTAL_DOCS: $TOTAL_DOCS, DOMAIN_COUNT: $DOMAIN_COUNT, TOTAL_SECTIONS: $TOTAL_SECTIONS, ISSUE_COUNT: $ISSUE_COUNT, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE, PREV_SECT_COUNT: $PREV_SECT_COUNT, SECT_TREND: $SECT_TREND}'
    Write-Report "03-document-audit.md" "03-document-audit.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["03-document-audit"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "03-document-audit.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($issueCount issues)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($issueCount -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 5: Cross-Section ──────────────────────────────────────────────────

function Phase-5-CrossSection {
    $Script:CURRENT_PHASE = "04-section-integrity"
    Write-Host "Phase 5: Cross-Section..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    # Collect unique (domain, type) pairs from Phase 4 data
    $pairs = @{}
    foreach ($d in $Script:AllResults.Domains.Keys) {
        $types = $Script:AllResults.Domains[$d].sectionTypes
        foreach ($t in $types.Keys) {
            $key = "$d|$t"
            $pairs[$key] = @{ domain = $d; type = $t }
        }
    }

    $allSectionIds = @{}
    $sectionsByType = @{}
    $totalSections = 0
    $pairCount = $pairs.Count

    foreach ($pair in $pairs.Values) {
        $d = $pair.domain
        $t = $pair.type

        $sects = Invoke-McpToolAll -Name "get_sections" -Arguments @{
            semantic_type = $t
            domain = $d
        } -CollectionKey "sections" -PageSize 100

        if ($sects -and $sects.Count -gt 0) {
            if (-not $allSectionIds.ContainsKey($d)) { $allSectionIds[$d] = @{} }
            $allSectionIds[$d][$t] = @($sects | ForEach-Object { $_.id })
            $totalSections += $sects.Count

            if (-not $sectionsByType.ContainsKey($t)) { $sectionsByType[$t] = @() }
            foreach ($s in $sects) { $sectionsByType[$t] += $s }
        }
    }

    $Script:AllResults.SectionIds = $allSectionIds
    $Script:AllResults.SectionsByType = $sectionsByType

    $checks = $checks | jq -c --argjson pc $pairCount --argjson ts $totalSections '. += [{"Name": "Cross-section query", "Status": "pass", "Detail": "\($pc) type-domain pairs, \($ts) sections"}]'

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "04-section-integrity"
    $errorCount = [int]($errorJson | jq 'length')

    $status = if ($totalSections -gt 0) { "✅ PASS" } else { "⚠️ PARTIAL" }

    # Score: retrieval rate
    $score = 0
    if ($pairCount -gt 0) {
        $expectedSects = $pairCount * 3
        if ($totalSections -ge $expectedSects) { $score = 100 }
        else { $score = [int]($totalSections * 100 / $expectedSects) }
    }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 5 }
    if ($score -lt 0) { $score = 0 }

    $prevScore = Get-PrevMetric "04-section-integrity" "score"
    $trend = Trend-Between $score $prevScore

    $Script:PHASE_RESULTS["04-cross-section"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, Score: $score}'
    Write-Host "  → $totalSections sections from $pairCount type-domain pairs" -ForegroundColor Green
}

# ─── Phase 6: Section Verification ───────────────────────────────────────────

function Phase-6-SectionVerify {
    $Script:CURRENT_PHASE = "04-section-integrity"
    Write-Host "Phase 6: Section Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    $sectionIds = $Script:AllResults.SectionIds
    $sectionVerifyParts = [System.Collections.ArrayList]::new()
    $sectionTypeRows = [System.Collections.ArrayList]::new()
    $changeTrackRows = [System.Collections.ArrayList]::new()
    $knowledgeRows = [System.Collections.ArrayList]::new()
    $totalSections = 0
    $staleCount = 0
    $knowledgeCount = 0
    $knowledgeMissing = 0

    $domainNames = @($Script:AllResults.Domains.Keys)

    # Section types by domain
    foreach ($d in $domainNames) {
        $types = $Script:AllResults.Domains[$d].sectionTypes
        if ($types.Keys.Count -gt 0) {
            $typeList = ($types.Keys | ForEach-Object { "$_ ($($types[$_]))" }) -join ", "
            $sectionTypeRows.Add("| $d | $typeList |") | Out-Null
        }
    }

    # Verify each section_id
    $verifyLines = [System.Collections.ArrayList]::new()
    $verifyLines.Add("| Domain | Section ID | Type | get_section | changed |") | Out-Null
    $verifyLines.Add("|--------|-----------|------|-------------|---------|") | Out-Null
    $verifyCount = 0
    $maxVerify = 500  # cap to prevent excessive calls

    $allUniqueTypes = @{}

    foreach ($d in $sectionIds.Keys) {
        foreach ($t in $sectionIds[$d].Keys) {
            $ids = $sectionIds[$d][$t]
            $allUniqueTypes[$t] = $true
            foreach ($sid in $ids) {
                if ($verifyCount -ge $maxVerify) { break }
                $verifyCount++

                # get_section
                $sectResult = Invoke-McpTool -Name "get_section" -Arguments @{ section_id = $sid }
                $sectOk = if ($sectResult -and $sectResult.id -eq $sid) { "✅" } else { "❌" }

                # get_section_changed
                $changedResult = Invoke-McpTool -Name "get_section_changed" -Arguments @{ section_id = $sid }
                $changed = if ($changedResult) { $changedResult.changed } else { "?" }
                if ($changed -eq $true) { $staleCount++ }

                $verifyLines.Add("| $d | $sid | $t | $sectOk | $changed |") | Out-Null
                $totalSections++
            }
            # Audit knowledge per (domain, type) pair
            if (-not $Script:NoAudit) {
                $knResult = Invoke-McpTool -Name "get_audit_knowledge" -Arguments @{
                    domain = $d
                    section_type = $t
                } -Quiet
                if ($knResult -and $knResult.content) {
                    $knowledgeCount++
                    $knowledgeRows.Add("| $d | $t | ✅ $($knResult.content.Length) chars |") | Out-Null
                } else {
                    $knowledgeMissing++
                    $knowledgeRows.Add("| $d | $t | ❌ Missing |") | Out-Null
                }
            }
            if ($verifyCount -ge $maxVerify) { break }
        }
        if ($verifyCount -ge $maxVerify) { break }
    }

    $sectionVerifyParts.Add($verifyLines -join "`n") | Out-Null

    if ($totalSections -eq 0) {
        # Fallback: verify from get_document_section instead
        Write-Host "  No section_ids found from cross-section; using get_document_section" -ForegroundColor DarkGray
        $verifyLines.Clear()
        $verifyLines.Add("| Doc ID | Section Index | Heading | Content Available |") | Out-Null
        $verifyLines.Add("|--------|--------------|--------|-------------------|") | Out-Null
        $docs = $Script:AllResults.AllDocs
        $sampleDocs = $docs | Select-Object -First 10
        foreach ($item in $sampleDocs) {
            $doc = $item.doc
            $bodyProp = $null
            foreach ($prop in $doc.body.PSObject.Properties) { $bodyProp = $prop.Value; break }
            $sections = if ($bodyProp -and $bodyProp.sections) { $bodyProp.sections } else { @() }
            $si = 0
            $maxSect = if ($Script:MaxSections -gt 0) { $Script:MaxSections } else { $sections.Count }
            foreach ($s in $sections) {
                if ($si -ge $maxSect) { break }
                if ($Script:NoSectionContent) {
                    $contentOk = if ($s.body -and $s.body.Length -gt 0) { "✅" } else { "❌ empty" }
                    $verifyLines.Add("| $($doc.id) | $si | $($s.heading) | $contentOk |") | Out-Null
                } else {
                    $sectResult = Invoke-McpTool -Name "get_document_section" -Arguments @{
                        id = $doc.id
                        section = $si
                        limit = 5
                    }
                    $contentOk = if ($sectResult -and $sectResult.content -and $sectResult.content.Length -gt 0) {
                        "✅ ($($sectResult.content.Length) chars)"
                    } else { "❌" }
                    $verifyLines.Add("| $($doc.id) | $si | $($s.heading) | $contentOk |") | Out-Null
                    $totalSections++
                }
                $si++
            }
        }
        $sectionVerifyParts.Add($verifyLines -join "`n") | Out-Null
    }

    $sectionTypesCount = $allUniqueTypes.Keys.Count
    $typeCount = $sectionTypesCount
    $checks = $checks | jq -c --argjson ts $totalSections '. += [{"Name": "Section verification", "Status": "pass", "Detail": "\($ts) sections checked"}]'
    if ($staleCount -gt 0) {
        $checks = $checks | jq -c --argjson sc $staleCount '. += [{"Name": "Stale sections", "Status": "warn", "Detail": "\($sc) changed since last audit"}]'
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "04-section-integrity"
    $errorCount = [int]($errorJson | jq 'length')
    $status = "✅ PASS"

    # Change tracking table
    $changeTrackRows.Add("| Stale (changed) | $staleCount |") | Out-Null
    $changeTrackRows.Add("| Fresh (unchanged) | $($totalSections - $staleCount) |") | Out-Null

    if ($knowledgeRows.Count -eq 0) { $knowledgeRows.Add("| -- | -- | -- |") | Out-Null }
    if ($sectionTypeRows.Count -eq 0) { $sectionTypeRows.Add("| -- | -- |") | Out-Null }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "04-section-integrity") -join "`n"

    # Score: verification success rate + knowledge coverage
    $score = 0
    if ($totalSections -gt 0) {
        $verOk = $totalSections - $staleCount
        $verRate = [int]($verOk * 50 / $totalSections)
        $knRate = 0
        $knTotal = $knowledgeCount + $knowledgeMissing
        if ($knTotal -gt 0) { $knRate = [int]($knowledgeCount * 50 / $knTotal) }
        $score = $verRate + $knRate
    }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 3 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "04-section-integrity" $checks
    $recommendations = Gen-PhaseRecs "04-section-integrity" $checks
    $prevScore = Get-PrevMetric "04-section-integrity" "score"
    $trend = Trend-Between $score $prevScore
    $prevStale = Get-PrevMetric "04-section-integrity" "stale"
    if ([string]::IsNullOrEmpty($prevStale)) { $prevStale = "" }
    $staleTrend = Trend-Between $staleCount $prevStale

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg SECTION_TYPES_TABLE "$($sectionTypeRows -join "`n")" `
        --arg SECTION_VERIFY_TABLE "$($sectionVerifyParts -join "`n`n")" `
        --arg CHANGE_TRACKING_TABLE "$($changeTrackRows -join "`n")" `
        --arg KNOWLEDGE_TABLE "$($knowledgeRows -join "`n")" `
        --arg TOTAL_SECTIONS $totalSections `
        --arg DOMAIN_COUNT $domainNames.Count `
        --arg UNIQUE_TYPES $typeCount `
        --arg STALE_SECTIONS $staleCount `
        --arg KNOWLEDGE_COUNT $knowledgeCount `
        --arg KNOWLEDGE_MISSING $knowledgeMissing `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        --arg PREV_STALE "$prevStale" `
        --arg STALE_TREND "$staleTrend" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, SECTION_TYPES_TABLE: $SECTION_TYPES_TABLE, SECTION_VERIFY_TABLE: $SECTION_VERIFY_TABLE, CHANGE_TRACKING_TABLE: $CHANGE_TRACKING_TABLE, KNOWLEDGE_TABLE: $KNOWLEDGE_TABLE, TOTAL_SECTIONS: $TOTAL_SECTIONS, DOMAIN_COUNT: $DOMAIN_COUNT, UNIQUE_TYPES: $UNIQUE_TYPES, STALE_SECTIONS: $STALE_SECTIONS, KNOWLEDGE_COUNT: $KNOWLEDGE_COUNT, KNOWLEDGE_MISSING: $KNOWLEDGE_MISSING, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE, PREV_STALE: $PREV_STALE, STALE_TREND: $STALE_TREND}'
    Write-Report "04-section-integrity.md" "04-section-integrity.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["04-section-integrity"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "04-section-integrity.md" --argjson score $score --argjson stale $staleCount '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score, Stale: $stale}'
    Write-Host "  → Score: $score/100 $trend — $status ($totalSections sections)" -ForegroundColor Green
}

# ─── Phase 7: Search ─────────────────────────────────────────────────────────

function Phase-7-Search {
    $Script:CURRENT_PHASE = "05-search-results"
    Write-Host "Phase 7: Search..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    # Pick sample queries from discovered data
    $allDocs = $Script:AllResults.AllDocs
    $sampleQueries = @()

    # Get up to 5 doc titles as search queries
    $titles = @()
    foreach ($item in $allDocs) {
        $t = $item.doc.title
        if ($t -and $t.Length -gt 3 -and $t.Length -lt 40) { $titles += $t }
        if ($titles.Count -ge 5) { break }
    }
    foreach ($t in $titles) { $sampleQueries += @{ query = $t; domain = "" } }

    # Add general queries
    $sampleQueries += @{ query = "purpose"; domain = "" }
    $sampleQueries += @{ query = "architecture"; domain = "" }

    $queryResultParts = [System.Collections.ArrayList]::new()
    $expectedFound = 0
    $expectedTotal = $sampleQueries.Count
    $searchErrors = 0

    foreach ($q in $sampleQueries) {
        $args = @{ query = $q.query; limit = 5 }
        if ($q.domain) { $args.domain = $q.domain }

        $result = Invoke-McpTool -Name "search" -Arguments $args
        if ($result -and $result.results) {
            $hitCount = $result.results.Count
            $totalHits = $result.total
            $expectedFound++
            $queryResultParts.Add("### Query: `"$($q.query)`"") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("| Document | Title | Score |") | Out-Null
            $queryResultParts.Add("|----------|-------|-------|") | Out-Null
            foreach ($hit in $result.results) {
                $score = if ($hit.score -ne $null) { [math]::Round($hit.score, 4) } else { "--" }
                $queryResultParts.Add("| $($hit.document_id) | `"$($hit.title)`" | $score |") | Out-Null
            }
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("_Results: $hitCount shown, $totalHits total_") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $checks = $checks | jq -c --arg q "$($q.query)" --argjson hc $hitCount '. += [{"Name": "Search \"\($q)\"", "Status": "pass", "Detail": "\($hc) results"}]'
        } else {
            $searchErrors++
            $queryResultParts.Add("### Query: `"$($q.query)`"") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("❌ No results or error") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $checks = $checks | jq -c --arg q "$($q.query)" '. += [{"Name": "Search \"\($q)\"", "Status": "warn", "Detail": "No results"}]'
        }
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "05-search-results"
    $errorCount = [int]($errorJson | jq 'length')
    $status = if ($searchErrors -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "05-search-results") -join "`n"

    # Score: % of queries returning results
    $score = 0
    if ($expectedTotal -gt 0) { $score = [int]($expectedFound * 100 / $expectedTotal) }

    $analysis = Gen-PhaseAnalysis "05-search-results" $checks
    $recommendations = Gen-PhaseRecs "05-search-results" $checks
    $prevScore = Get-PrevMetric "05-search-results" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg QUERY_RESULTS "$($queryResultParts -join "`n")" `
        --arg QUERY_COUNT $sampleQueries.Count `
        --arg EXPECTED_FOUND $expectedFound `
        --arg EXPECTED_TOTAL $expectedTotal `
        --arg SEARCH_ERRORS $searchErrors `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, QUERY_RESULTS: $QUERY_RESULTS, QUERY_COUNT: $QUERY_COUNT, EXPECTED_FOUND: $EXPECTED_FOUND, EXPECTED_TOTAL: $EXPECTED_TOTAL, SEARCH_ERRORS: $SEARCH_ERRORS, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE}'
    Write-Report "05-search-results.md" "05-search-results.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["05-search-results"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "05-search-results.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($expectedFound/$expectedTotal queries OK)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 8: Audit ──────────────────────────────────────────────────────────

function Phase-8-Audit {
    $Script:CURRENT_PHASE = "06-audit-findings"
    Write-Host "Phase 8: Audit..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    if ($Script:NoAudit) {
        Write-Host "  Skipped (NoAudit)" -ForegroundColor DarkGray
        $reportVals = jq -c -n `
            --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
            --arg DURATION 0 `
            --arg STATUS "⬜ SKIPPED" `
            --arg CHECKS_TABLE "| - | Audit | ⬜ | Skipped via -NoAudit |" `
            --arg ERRORS_TABLE "✅ No errors" `
            --arg AUDIT_SCORES_TABLE "| -- | -- | -- | -- | -- | -- |" `
            --arg FINDINGS_BY_DOMAIN "--" `
            --arg GATES_TABLE "| -- | -- | -- | -- | -- |" `
            --arg BLOCKED_GATES_DETAIL "--" `
            --arg DOMAIN_COUNT 0 `
            --arg TOTAL_FINDINGS 0 `
            --arg GATE_PASSES 0 `
            --arg GATE_TOTAL 0 `
            --arg GATE_BLOCKS 0 `
            --arg SCORE 0 `
            --arg TREND "—" `
            --arg ANALYSIS "Audit phase was skipped via -NoAudit flag." `
            --arg RECOMMENDATIONS "Run without -NoAudit to assess audit health." `
            --arg PREV_SCORE "" `
            '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, AUDIT_SCORES_TABLE: $AUDIT_SCORES_TABLE, FINDINGS_BY_DOMAIN: $FINDINGS_BY_DOMAIN, GATES_TABLE: $GATES_TABLE, BLOCKED_GATES_DETAIL: $BLOCKED_GATES_DETAIL, DOMAIN_COUNT: $DOMAIN_COUNT, TOTAL_FINDINGS: $TOTAL_FINDINGS, GATE_PASSES: $GATE_PASSES, GATE_TOTAL: $GATE_TOTAL, GATE_BLOCKS: $GATE_BLOCKS, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE}'
        Write-Report "06-audit-findings.md" "06-audit-findings.md" $reportVals | Out-Null
        $Script:PHASE_RESULTS["06-audit-findings"] = jq -c -n --arg status "⬜ SKIPPED" --argjson errors 0 --arg duration 0 --arg report "06-audit-findings.md" --argjson score 0 '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
        return
    }

    $domains = $Script:AllResults.Domains.Keys
    $domainNames = @($domains)
    $scoreRows = [System.Collections.ArrayList]::new()
    $findingsByDomain = [System.Collections.ArrayList]::new()
    $gatesRows = [System.Collections.ArrayList]::new()
    $blockedDetail = [System.Collections.ArrayList]::new()
    $totalFindings = 0
    $gatePasses = 0
    $gateTotal = 0
    $gateBlocks = 0

    foreach ($d in $domains) {
        Write-Host "  Auditing '$d'..." -ForegroundColor DarkGray

        # audit
        $auditResult = Invoke-McpTool -Name "audit" -Arguments @{ domain = $d }
        $overallScore = if ($auditResult -and $auditResult.score -ne $null) { $auditResult.score } else { $null }
        $findings = if ($auditResult -and $auditResult.findings) { $auditResult.findings } else { @() }

        if ($findings -is [array]) { $totalFindings += $findings.Count }

        # stage reports
        $stageScores = @{}
        foreach ($stage in @("deterministic", "section", "document", "cross_domain")) {
            $reportResult = Invoke-McpTool -Name "get_audit_report" -Arguments @{ domain = $d; stage = $stage }
            $s = if ($reportResult -and $reportResult.score -ne $null) { [math]::Round($reportResult.score, 2) } else { "--" }
            $stageScores[$stage] = $s
        }

        $scoreStr = if ($overallScore -ne $null) { [math]::Round($overallScore, 2) } else { "--" }
        $scoreRows.Add("| $d | $scoreStr | $($stageScores["deterministic"]) | $($stageScores["section"]) | $($stageScores["document"]) | $($stageScores["cross_domain"]) |") | Out-Null

        # findings
        if ($findings.Count -gt 0) {
            $findingsByDomain.Add("### $d ($($findings.Count) findings)") | Out-Null
            $findingsByDomain.Add("") | Out-Null
            $findingsByDomain.Add("| Check ID | Severity | Message |") | Out-Null
            $findingsByDomain.Add("|----------|----------|---------|") | Out-Null
            $fi = 0
            foreach ($f in $findings) {
                if ($fi -ge 20) { $findingsByDomain.Add("| ... | ... | _($($findings.Count - 20) more)_ |") | Out-Null; break }
                $sev = if ($f.severity) { $f.severity } else { "--" }
                $msg = if ($f.message) { ($f.message -replace '\|', '\|') } else { "--" }
                if ($msg.Length -gt 80) { $msg = $msg.Substring(0, 80) + "..." }
                $findingsByDomain.Add("| $($f.check_id) | $sev | $msg |") | Out-Null
                $fi++
            }
            $findingsByDomain.Add("") | Out-Null
        } else {
            $findingsByDomain.Add("### $d") | Out-Null
            $findingsByDomain.Add("") | Out-Null
            $findingsByDomain.Add("No findings") | Out-Null
            $findingsByDomain.Add("") | Out-Null
        }

        # gates
        $gateRow = @{}
        foreach ($stage in @("deterministic", "section", "document", "cross_domain")) {
            $gateResult = Invoke-McpTool -Name "check_gate" -Arguments @{ stage = $stage }
            $gateTotal++
            if ($gateResult -and $gateResult.blocked -eq $false) {
                $gateRow[$stage] = "✅"
                $gatePasses++
            } elseif ($gateResult -and $gateResult.blocked -eq $true) {
                $reason = if ($gateResult.reason) { $gateResult.reason } else { "blocked" }
                $gateRow[$stage] = "❌"
                $gateBlocks++
                $blockedDetail.Add("| $d | $stage | $reason |") | Out-Null
            } else {
                $gateRow[$stage] = "⚠️"
            }
        }
        $gatesRows.Add("| $d | $($gateRow["deterministic"]) | $($gateRow["section"]) | $($gateRow["document"]) | $($gateRow["cross_domain"]) |") | Out-Null
    }

    $checks = $checks | jq -c --argjson dc $domainNames.Count '. += [{"Name": "Domain audits", "Status": "pass", "Detail": "\($dc) domains audited"}]'
    $gd = if ($gateBlocks -gt 0) { "warn" } else { "pass" }
    $checks = $checks | jq -c --arg gd "$gd" --argjson gp $gatePasses --argjson gt $gateTotal '. += [{"Name": "Stage gates", "Status": $gd, "Detail": "\($gp)/\($gt) passed"}]'

    if ($blockedDetail.Count -eq 0) { $blockedDetail.Add("No blocked gates") | Out-Null }
    if ($scoreRows.Count -eq 0) { $scoreRows.Add("| -- | -- | -- | -- | -- | -- |") | Out-Null }
    if ($gatesRows.Count -eq 0) { $gatesRows.Add("| -- | -- | -- | -- | -- |") | Out-Null }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "06-audit-findings"
    $errorCount = [int]($errorJson | jq 'length')
    $status = if ($gateBlocks -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "06-audit-findings") -join "`n"

    # Score: gate pass rate (60%) + audit avg (40%)
    $score = 0
    if ($gateTotal -gt 0) {
        $gateScore = [int]($gatePasses * 60 / $gateTotal)
        $score = $gateScore
    }

    $analysis = Gen-PhaseAnalysis "06-audit-findings" $checks
    $recommendations = Gen-PhaseRecs "06-audit-findings" $checks
    $prevScore = Get-PrevMetric "06-audit-findings" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg AUDIT_SCORES_TABLE "$($scoreRows -join "`n")" `
        --arg FINDINGS_BY_DOMAIN "$($findingsByDomain -join "`n")" `
        --arg GATES_TABLE "$($gatesRows -join "`n")" `
        --arg BLOCKED_GATES_DETAIL "$($blockedDetail -join "`n")" `
        --arg DOMAIN_COUNT $domainNames.Count `
        --arg TOTAL_FINDINGS $totalFindings `
        --arg GATE_PASSES $gatePasses `
        --arg GATE_TOTAL $gateTotal `
        --arg GATE_BLOCKS $gateBlocks `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, AUDIT_SCORES_TABLE: $AUDIT_SCORES_TABLE, FINDINGS_BY_DOMAIN: $FINDINGS_BY_DOMAIN, GATES_TABLE: $GATES_TABLE, BLOCKED_GATES_DETAIL: $BLOCKED_GATES_DETAIL, DOMAIN_COUNT: $DOMAIN_COUNT, TOTAL_FINDINGS: $TOTAL_FINDINGS, GATE_PASSES: $GATE_PASSES, GATE_TOTAL: $GATE_TOTAL, GATE_BLOCKS: $GATE_BLOCKS, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE}'
    Write-Report "06-audit-findings.md" "06-audit-findings.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["06-audit-findings"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "06-audit-findings.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($totalFindings findings, $gatePasses/$gateTotal gates)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 9: Coverage Gaps ──────────────────────────────────────────────────

function Phase-9-Gaps {
    $Script:CURRENT_PHASE = "07-coverage-gaps"
    Write-Host "Phase 9: Coverage Gaps..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    $missingKnowledge = [System.Collections.ArrayList]::new()
    $emptySections = [System.Collections.ArrayList]::new()
    $lowQuality = [System.Collections.ArrayList]::new()
    $requiredMissing = [System.Collections.ArrayList]::new()

    $missingKnCount = 0
    $emptyCount = 0
    $lowQCount = 0
    $reqMissingCount = 0

    # Check knowledge coverage: if a section type exists for a domain but no knowledge file
    foreach ($d in $Script:AllResults.Domains.Keys) {
        $types = $Script:AllResults.Domains[$d].sectionTypes
        foreach ($t in $types.Keys) {
            if (-not $Script:NoAudit) {
                $knResult = Invoke-McpTool -Name "get_audit_knowledge" -Arguments @{ domain = $d; section_type = $t } -Quiet
                if (-not $knResult -or -not $knResult.content) {
                    $missingKnowledge.Add("| $d | $t | ❌ Missing |") | Out-Null
                    $missingKnCount++
                }
            }
        }
    }

    # Check for empty sections and low quality docs from Phase 4 data
    foreach ($item in $Script:AllResults.AllDocs) {
        $doc = $item.doc
        $quality = $doc.quality
        $bodyProp = $null
        if ($doc.body) {
            foreach ($prop in $doc.body.PSObject.Properties) { $bodyProp = $prop.Value; break }
        }

        $cov = if ($quality -and $quality.coverage -ne $null) { $quality.coverage } else { 1.0 }

        if ($quality -and $quality.empty_section_count -gt 0) {
            $lowQuality.Add("| $($doc.id) | `"$($doc.title)`" | $($item.domain) | $($quality.empty_section_count) empty, coverage $([math]::Round($cov * 100, 0))% |") | Out-Null
            $lowQCount++
        }

        if ($quality -and $quality.missing_section_count -gt 0) {
            $requiredMissing.Add("| $($doc.id) | `"$($doc.title)`" | $($item.domain) | $($quality.missing_section_count) missing |") | Out-Null
            $reqMissingCount++
        }

        if ($cov -lt 0.7 -and $cov -gt 0) {
            $lowQuality.Add("| $($doc.id) | `"$($doc.title)`" | $($item.domain) | coverage $([math]::Round($cov * 100, 0))% |") | Out-Null
            $lowQCount++
        }

        # Check for truly empty sections (body is empty or whitespace only)
        if ($bodyProp -and $bodyProp.sections) {
            $si = 0
            foreach ($s in $bodyProp.sections) {
                $body = $s.body
                if ($body -and $body.Trim().Length -eq 0) {
                    $emptySections.Add("| $($doc.id) | $si `"$($s.heading)`" | $($s.semantic_type) |") | Out-Null
                    $emptyCount++
                }
                $si++
            }
        }
    }

    $checks = $checks | jq -c --argjson mkc $missingKnCount '. += [{"Name": "Knowledge coverage", "Status": (if $mkc > 0 then "warn" else "pass" end), "Detail": "\($mkc) missing"}]'
    $checks = $checks | jq -c --argjson ec $emptyCount '. += [{"Name": "Empty sections", "Status": (if $ec > 0 then "warn" else "pass" end), "Detail": "\($ec) empty"}]'
    $checks = $checks | jq -c --argjson lqc $lowQCount '. += [{"Name": "Low quality docs", "Status": (if $lqc > 0 then "warn" else "pass" end), "Detail": "\($lqc) docs"}]'

    if ($missingKnowledge.Count -eq 0) { $missingKnowledge.Add("| -- | -- | All covered |") | Out-Null }
    if ($emptySections.Count -eq 0) { $emptySections.Add("| -- | -- | -- |") | Out-Null }
    if ($lowQuality.Count -eq 0) { $lowQuality.Add("| -- | -- | -- | -- |") | Out-Null }
    if ($requiredMissing.Count -eq 0) { $requiredMissing.Add("| -- | -- | -- | -- |") | Out-Null }

    $mkTable = "| Domain | Section Type | Status |`n|--------|-------------|--------|`n$($missingKnowledge -join "`n")"
    $esTable = "| Doc ID | Section | Type |`n|--------|---------|------|`n$($emptySections -join "`n")"
    $lqTable = "| Doc ID | Title | Domain | Issue |`n|--------|-------|--------|-------|`n$($lowQuality -join "`n")"
    $rmTable = "| Doc ID | Title | Domain | Missing |`n|--------|-------|--------|---------|`n$($requiredMissing -join "`n")"

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "07-coverage-gaps"
    $errorCount = [int]($errorJson | jq 'length')
    $status = if ($missingKnCount -gt 0 -or $emptyCount -gt 0 -or $lowQCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "07-coverage-gaps") -join "`n"

    # Score: lower gaps = better; max penalty 100 pts across gap types
    $gapTotal = $missingKnCount + $emptyCount + $lowQCount + $reqMissingCount
    $score = 100
    if ($gapTotal -gt 0) { $score = 100 - $gapTotal * 10 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "07-coverage-gaps" $checks
    $recommendations = Gen-PhaseRecs "07-coverage-gaps" $checks
    $prevScore = Get-PrevMetric "07-coverage-gaps" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg MISSING_KNOWLEDGE_TABLE "$mkTable" `
        --arg EMPTY_SECTIONS_TABLE "$esTable" `
        --arg LOW_QUALITY_TABLE "$lqTable" `
        --arg REQUIRED_MISSING_TABLE "$rmTable" `
        --arg MISSING_KNOWLEDGE_COUNT $missingKnCount `
        --arg EMPTY_SECTION_COUNT $emptyCount `
        --arg LOW_QUALITY_COUNT $lowQCount `
        --arg REQUIRED_MISSING_COUNT $reqMissingCount `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, MISSING_KNOWLEDGE_TABLE: $MISSING_KNOWLEDGE_TABLE, EMPTY_SECTIONS_TABLE: $EMPTY_SECTIONS_TABLE, LOW_QUALITY_TABLE: $LOW_QUALITY_TABLE, REQUIRED_MISSING_TABLE: $REQUIRED_MISSING_TABLE, MISSING_KNOWLEDGE_COUNT: $MISSING_KNOWLEDGE_COUNT, EMPTY_SECTION_COUNT: $EMPTY_SECTION_COUNT, LOW_QUALITY_COUNT: $LOW_QUALITY_COUNT, REQUIRED_MISSING_COUNT: $REQUIRED_MISSING_COUNT, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE}'
    Write-Report "07-coverage-gaps.md" "07-coverage-gaps.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["07-coverage-gaps"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "07-coverage-gaps.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($gapTotal gaps)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($gapTotal -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 10: Registry + Write-Tool Smoke ───────────────────────────────────

function Phase-10-Registry {
    $Script:CURRENT_PHASE = "08-registry-state"
    Write-Host "Phase 10: Registry State..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = '[]'

    # list_repositories
    $reposResult = Invoke-McpTool -Name "list_repositories" -Arguments @{ limit = 50 }
    $repoCount = if ($reposResult -and $reposResult.repositories) { $reposResult.repositories.Count } else { 0 }
    $checks = $checks | jq -c --argjson rc $repoCount '. += [{"Name": "List repositories", "Status": "pass", "Detail": "\($rc) repos"}]'

    $reposTableLines = [System.Collections.ArrayList]::new()
    $reposTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $reposTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($reposResult -and $reposResult.repositories) {
        $ri = 0
        foreach ($r in $reposResult.repositories) {
            $ri++
            $reposTableLines.Add("| $ri | $($r.id) | $($r.uuid) | $($r.status) |") | Out-Null
        }
    } else {
        $reposTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # resolve_dependencies
    $depsResult = Invoke-McpTool -Name "resolve_dependencies" -Arguments @{}
    $depCount = 0
    $unresolvedCount = 0
    $depsTableLines = [System.Collections.ArrayList]::new()
    $depsTableLines.Add("| Name | Path | Available | Required |") | Out-Null
    $depsTableLines.Add("|------|------|-----------|----------|") | Out-Null
    if ($depsResult -and $depsResult.dependencies) {
        $depCount = $depsResult.dependencies.Count
        foreach ($dep in $depsResult.dependencies) {
            $avail = if ($dep.available) { "✅" } else { "❌" }
            $req = if ($dep.required) { "yes" } else { "no" }
            $depsTableLines.Add("| $($dep.name) | $($dep.path) | $avail | $req |") | Out-Null
            if (-not $dep.available) { $unresolvedCount++ }
        }
    } else {
        $depsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }
    $checks = $checks | jq -c --argjson dc $depCount --argjson uc $unresolvedCount '. += [{"Name": "Resolve dependencies", "Status": "pass", "Detail": "\($dc) deps, \($uc) unresolved"}]'

    # workspace_status
    $wsResult = Invoke-McpTool -Name "workspace_status" -Arguments @{}
    $wsTableLines = [System.Collections.ArrayList]::new()
    $wsTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $wsTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($wsResult -and $wsResult.repositories) {
        $wi = 0
        foreach ($r in $wsResult.repositories) {
            $wi++
            $wsTableLines.Add("| $wi | $($r.id) | $($r.uuid) | $($r.status) |") | Out-Null
        }
    } else {
        $wsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # repository_status
    $rsResult = Invoke-McpTool -Name "repository_status" -Arguments @{ limit = 50 }
    $rsTableLines = [System.Collections.ArrayList]::new()
    $rsTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $rsTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($rsResult -and $rsResult.repositories) {
        $ri = 0
        foreach ($r in $rsResult.repositories) {
            $ri++
            $rsTableLines.Add("| $ri | $($r.id) | $($r.uuid) | $($r.status) |") | Out-Null
        }
    } else {
        $rsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # synchronize_repository
    $syncResult = Invoke-McpTool -Name "synchronize_repository" -Arguments @{}
    $checks = $checks | jq -c --arg r ($syncResult -ne $null) '. += [{"Name": "Synchronize repository", "Status": (if $r == "True" then "pass" else "warn" end), "Detail": "done"}]'

    # ── Write-tool smoke tests ──
    $writeToolLines = [System.Collections.ArrayList]::new()
    $writeToolLines.Add("| Tool | Input | Expected | Result |") | Out-Null
    $writeToolLines.Add("|------|-------|----------|--------|") | Out-Null
    $writePass = 0
    $writeTotal = 0

    # store_section_report with empty object → should fail validation
    $writeTotal++
    $srResult = Invoke-McpTool -Name "store_section_report" -Arguments @{ report_json = @{} } -Quiet
    if ($null -eq $srResult) { $writePass++; $writeToolLines.Add("| store_section_report | `{}` | reject bad input | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| store_section_report | `{}` | reject bad input | ❌ accepted |") | Out-Null }

    # store_document_report with empty object → should fail
    $writeTotal++
    $drResult = Invoke-McpTool -Name "store_document_report" -Arguments @{ report_json = @{} } -Quiet
    if ($null -eq $drResult) { $writePass++; $writeToolLines.Add("| store_document_report | `{}` | reject bad input | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| store_document_report | `{}` | reject bad input | ❌ accepted |") | Out-Null }

    # store_cross_domain_report with empty object → should fail
    $writeTotal++
    $crResult = Invoke-McpTool -Name "store_cross_domain_report" -Arguments @{ report_json = @{} } -Quiet
    if ($null -eq $crResult) { $writePass++; $writeToolLines.Add("| store_cross_domain_report | `{}` | reject bad input | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| store_cross_domain_report | `{}` | reject bad input | ❌ accepted |") | Out-Null }

    # update_finding_status with invalid data → should fail
    $writeTotal++
    $ufResult = Invoke-McpTool -Name "update_finding_status" -Arguments @{ report_id = 0; criterion_id = ""; status = "invalid" } -Quiet
    if ($null -eq $ufResult) { $writePass++; $writeToolLines.Add("| update_finding_status | invalid data | reject bad input | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| update_finding_status | invalid data | reject bad input | ❌ accepted |") | Out-Null }

    # register_repository with empty string → should fail
    $writeTotal++
    $rgResult = Invoke-McpTool -Name "register_repository" -Arguments @{ manifest = "{}" } -Quiet
    if ($null -eq $rgResult) { $writePass++; $writeToolLines.Add("| register_repository | `{}` manifest | reject bad input | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| register_repository | `{}` manifest | reject bad input | ❌ accepted |") | Out-Null }

    # unregister_repository with bad uuid → should fail
    $writeTotal++
    $urResult = Invoke-McpTool -Name "unregister_repository" -Arguments @{ uuid = "00000000-0000-0000-0000-000000000000" } -Quiet
    if ($null -eq $urResult) { $writePass++; $writeToolLines.Add("| unregister_repository | bogus UUID | reject not-found | ✅ rejected |") | Out-Null }
    else { $writeToolLines.Add("| unregister_repository | bogus UUID | reject not-found | ❌ accepted |") | Out-Null }

    $ws = if ($writePass -eq $writeTotal) { "pass" } else { "warn" }
    $checks = $checks | jq -c --arg ws "$ws" --argjson wp $writePass --argjson wt $writeTotal '. += [{"Name": "Write-tool validation", "Status": $ws, "Detail": "\($wp)/\($wt) pass"}]'

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "08-registry-state"
    $errorCount = [int]($errorJson | jq 'length')
    $status = if ($writePass -ne $writeTotal) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksTable = (Get-ChecksTable $checks) -join "`n"
    $errorsTable = (Get-ErrorsTable "08-registry-state") -join "`n"

    # Score: write-tool pass rate (70%) + dependencies resolved (30%)
    $subScore = [int]($writePass * 70 / [Math]::Max($writeTotal, 1))
    $depTotal = $depCount + $unresolvedCount
    $depScore = 0
    if ($depTotal -gt 0) { $depScore = [int]($depCount * 30 / $depTotal) }
    $score = $subScore + $depScore

    $analysis = Gen-PhaseAnalysis "08-registry-state" $checks
    $recommendations = Gen-PhaseRecs "08-registry-state" $checks
    $prevScore = Get-PrevMetric "08-registry-state" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg DURATION $duration `
        --arg STATUS "$status" `
        --arg CHECKS_TABLE "$checksTable" `
        --arg ERRORS_TABLE "$errorsTable" `
        --arg REPOS_TABLE "$($reposTableLines -join "`n")" `
        --arg DEPS_TABLE "$($depsTableLines -join "`n")" `
        --arg WORKSPACE_TABLE "$($wsTableLines -join "`n")" `
        --arg REPO_STATUS_TABLE "$($rsTableLines -join "`n")" `
        --arg WRITE_TOOL_TABLE "$($writeToolLines -join "`n")" `
        --arg REPO_COUNT $repoCount `
        --arg DEP_COUNT $depCount `
        --arg UNRESOLVED_COUNT $unresolvedCount `
        --arg WRITE_PASS $writePass `
        --arg WRITE_TOTAL $writeTotal `
        --arg SCORE $score `
        --arg TREND "$trend" `
        --arg ANALYSIS "$analysis" `
        --arg RECOMMENDATIONS "$recommendations" `
        --arg PREV_SCORE "$prevScore" `
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS, CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE, REPOS_TABLE: $REPOS_TABLE, DEPS_TABLE: $DEPS_TABLE, WORKSPACE_TABLE: $WORKSPACE_TABLE, REPO_STATUS_TABLE: $REPO_STATUS_TABLE, WRITE_TOOL_TABLE: $WRITE_TOOL_TABLE, REPO_COUNT: $REPO_COUNT, DEP_COUNT: $DEP_COUNT, UNRESOLVED_COUNT: $UNRESOLVED_COUNT, WRITE_PASS: $WRITE_PASS, WRITE_TOTAL: $WRITE_TOTAL, SCORE: $SCORE, TREND: $TREND, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS, PREV_SCORE: $PREV_SCORE}'
    Write-Report "08-registry-state.md" "08-registry-state.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["08-registry-state"] = jq -c -n --arg status "$status" --argjson errors $errorCount --argjson duration $duration --arg report "08-registry-state.md" --argjson score $score '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}'
    Write-Host "  → Score: $score/100 $trend — $status ($writePass/$writeTotal write-tool)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 11: Summary ───────────────────────────────────────────────────────

function Phase-11-Summary {
    $Script:CURRENT_PHASE = "00-summary"
    Write-Host "Phase 11: Summary..." -ForegroundColor Cyan
    $start = Get-Date

    $phaseRanks = @{
        "01-tool-health" = 1; "02-domain-catalog" = 2; "03-document-audit" = 3
        "04-section-integrity" = 4; "05-search-results" = 5; "06-audit-findings" = 6
        "07-coverage-gaps" = 7; "08-registry-state" = 8
    }

    $phaseOrder = @("01-tool-health", "02-domain-catalog", "03-document-audit", "04-section-integrity", "05-search-results", "06-audit-findings", "07-coverage-gaps", "08-registry-state")

    $phaseRows = [System.Collections.ArrayList]::new()
    $failedPhases = [System.Collections.ArrayList]::new()
    $totalErrors = 0
    $totalDuration = 0
    $scoreSum = 0
    $scoreCount = 0

    foreach ($key in $phaseOrder) {
        if (-not $Script:PHASE_RESULTS.ContainsKey($key)) { continue }
        $pr = $Script:PHASE_RESULTS[$key]
        $reportFile = $pr | jq -r '.ReportFile // ""'
        if ([string]::IsNullOrEmpty($reportFile)) { continue }
        $name = $reportFile -replace '\.md$', ''
        $status = $pr | jq -r '.Status // "?"'
        $errors = [int]($pr | jq -r '.Errors // 0')
        $pDuration = [int]($pr | jq -r '.Duration // 0')
        $ps = $pr | jq -r '.Score // empty'
        $phaseRows.Add("| $name | $reportFile | $status | $errors | ${pDuration}s |") | Out-Null
        $totalErrors += $errors
        $totalDuration += $pDuration
        if ($status -match "FAIL|PARTIAL") {
            $failedPhases.Add("- **$reportFile**: $status ($errors errors)") | Out-Null
        }
        if (-not [string]::IsNullOrEmpty($ps) -and $ps -ne "null") {
            $scoreSum += [int]$ps
            $scoreCount++
        }
    }

    # Gather stats
    $toolCount = if ($Script:AllResults.Tools) { $Script:AllResults.Tools.Count } else { 0 }
    $domainCount = if ($Script:AllResults.Domains) { $Script:AllResults.Domains.Keys.Count } else { 0 }
    $docCount = $Script:AllResults.TotalDocs
    $sectCount = $Script:AllResults.TotalSections
    $sectTypeCount = if ($Script:AllResults.SectionsByType) { $Script:AllResults.SectionsByType.Keys.Count } else { 0 }

    $overallStatus = if ($totalErrors -gt 0) { "⚠️ WITH ERRORS" } else { "✅ CLEAN" }

    # Compute total score = floor of average of all phase scores
    $totalScore = 0
    if ($scoreCount -gt 0) { $totalScore = [int]($scoreSum / $scoreCount) }

    if ($phaseRows.Count -eq 0) { $phaseRows.Add("| -- | -- | -- | -- | -- |") | Out-Null }
    if ($failedPhases.Count -eq 0) { $failedPhases.Add("✅ All phases passed") | Out-Null }

    $archiveStr = if ($Script:ArchivePath) { $Script:ArchivePath } else { "No previous run" }

    # Load previous metrics for historical comparison
    $prevTotalScore = Get-PrevMetric "00-summary" "score"
    if ([string]::IsNullOrEmpty($prevTotalScore)) { $prevTotalScore = "" }
    $totalTrend = Trend-Between $totalScore $prevTotalScore

    # Generate overall analysis and recommendations
    $overallAnalysis = ""
    $overallRecs = ""
    if ($scoreCount -gt 0) {
        $minScore = 100; $minPhase = ""; $maxScore = 0; $maxPhase = ""
        foreach ($key in $phaseOrder) {
            if (-not $Script:PHASE_RESULTS.ContainsKey($key)) { continue }
            $pr = $Script:PHASE_RESULTS[$key]
            $ps = [int]($pr | jq -r '.Score // 0')
            $rpt = $pr | jq -r '.ReportFile // ""'
            if ($ps -lt $minScore) { $minScore = $ps; $minPhase = $rpt }
            if ($ps -gt $maxScore) { $maxScore = $ps; $maxPhase = $rpt }
        }
        $overallAnalysis = "Total score **$totalScore/100** across $scoreCount phases. "
        if ($minPhase) { $overallAnalysis += "Lowest: **$minPhase** ($minScore). " }
        if ($maxPhase) { $overallAnalysis += "Highest: **$maxPhase** ($maxScore). " }
        if ($totalErrors -gt 0) { $overallAnalysis += "$totalErrors total errors detected." }
        if ($prevTotalScore) { $overallAnalysis += " Previous total: **$prevTotalScore/100**." }

        $lowPhases = 0
        foreach ($key in $phaseOrder) {
            if (-not $Script:PHASE_RESULTS.ContainsKey($key)) { continue }
            $pr = $Script:PHASE_RESULTS[$key]
            $ps = [int]($pr | jq -r '.Score // 0')
            if ($ps -lt 60) { $lowPhases++ }
        }
        if ($lowPhases -gt 0) { $overallRecs += "- Investigate $lowPhases phase(s) scoring below 60.`n" }
        if ($totalErrors -gt 0) { $overallRecs += "- Address $totalErrors error(s) across phases.`n" }
        if ([string]::IsNullOrEmpty($overallRecs)) { $overallRecs = "- All phases performing well. Maintain current practices.`n" }
    }

    $reportVals = jq -c -n `
        --arg TIMESTAMP "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --arg TOTAL_DURATION $totalDuration `
        --arg OVERALL_STATUS "$overallStatus" `
        --arg PHASE_RESULTS_ROWS "$($phaseRows -join "`n")" `
        --arg TOOL_COUNT $toolCount `
        --arg DOMAIN_COUNT $domainCount `
        --arg DOCUMENT_COUNT $docCount `
        --arg SECTION_COUNT $sectCount `
        --arg SECTION_TYPE_COUNT $sectTypeCount `
        --arg TOTAL_CALLS $Script:TOTAL_CALLS `
        --arg TOTAL_ERRORS $totalErrors `
        --arg FAILED_PHASES "$($failedPhases -join "`n")" `
        --arg ARCHIVE_PATH "$archiveStr" `
        --arg SCORE $totalScore `
        --arg TREND "$totalTrend" `
        --arg PREV_SCORE "$prevTotalScore" `
        --arg ANALYSIS "$overallAnalysis" `
        --arg RECOMMENDATIONS "$overallRecs" `
        '{TIMESTAMP: $TIMESTAMP, TOTAL_DURATION: $TOTAL_DURATION, OVERALL_STATUS: $OVERALL_STATUS, PHASE_RESULTS_ROWS: $PHASE_RESULTS_ROWS, TOOL_COUNT: $TOOL_COUNT, DOMAIN_COUNT: $DOMAIN_COUNT, DOCUMENT_COUNT: $DOCUMENT_COUNT, SECTION_COUNT: $SECTION_COUNT, SECTION_TYPE_COUNT: $SECTION_TYPE_COUNT, TOTAL_CALLS: $TOTAL_CALLS, TOTAL_ERRORS: $TOTAL_ERRORS, FAILED_PHASES: $FAILED_PHASES, ARCHIVE_PATH: $ARCHIVE_PATH, SCORE: $SCORE, TREND: $TREND, PREV_SCORE: $PREV_SCORE, ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS}'
    Write-Report "00-summary.md" "00-summary.md" $reportVals | Out-Null

    # Build and save metrics JSON for next run
    $psJson = Build-PhaseScoresJson
    $allJq = $Script:AllResults | ConvertTo-Json -Compress -Depth 10 | jq -c 'if type == "object" then . else null end'
    $metrics = jq -c -n `
        --arg ts "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" `
        --argjson ps "$psJson" `
        --argjson ts_score $totalScore `
        --argjson tc $toolCount `
        --argjson dc $docCount `
        --argjson sc $sectCount `
        --argjson stc $sectTypeCount `
        --argjson dmc $domainCount `
        --argjson ec $totalErrors `
        --argjson ttl $Script:TOTAL_CALLS `
        '{timestamp: $ts, phase_scores: $ps, total_score: $ts_score, metrics: {tool_count: $tc, doc_count: $dc, section_count: $sc, section_type_count: $stc, domain_count: $dmc, error_count: $ec, total_calls: $ttl}}'
    $metrics | Set-Content -Path (Join-Path $Script:LATEST_DIR "metrics.json") -Encoding UTF8

    $Script:PHASE_RESULTS["00-summary"] = jq -c -n --arg status "✅ DONE" --argjson errors $totalErrors --argjson score $totalScore --argjson duration $totalDuration --arg report "00-summary.md" '{Status: $status, Errors: $errors, Score: $score, Duration: $duration, ReportFile: $report}'

    Write-Host "  → Total Score: $totalScore/100 $totalTrend — Done" -ForegroundColor Green
}

# ─── Main ─────────────────────────────────────────────────────────────────────

Write-Host @"

╔═══════════════════════════════════════════╗
║    Samgraha MCP Discovery                ║
║    $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')              ║
╚═══════════════════════════════════════════╝
"@ -ForegroundColor Cyan

$mainStart = Get-Date

try {
    Phase-1-Bootstrap
    Phase-2-DomainScan
    Phase-3-DocDiscover
    Phase-4-DocVerify

    if ($Script:AllResults.TotalSections -gt 0) {
        Phase-5-CrossSection
        Phase-6-SectionVerify
    } else {
        Write-Host "Phase 5-6: No sections to verify (skipping)" -ForegroundColor DarkGray
    }

    Phase-7-Search
    Phase-8-Audit
    Phase-9-Gaps
    Phase-10-Registry
    Phase-11-Summary

    $totalMain = (Get-Date) - $mainStart

    Write-Host @"

╔═══════════════════════════════════════════╗
║  Complete: $($totalMain.TotalSeconds.ToString('0.0'))s          ║
║  Reports: $($Script:LATEST_DIR)          ║
╚═══════════════════════════════════════════╝
"@ -ForegroundColor Cyan

    if ($PassThru) {
        Write-Host "`nReport files:" -ForegroundColor Cyan
        Get-ChildItem $Script:LATEST_DIR -Filter "*.md" | Sort-Object Name | ForEach-Object {
            Write-Host "  $($_.FullName)" -ForegroundColor White
        }
    }

} catch {
    Write-Host "❌ Fatal error: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkRed
    exit 1
}
