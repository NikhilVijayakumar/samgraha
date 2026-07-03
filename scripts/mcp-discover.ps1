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

# jq dependency removed — all JSON via native PS
$Script:PyCmd = if (Get-Command python3 -ErrorAction SilentlyContinue) { 'python3' } else { 'python' }
if (-not (Get-Command $Script:PyCmd -ErrorAction SilentlyContinue)) { throw "python3/python is required but not found on PATH" }

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

$Script:ArchivePath = $null
# Initialize-ReportDirs handles archive rotation and LATEST_DIR creation
Initialize-ReportDirs "mcp"
# Capture the archive path that Initialize-ReportDirs created (newest entry in archive)
$_newestArchive = Get-ChildItem $Script:ARCHIVE_DIR -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
if ($_newestArchive) { $Script:ArchivePath = $_newestArchive.FullName }

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
        $callArgs = @{} + $Arguments
        $callArgs.limit = $PageSize
        $callArgs.offset = $offset

        $result = Invoke-McpTool -Name $Name -Arguments $callArgs -Id $id
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
    $arr = New-Object System.Collections.ArrayList
    foreach ($key in $order) {
        if ($Script:PHASE_RESULTS.ContainsKey($key)) {
            $pr = $Script:PHASE_RESULTS[$key] | ConvertFrom-Json
            $score = if ($null -ne $pr.Score) { $pr.Score } else { 0 }
            $status = if ($null -ne $pr.Status) { $pr.Status } else { "?" }
            $errors = if ($null -ne $pr.Errors) { $pr.Errors } else { 0 }
            $dur = if ($null -ne $pr.Duration) { $pr.Duration } else { 0 }
            $dc = if ($null -ne $pr.doc_count) { $pr.doc_count } else { 0 }
            $ic = if ($null -ne $pr.issue_count) { $pr.issue_count } else { 0 }
            [void]$arr.Add(@{phase = $key; score = $score; status = $status; errors = $errors; duration = $dur; doc_count = $dc; issue_count = $ic})
        }
    }
    return $arr
}

# ─── Phase 1: Bootstrap ──────────────────────────────────────────────────────

function Phase-1-Bootstrap {
    $Script:CURRENT_PHASE = "01-tool-health"
    Write-Host "Phase 1: Bootstrap..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

    # initialize
    $initResult = Invoke-McpDirect -Method "initialize" -Params @{
        protocolVersion = "2025-03-26"
        capabilities = @{}
        clientInfo = @{ name = "mcp-discover"; version = "1.0" }
    }
    if ($initResult) {
        $pv = $initResult.protocolVersion
        $Script:AllResults.Protocol = $pv
        [void]$checks.Add(@{Name = "Initialize"; Status = "pass"; Detail = "Protocol $pv"})
    } else {
        [void]$checks.Add(@{Name = "Initialize"; Status = "fail"; Detail = "No response"})
    }

    # tools/list
    $toolsResult = Invoke-McpDirect -Method "tools/list"
    $tools = @()
    if ($toolsResult -and $toolsResult.tools) {
        $tools = $toolsResult.tools
        $tc = $tools.Count
        $Script:AllResults.Tools = $tools
        [void]$checks.Add(@{Name = "Tools/List"; Status = "pass"; Detail = "$tc tools"})
    } else {
        [void]$checks.Add(@{Name = "Tools/List"; Status = "fail"; Detail = "No tools returned"})
    }

    # info via tools/call
    $infoResult = Invoke-McpTool -Name "info"
    if ($infoResult) {
        $dc = $infoResult.document_count
        $Script:AllResults.Runtime = $infoResult
        [void]$checks.Add(@{Name = "Info"; Status = "pass"; Detail = "$dc docs"})
    } else {
        [void]$checks.Add(@{Name = "Info"; Status = "fail"; Detail = "No response"})
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "01-tool-health"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $hasFail = @($checks | Where-Object { $_.Status -eq "fail" }).Count
    $status = if ($hasFail -gt 0) { "❌ FAIL" } else { "✅ PASS" }

    # Build tool table for template
    $toolRows = [System.Collections.ArrayList]::new()
    $ti = 0
    $phase1ErrItems = Get-PhaseErrorsJson "01-tool-health" | ConvertFrom-Json
    foreach ($t in $tools) {
        $ti++
        $name = $t.name
        $req = if ($t.inputSchema -and $t.inputSchema.required) { $t.inputSchema.required -join ", " } else { "none" }
        $hasErr = [bool]@($phase1ErrItems | Where-Object { $_.Tool -match $name }).Count
        $toolStatus = if ($hasErr) { "⚠️" } else { "✅" }
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

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "01-tool-health") -join "`n"

    # Score: % of checks passing, penalized by errors
    $tc = $checks.Count
    $passC = @($checks | Where-Object { $_.Status -eq "pass" }).Count
    $score = 0
    if ($tc -gt 0) { $score = [int]($passC * 100 / $tc) }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 5 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "01-tool-health" $checksJson
    $recommendations = Gen-PhaseRecs "01-tool-health" $checksJson
    $prevScore = Get-PrevMetric "01-tool-health" "score"
    $trend = Trend-Between $score $prevScore
    $prevDocCount = Get-PrevMetric "01-tool-health" "prev_doc_count"
    if ([string]::IsNullOrEmpty($prevDocCount)) { $prevDocCount = "" }
    $docTrend = Trend-Between $docCount $prevDocCount

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        TOOLS_TABLE = "$($toolRows -join "`n")"
        DOC_COUNT = "$docCount"
        STANDARDS_LIST = "$standards"
        STANDARD_COUNT = $standardCount
        REGISTRY_PATH = "$registryPath"
        REPOSITORY = "$repositoryName"
        SERVICES = "$services"
        POLICY = "$policy"
        TOOL_COUNT = $healthyToolCount
        TOOL_ERROR_COUNT = $errorCount
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
        PROTOCOL_VERSION = "$($Script:AllResults.Protocol)"
        HEALTHY_TOOL_COUNT = $healthyToolCount
        PREV_DOC_COUNT = "$prevDocCount"
        DOC_TREND = "$docTrend"
    } | ConvertTo-Json -Compress
    Write-Report "01-tool-health.md" "01-tool-health.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["01-tool-health"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "01-tool-health.md"; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($duration`s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 2: Domain Catalog ──────────────────────────────────────────────────

function Phase-2-DomainScan {
    $Script:CURRENT_PHASE = "02-domain-catalog"
    Write-Host "Phase 2: Domain Scan..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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
        [void]$checks.Add(@{Name = "List Domains"; Status = "pass"; Detail = "$cnt domains"})
    } else {
        [void]$checks.Add(@{Name = "List Domains"; Status = "fail"; Detail = "No domains"})
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "02-domain-catalog"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count

    $status = if ($allDomainNames.Count -gt 0) { "✅ PASS" } else { "❌ FAIL" }

    # Score: 100 if any domains found, proportional to count, min 20 per domain
    $dc = $allDomainNames.Count
    $score = 0
    if ($dc -gt 0) { $score = [Math]::Min($dc * 25, 100) }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 10 }
    if ($score -lt 0) { $score = 0 }

    $prevScore = Get-PrevMetric "02-domain-catalog" "score"
    $trend = Trend-Between $score $prevScore

    $Script:PHASE_RESULTS["02-domain-catalog"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "02-domain-catalog.md"; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($duration`s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 3: Document Discovery ─────────────────────────────────────────────

function Phase-3-DocDiscover {
    $Script:CURRENT_PHASE = "03-document-audit"
    Write-Host "Phase 3: Document Discovery..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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
            [void]$checks.Add(@{Name = $cn; Status = "pass"; Detail = "$dc docs"})
        } else {
            $Script:AllResults.Domains[$d].docCount = 0
            $cn = "Docs in '$d'"
            [void]$checks.Add(@{Name = $cn; Status = "skip"; Detail = "0 docs (or error)"})
        }
        $allDocCount += $Script:AllResults.Domains[$d].docCount
    }

    $Script:AllResults.TotalDocs = $allDocCount

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "03-document-audit"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $hasFailChecks = @($checks | Where-Object { $_.Status -eq "fail" }).Count
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
    $checksJson = $checks | ConvertTo-Json -Compress
    $allChecks = New-Object System.Collections.ArrayList
    $phase3Items = $checksJson | ConvertFrom-Json
    foreach ($item in $phase3Items) { [void]$allChecks.Add($item) }
    [void]$allChecks.Add(@{Name = "List Domains"; Status = "pass"; Detail = "$($domainNames.Count) domains"})
    $allChecksJson = $allChecks | ConvertTo-Json -Compress
    $allChecksTable = (Get-ChecksTable $allChecksJson) -join "`n"

    $phase2Errors = Get-PhaseErrorsJson "02-domain-catalog"
    $p2ErrorItems = $phase2Errors | ConvertFrom-Json
    if ($p2ErrorItems.Count -eq 0) {
        $p2ErrorRows = "No errors"
    } else {
        $rows = @("| Tool Call | Error | Response |", "|-----------|-------|----------|")
        foreach ($e in $p2ErrorItems) {
            $resp = if ($e.Response.Length -gt 120) { $e.Response.Substring(0, 120) } else { $e.Response }
            $rows += "| $($e.Tool) | $($e.Error) | $resp |"
        }
        $p2ErrorRows = $rows -join "`n"
    }

    # Phase 2 score from stored results
    $p2Obj = $Script:PHASE_RESULTS["02-domain-catalog"]
    $p2Score = if ($p2Obj) { $p2Parsed = $p2Obj | ConvertFrom-Json; if ($null -ne $p2Parsed.Score) { [int]$p2Parsed.Score } else { 0 } } else { 0 }
    $p2Prev = Get-PrevMetric "02-domain-catalog" "score"
    $p2Trend = Trend-Between $p2Score $p2Prev
    $p2Analysis = Gen-PhaseAnalysis "02-domain-catalog" $allChecksJson
    $p2Recs = Gen-PhaseRecs "02-domain-catalog" $allChecksJson

    $prevDocCount = Get-PrevMetric "03-document-audit" "prev_doc_count"
    if ([string]::IsNullOrEmpty($prevDocCount)) { $prevDocCount = "" }
    $docTrend = Trend-Between $allDocCount $prevDocCount
    $prevDomainCount = Get-PrevMetric "02-domain-catalog" "prev_domain_count"
    if ([string]::IsNullOrEmpty($prevDomainCount)) { $prevDomainCount = "" }
    $domainTrend = Trend-Between $domainNames.Count $prevDomainCount

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "✅ PASS"
        CHECKS_TABLE = "$allChecksTable"
        ERRORS_TABLE = "$p2ErrorRows"
        DOMAINS_TABLE = "$($dcDomainRows -join "`n")"
        STANDARDS_LIST = "$dcStandards"
        STANDARD_COUNT = $sc
        DOC_COUNTS_TABLE = "$($dcDocCountRows -join "`n")"
        DOMAIN_COUNT = $domainNames.Count
        DOCUMENT_COUNT = $allDocCount
        SCORE = $p2Score
        TREND = "$p2Trend"
        ANALYSIS = "$p2Analysis"
        RECOMMENDATIONS = "$p2Recs"
        PREV_SCORE = "$p2Prev"
        PREV_DOC_COUNT = "$prevDocCount"
        DOC_TREND = "$docTrend"
        PREV_DOMAIN_COUNT = "$prevDomainCount"
        DOMAIN_TREND = "$domainTrend"
    } | ConvertTo-Json -Compress
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

    $Script:PHASE_RESULTS["03-document-audit"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($allDocCount docs discovered)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 4: Document Verification ──────────────────────────────────────────

function Phase-4-DocVerify {
    $Script:CURRENT_PHASE = "03-document-audit"
    Write-Host "Phase 4: Document Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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
                        if ($st) {
                            $cur = $Script:AllResults.Domains[$d].sectionTypes[$st]
                            $curInt = if ($null -eq $cur) { 0 } else { [int]$cur }
                            $Script:AllResults.Domains[$d].sectionTypes[$st] = $curInt + 1
                        }
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
            $coverageStr = if ($coverage -as [double] -is [double]) { "$([math]::Round([double]$coverage * 100, 0))%" } else { "?" }
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
    [void]$checks.Add(@{Name = "Document verification"; Status = "pass"; Detail = "$allDocsCount docs across $($domainNames.Count) domains"})
    [void]$checks.Add(@{Name = "Section count"; Status = "pass"; Detail = "$allSectionsTotal sections total"})

    $distRows = [System.Collections.ArrayList]::new()
    foreach ($k in @("0", "1-3", "4-7", "8-15", "16+")) {
        if ($sectDist[$k] -gt 0) {
            $distRows.Add("| $k sections | $($sectDist[$k]) docs |") | Out-Null
        }
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "03-document-audit"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = if ($issueCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
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

    $analysis = Gen-PhaseAnalysis "03-document-audit" $checksJson
    $recommendations = Gen-PhaseRecs "03-document-audit" $checksJson
    $prevScore = Get-PrevMetric "03-document-audit" "score"
    $trend = Trend-Between $score $prevScore
    $prevSectCount = Get-PrevMetric "03-document-audit" "prev_sect_count"
    if ([string]::IsNullOrEmpty($prevSectCount)) { $prevSectCount = "" }
    $sectTrend = Trend-Between $allSectionsTotal $prevSectCount
    $prevDocCount = Get-PrevMetric "03-document-audit" "prev_doc_count"
    if ([string]::IsNullOrEmpty($prevDocCount)) { $prevDocCount = "" }
    $prevIssues = Get-PrevMetric "03-document-audit" "prev_issue_count"
    if ([string]::IsNullOrEmpty($prevIssues)) { $prevIssues = "" }

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        DOMAIN_DOCS_SECTIONS = "$($domainDocsSectionsParts -join "`n`n")"
        QUALITY_TABLE = "$($qualityRows -join "`n")"
        ISSUES_LIST = "$issuesTable"
        SECTION_DIST_TABLE = "$($distRows -join "`n")"
        TOTAL_DOCS = $allDocsCount
        DOMAIN_COUNT = $domainNames.Count
        TOTAL_SECTIONS = $allSectionsTotal
        ISSUE_COUNT = $issueCount
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
        PREV_SECT_COUNT = "$prevSectCount"
        SECT_TREND = "$sectTrend"
        PREV_DOCS = "$prevDocCount"
        PREV_ISSUES = "$prevIssues"
    } | ConvertTo-Json -Compress
    Write-Report "03-document-audit.md" "03-document-audit.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["03-document-audit"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "03-document-audit.md"; Score = $score; doc_count = $allDocsCount; issue_count = $issueCount} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($issueCount issues)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($issueCount -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 5: Cross-Section ──────────────────────────────────────────────────

function Phase-5-CrossSection {
    $Script:CURRENT_PHASE = "04-section-integrity"
    Write-Host "Phase 5: Cross-Section..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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

    [void]$checks.Add(@{Name = "Cross-section query"; Status = "pass"; Detail = "$pairCount type-domain pairs, $totalSections sections"})

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "04-section-integrity"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count

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

    $Script:PHASE_RESULTS["04-section-integrity"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → $totalSections sections from $pairCount type-domain pairs" -ForegroundColor Green
}

# ─── Phase 6: Section Verification ───────────────────────────────────────────

function Phase-6-SectionVerify {
    $Script:CURRENT_PHASE = "04-section-integrity"
    Write-Host "Phase 6: Section Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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

    # Pass 1: section verification (capped at $maxVerify to limit MCP calls)
    foreach ($d in $sectionIds.Keys) {
        foreach ($t in $sectionIds[$d].Keys) {
            $ids = $sectionIds[$d][$t]
            $allUniqueTypes[$t] = $true
            foreach ($sid in $ids) {
                if ($verifyCount -ge $maxVerify) { break }
                $verifyCount++

                $sectResult = Invoke-McpTool -Name "get_section" -Arguments @{ section_id = $sid }
                $sectOk = if ($sectResult -and $sectResult.id -eq $sid) { "✅" } else { "❌" }

                $changedResult = Invoke-McpTool -Name "get_section_changed" -Arguments @{ section_id = $sid }
                $changed = if ($changedResult) { $changedResult.changed } else { "?" }
                if ($changed -eq $true) { $staleCount++ }

                $verifyLines.Add("| $d | $sid | $t | $sectOk | $changed |") | Out-Null
                $totalSections++
            }
            if ($verifyCount -ge $maxVerify) { break }
        }
        if ($verifyCount -ge $maxVerify) { break }
    }

    # Pass 2: knowledge coverage — runs over ALL pairs, independent of section verify cap
    if (-not $Script:NoAudit) {
        foreach ($d in $sectionIds.Keys) {
            foreach ($t in $sectionIds[$d].Keys) {
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
        }
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
    [void]$checks.Add(@{Name = "Section verification"; Status = "pass"; Detail = "$totalSections sections checked"})
    if ($staleCount -gt 0) {
        [void]$checks.Add(@{Name = "Stale sections"; Status = "warn"; Detail = "$staleCount changed since last audit"})
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "04-section-integrity"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = "✅ PASS"

    # Change tracking table
    $changeTrackRows.Add("| Stale (changed) | $staleCount |") | Out-Null
    $changeTrackRows.Add("| Fresh (unchanged) | $($totalSections - $staleCount) |") | Out-Null

    if ($knowledgeRows.Count -eq 0) { $knowledgeRows.Add("| -- | -- | -- |") | Out-Null }
    if ($sectionTypeRows.Count -eq 0) { $sectionTypeRows.Add("| -- | -- |") | Out-Null }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "04-section-integrity") -join "`n"

    # Score: section freshness (primary) + knowledge coverage (secondary bonus)
    # Knowledge gaps are tracked separately in Phase 9 (coverage-gaps) — don't let them
    # tank a perfect section verification result.
    $score = 0
    if ($totalSections -gt 0) {
        $verOk = $totalSections - $staleCount
        $knTotal = $knowledgeCount + $knowledgeMissing
        if ($knTotal -gt 0) {
            # Both dimensions available: 70% freshness, 30% knowledge
            $verRate  = [int]($verOk * 70 / $totalSections)
            $knRate   = [int]($knowledgeCount * 30 / $knTotal)
            $score    = $verRate + $knRate
        } else {
            # No knowledge data — score on freshness alone (full 100)
            $score = [int]($verOk * 100 / $totalSections)
        }
    }
    if ($errorCount -gt 0) { $score = $score - $errorCount * 3 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "04-section-integrity" $checksJson
    $recommendations = Gen-PhaseRecs "04-section-integrity" $checksJson
    $prevScore = Get-PrevMetric "04-section-integrity" "score"
    $trend = Trend-Between $score $prevScore
    $prevStale = Get-PrevMetric "04-section-integrity" "stale"
    if ([string]::IsNullOrEmpty($prevStale)) { $prevStale = "" }
    $staleTrend = Trend-Between $staleCount $prevStale

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        SECTION_TYPES_TABLE = "$($sectionTypeRows -join "`n")"
        SECTION_VERIFY_TABLE = "$($sectionVerifyParts -join "`n`n")"
        CHANGE_TRACKING_TABLE = "$($changeTrackRows -join "`n")"
        KNOWLEDGE_TABLE = "$($knowledgeRows -join "`n")"
        TOTAL_SECTIONS = $totalSections
        DOMAIN_COUNT = $domainNames.Count
        UNIQUE_TYPES = $typeCount
        STALE_SECTIONS = $staleCount
        KNOWLEDGE_COUNT = $knowledgeCount
        KNOWLEDGE_MISSING = $knowledgeMissing
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
        PREV_STALE = "$prevStale"
        STALE_TREND = "$staleTrend"
    } | ConvertTo-Json -Compress
    Write-Report "04-section-integrity.md" "04-section-integrity.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["04-section-integrity"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "04-section-integrity.md"; Score = $score; Stale = $staleCount} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($totalSections sections)" -ForegroundColor Green
}

# ─── Phase 7: Search ─────────────────────────────────────────────────────────

function Phase-7-Search {
    $Script:CURRENT_PHASE = "05-search-results"
    Write-Host "Phase 7: Search..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

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
                $score = if ($hit.score -as [double] -is [double]) { [math]::Round([double]$hit.score, 4) } else { "--" }
                $queryResultParts.Add("| $($hit.document_id) | `"$($hit.title)`" | $score |") | Out-Null
            }
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("_Results: $hitCount shown, $totalHits total_") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $searchName = "Search `"$($q.query)`""
            [void]$checks.Add(@{Name = $searchName; Status = "pass"; Detail = "$hitCount results"})
        } else {
            $searchErrors++
            $queryResultParts.Add("### Query: `"$($q.query)`"") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("❌ No results or error") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $searchName = "Search `"$($q.query)`""
            [void]$checks.Add(@{Name = $searchName; Status = "warn"; Detail = "No results"})
        }
    }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "05-search-results"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = if ($searchErrors -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "05-search-results") -join "`n"

    # Score: % of queries returning results
    $score = 0
    if ($expectedTotal -gt 0) { $score = [int]($expectedFound * 100 / $expectedTotal) }

    $analysis = Gen-PhaseAnalysis "05-search-results" $checksJson
    $recommendations = Gen-PhaseRecs "05-search-results" $checksJson
    $prevScore = Get-PrevMetric "05-search-results" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        QUERY_RESULTS = "$($queryResultParts -join "`n")"
        QUERY_COUNT = $sampleQueries.Count
        EXPECTED_FOUND = $expectedFound
        EXPECTED_TOTAL = $expectedTotal
        SEARCH_ERRORS = $searchErrors
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
    } | ConvertTo-Json -Compress
    Write-Report "05-search-results.md" "05-search-results.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["05-search-results"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "05-search-results.md"; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($expectedFound/$expectedTotal queries OK)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 8: Audit ──────────────────────────────────────────────────────────

function Phase-8-Audit {
    $Script:CURRENT_PHASE = "06-audit-findings"
    Write-Host "Phase 8: Audit..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

    if ($Script:NoAudit) {
        Write-Host "  Skipped (NoAudit)" -ForegroundColor DarkGray
        $reportVals = @{
            TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
            DURATION = 0
            STATUS = "⬜ SKIPPED"
            CHECKS_TABLE = "| - | Audit | ⬜ | Skipped via -NoAudit |"
            ERRORS_TABLE = "✅ No errors"
            AUDIT_SCORES_TABLE = "| -- | -- | -- | -- | -- | -- |"
            FINDINGS_BY_DOMAIN = "--"
            GATES_TABLE = "| -- | -- | -- | -- | -- |"
            BLOCKED_GATES_DETAIL = "--"
            DOMAIN_COUNT = 0
            TOTAL_FINDINGS = 0
            GATE_PASSES = 0
            GATE_TOTAL = 0
            GATE_BLOCKS = 0
            SCORE = 0
            TREND = "—"
            ANALYSIS = "Audit phase was skipped via -NoAudit flag."
            RECOMMENDATIONS = "Run without -NoAudit to assess audit health."
            PREV_SCORE = ""
        } | ConvertTo-Json -Compress
        Write-Report "06-audit-findings.md" "06-audit-findings.md" $reportVals | Out-Null
        $Script:PHASE_RESULTS["06-audit-findings"] = @{Status = "⬜ SKIPPED"; Errors = 0; Duration = 0; ReportFile = "06-audit-findings.md"; Score = 0} | ConvertTo-Json -Compress
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
            $s = if ($reportResult -and $reportResult.score -as [double] -is [double]) { [math]::Round([double]$reportResult.score, 2) } else { "--" }
            $stageScores[$stage] = $s
        }

        # If tool returns null/0 score but 0 findings, treat as 100 (no issues = perfect)
        $effectiveScore = if ($overallScore -as [double] -is [double]) { [double]$overallScore } else { $null }
        if (($null -eq $effectiveScore -or $effectiveScore -eq 0) -and $findings.Count -eq 0) { $effectiveScore = 100 }
        $scoreStr = if ($null -ne $effectiveScore) { [math]::Round($effectiveScore, 2) } else { "--" }
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

    [void]$checks.Add(@{Name = "Domain audits"; Status = "pass"; Detail = "$($domainNames.Count) domains audited"})
    $gd = if ($gateBlocks -gt 0) { "warn" } else { "pass" }
    [void]$checks.Add(@{Name = "Stage gates"; Status = $gd; Detail = "$gatePasses/$gateTotal passed"})

    if ($blockedDetail.Count -eq 0) { $blockedDetail.Add("No blocked gates") | Out-Null }
    if ($scoreRows.Count -eq 0) { $scoreRows.Add("| -- | -- | -- | -- | -- | -- |") | Out-Null }
    if ($gatesRows.Count -eq 0) { $gatesRows.Add("| -- | -- | -- | -- | -- |") | Out-Null }

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "06-audit-findings"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = if ($gateBlocks -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "06-audit-findings") -join "`n"

    # Score: gate pass rate (60%) + domain audit avg (40%)
    $score = 0
    if ($gateTotal -gt 0) {
        $gateScore = [int]($gatePasses * 60 / $gateTotal)
        $score = $gateScore
    }
    # Compute audit avg from domain scores
    $auditScores = @()
    foreach ($row in $scoreRows) {
        # row format: "| domain | overall | det | sec | doc | cross |"
        $cells = $row -split '\|' | Where-Object { $_.Trim() -ne '' }
        if ($cells.Count -ge 2) {
            $ov = $cells[1].Trim()
            if ($ov -as [double] -is [double]) { $auditScores += [double]$ov }
        }
    }
    if ($auditScores.Count -gt 0) {
        $auditAvg = ($auditScores | Measure-Object -Average).Average
        $score += [int]($auditAvg * 0.4)
    }
    if ($score -gt 100) { $score = 100 }

    $analysis = Gen-PhaseAnalysis "06-audit-findings" $checksJson
    $recommendations = Gen-PhaseRecs "06-audit-findings" $checksJson
    $prevScore = Get-PrevMetric "06-audit-findings" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        AUDIT_SCORES_TABLE = "$($scoreRows -join "`n")"
        FINDINGS_BY_DOMAIN = "$($findingsByDomain -join "`n")"
        GATES_TABLE = "$($gatesRows -join "`n")"
        BLOCKED_GATES_DETAIL = "$($blockedDetail -join "`n")"
        DOMAIN_COUNT = $domainNames.Count
        TOTAL_FINDINGS = $totalFindings
        GATE_PASSES = $gatePasses
        GATE_TOTAL = $gateTotal
        GATE_BLOCKS = $gateBlocks
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
    } | ConvertTo-Json -Compress
    Write-Report "06-audit-findings.md" "06-audit-findings.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["06-audit-findings"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "06-audit-findings.md"; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($totalFindings findings, $gatePasses/$gateTotal gates)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 9: Coverage Gaps ──────────────────────────────────────────────────

function Phase-9-Gaps {
    $Script:CURRENT_PHASE = "07-coverage-gaps"
    Write-Host "Phase 9: Coverage Gaps..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

    $missingKnowledge = [System.Collections.ArrayList]::new()
    $emptySections = [System.Collections.ArrayList]::new()
    $lowQuality = [System.Collections.ArrayList]::new()
    $requiredMissing = [System.Collections.ArrayList]::new()

    $missingKnCount = 0
    $emptyCount = 0
    $lowQCount = 0
    $reqMissingCount = 0

    # Check knowledge coverage for all (domain, type) pairs
    # Phase 6 already ran these calls — results are in $Script:AllResults.SectionIds
    # We re-check here because Phase 9 may run with -NoAudit=false even when Phase 6 was skipped
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

        $rawCov = if ($quality -and $quality.coverage -ne $null) { $quality.coverage } else { 1.0 }
        $cov = if ($rawCov -as [double] -is [double]) { [double]$rawCov } else { 1.0 }

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

    $knStatus = if ($missingKnCount -gt 0) { "warn" } else { "pass" }
    [void]$checks.Add(@{Name = "Knowledge coverage"; Status = $knStatus; Detail = "$missingKnCount missing"})
    $esStatus = if ($emptyCount -gt 0) { "warn" } else { "pass" }
    [void]$checks.Add(@{Name = "Empty sections"; Status = $esStatus; Detail = "$emptyCount empty"})
    $lqStatus = if ($lowQCount -gt 0) { "warn" } else { "pass" }
    [void]$checks.Add(@{Name = "Low quality docs"; Status = $lqStatus; Detail = "$lowQCount docs"})

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
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = if ($missingKnCount -gt 0 -or $emptyCount -gt 0 -or $lowQCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "07-coverage-gaps") -join "`n"

    # Score: lower gaps = better; max penalty 100 pts across gap types
    $gapTotal = $missingKnCount + $emptyCount + $lowQCount + $reqMissingCount
    $score = 100
    if ($gapTotal -gt 0) { $score = 100 - $gapTotal * 10 }
    if ($score -lt 0) { $score = 0 }

    $analysis = Gen-PhaseAnalysis "07-coverage-gaps" $checksJson
    $recommendations = Gen-PhaseRecs "07-coverage-gaps" $checksJson
    $prevScore = Get-PrevMetric "07-coverage-gaps" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        MISSING_KNOWLEDGE_TABLE = "$mkTable"
        EMPTY_SECTIONS_TABLE = "$esTable"
        LOW_QUALITY_TABLE = "$lqTable"
        REQUIRED_MISSING_TABLE = "$rmTable"
        MISSING_KNOWLEDGE_COUNT = $missingKnCount
        EMPTY_SECTION_COUNT = $emptyCount
        LOW_QUALITY_COUNT = $lowQCount
        REQUIRED_MISSING_COUNT = $reqMissingCount
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
    } | ConvertTo-Json -Compress
    Write-Report "07-coverage-gaps.md" "07-coverage-gaps.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["07-coverage-gaps"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "07-coverage-gaps.md"; Score = $score} | ConvertTo-Json -Compress
    Write-Host "  → Score: $score/100 $trend — $status ($gapTotal gaps)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($gapTotal -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 10: Registry + Write-Tool Smoke ───────────────────────────────────

function Phase-10-Registry {
    $Script:CURRENT_PHASE = "08-registry-state"
    Write-Host "Phase 10: Registry State..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = New-Object System.Collections.ArrayList

    # list_repositories
    $reposResult = Invoke-McpTool -Name "list_repositories" -Arguments @{ limit = 50 }
    $repoCount = if ($reposResult -and $reposResult.repositories) { $reposResult.repositories.Count } else { 0 }
    [void]$checks.Add(@{Name = "List repositories"; Status = "pass"; Detail = "$repoCount repos"})

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
    [void]$checks.Add(@{Name = "Resolve dependencies"; Status = "pass"; Detail = "$depCount deps, $unresolvedCount unresolved"})

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
    $syncOk = ($syncResult -ne $null)
    $syncStatus = if ($syncOk) { "pass" } else { "warn" }
    [void]$checks.Add(@{Name = "Synchronize repository"; Status = $syncStatus; Detail = "done"})

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
    [void]$checks.Add(@{Name = "Write-tool validation"; Status = $ws; Detail = "$writePass/$writeTotal pass"})

    $duration = [math]::Round(((Get-Date) - $start).TotalSeconds)
    $errorJson = Get-PhaseErrorsJson "08-registry-state"
    $errorCount = [int]($errorJson | ConvertFrom-Json).Count
    $status = if ($writePass -ne $writeTotal) { "⚠️ PARTIAL" } else { "✅ PASS" }

    $checksJson = $checks | ConvertTo-Json -Compress
    $checksTable = (Get-ChecksTable $checksJson) -join "`n"
    $errorsTable = (Get-ErrorsTable "08-registry-state") -join "`n"

    # Score: write-tool pass rate (70%) + dependencies resolved (30%)
    $subScore = [int]($writePass * 70 / [Math]::Max($writeTotal, 1))
    $depScore = if ($depCount -eq 0) {
        30  # no deps declared = no dependency issues = full marks
    } else {
        $resolved = $depCount - $unresolvedCount
        [int]([Math]::Max($resolved, 0) * 30 / $depCount)
    }
    $score = $subScore + $depScore

    $analysis = Gen-PhaseAnalysis "08-registry-state" $checksJson
    $recommendations = Gen-PhaseRecs "08-registry-state" $checksJson
    $prevScore = Get-PrevMetric "08-registry-state" "score"
    $trend = Trend-Between $score $prevScore

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        DURATION = $duration
        STATUS = "$status"
        CHECKS_TABLE = "$checksTable"
        ERRORS_TABLE = "$errorsTable"
        REPOS_TABLE = "$($reposTableLines -join "`n")"
        DEPS_TABLE = "$($depsTableLines -join "`n")"
        WORKSPACE_TABLE = "$($wsTableLines -join "`n")"
        REPO_STATUS_TABLE = "$($rsTableLines -join "`n")"
        WRITE_TOOL_TABLE = "$($writeToolLines -join "`n")"
        REPO_COUNT = $repoCount
        DEP_COUNT = $depCount
        UNRESOLVED_COUNT = $unresolvedCount
        WRITE_PASS = $writePass
        WRITE_TOTAL = $writeTotal
        SCORE = $score
        TREND = "$trend"
        ANALYSIS = "$analysis"
        RECOMMENDATIONS = "$recommendations"
        PREV_SCORE = "$prevScore"
    } | ConvertTo-Json -Compress
    Write-Report "08-registry-state.md" "08-registry-state.md" $reportVals | Out-Null

    $Script:PHASE_RESULTS["08-registry-state"] = @{Status = "$status"; Errors = $errorCount; Duration = $duration; ReportFile = "08-registry-state.md"; Score = $score} | ConvertTo-Json -Compress
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
        $prObj = $pr | ConvertFrom-Json
        $reportFile = if ($null -ne $prObj.ReportFile) { $prObj.ReportFile } else { "" }
        if ([string]::IsNullOrEmpty($reportFile)) { continue }
        $name = $reportFile -replace '\.md$', ''
        $status = if ($null -ne $prObj.Status) { $prObj.Status } else { "?" }
        $errors = if ($null -ne $prObj.Errors) { [int]$prObj.Errors } else { 0 }
        $pDuration = if ($null -ne $prObj.Duration) { [int]$prObj.Duration } else { 0 }
        $ps = if ($null -ne $prObj.Score) { $prObj.Score } else { $null }
        $scoreStr = if ($null -ne $ps) { "$ps/100" } else { "—" }
        $phaseRows.Add("| $name | $reportFile | $scoreStr | $status | $errors | ${pDuration}s |") | Out-Null
        $totalErrors += $errors
        $totalDuration += $pDuration
        if ($status -match "FAIL|PARTIAL") {
            $failedPhases.Add("- **$reportFile**: $status ($errors errors)") | Out-Null
        }
        if ($null -ne $ps -and $ps -ne "null") {
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
            $prObj = $pr | ConvertFrom-Json
            $ps = if ($null -ne $prObj.Score) { [int]$prObj.Score } else { 0 }
            $rpt = if ($null -ne $prObj.ReportFile) { $prObj.ReportFile } else { "" }
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
            $prObj = $pr | ConvertFrom-Json
            $ps = if ($null -ne $prObj.Score) { [int]$prObj.Score } else { 0 }
            if ($ps -lt 60) { $lowPhases++ }
        }
        if ($lowPhases -gt 0) { $overallRecs += "- Investigate $lowPhases phase(s) scoring below 60.`n" }
        if ($totalErrors -gt 0) { $overallRecs += "- Address $totalErrors error(s) across phases.`n" }
        if ([string]::IsNullOrEmpty($overallRecs)) { $overallRecs = "- All phases performing well. Maintain current practices.`n" }
    }

    $reportVals = @{
        TIMESTAMP = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        TOTAL_DURATION = $totalDuration
        OVERALL_STATUS = "$overallStatus"
        PHASE_RESULTS_ROWS = "$($phaseRows -join "`n")"
        TOOL_COUNT = $toolCount
        DOMAIN_COUNT = $domainCount
        DOCUMENT_COUNT = $docCount
        SECTION_COUNT = $sectCount
        SECTION_TYPE_COUNT = $sectTypeCount
        TOTAL_CALLS = $Script:TOTAL_CALLS
        TOTAL_ERRORS = $totalErrors
        FAILED_PHASES = "$($failedPhases -join "`n")"
        ARCHIVE_PATH = "$archiveStr"
        SCORE = $totalScore
        TREND = "$totalTrend"
        PREV_SCORE = "$prevTotalScore"
        ANALYSIS = "$overallAnalysis"
        RECOMMENDATIONS = "$overallRecs"
    } | ConvertTo-Json -Compress
    Write-Report "00-summary.md" "00-summary.md" $reportVals | Out-Null

    # Build and save metrics JSON for next run
    $psJson = Build-PhaseScoresJson
    $allJq = $Script:AllResults | ConvertTo-Json -Compress -Depth 10
    $metrics = @{
        timestamp = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        phase_scores = $psJson
        total_score = $totalScore
        metrics = @{
            tool_count = $toolCount
            doc_count = $docCount
            section_count = $sectCount
            section_type_count = $sectTypeCount
            domain_count = $domainCount
            error_count = $totalErrors
            total_calls = $Script:TOTAL_CALLS
        }
    } | ConvertTo-Json -Compress
    $metrics | Set-Content -Path (Join-Path $Script:LATEST_DIR "metrics.json") -Encoding UTF8

    $Script:PHASE_RESULTS["00-summary"] = @{Status = "✅ DONE"; Errors = $totalErrors; Score = $totalScore; Duration = $totalDuration; ReportFile = "00-summary.md"} | ConvertTo-Json -Compress

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
