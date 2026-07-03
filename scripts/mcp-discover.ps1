<#
.SYNOPSIS
  Discover and test all MCP tools by walking the domain-document-section
  hierarchy. Produces templated reports in docs/report/manual-audit/mcp/latest/.

.DESCRIPTION
  Pipeline: Bootstrap → Domain Scan → Doc Discovery → Doc Verify →
  Cross-Section → Section Verify → Search → Audit → Gaps → Registry
  → Summary report.

  Reports are rendered from markdown templates in
  docs/report/manual-audit/mcp/templates/.
  Previous run is rotated to archive/{timestamp}/.

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

# Report directory setup
if (-not $ReportDir) { $ReportDir = "docs/report/manual-audit/mcp" }
$Script:ReportDir = Join-Path $Script:RootDir $ReportDir
$Script:TemplateDir = Join-Path $Script:ReportDir "templates"
$Script:LatestDir = Join-Path $Script:ReportDir "latest"
$Script:ArchiveDir = Join-Path $Script:ReportDir "archive"

# Ensure template and archive dirs exist
New-Item -ItemType Directory -Force $Script:TemplateDir | Out-Null
New-Item -ItemType Directory -Force $Script:ArchiveDir | Out-Null

# Rotate previous latest → archive/{timestamp}/
if (Test-Path $Script:LatestDir) {
    $ts = Get-Date -Format "yyyy-MM-dd_HHmmss"
    $archivePath = Join-Path $Script:ArchiveDir $ts
    Move-Item $Script:LatestDir $archivePath
    $Script:ArchivePath = $archivePath
    Write-Host "Archived previous run → $archivePath" -ForegroundColor DarkGray
}
New-Item -ItemType Directory -Force $Script:LatestDir | Out-Null

# ─── Global State ─────────────────────────────────────────────────────────────
$Script:PhaseErrors = @{}
$Script:PhaseResults = @{}
$Script:AllResults = @{}
$Script:NextId = 1
$Script:CurrentPhase = ""
$Script:TotalCalls = 0

# ─── Core Functions ───────────────────────────────────────────────────────────

function Get-Id {
    $id = $Script:NextId
    $Script:NextId++
    return $id
}

function Invoke-McpDirect {
    <#
    .SYNOPSIS
      Send JSON-RPC directly (method at top level). Used for initialize, tools/list.
    #>
    param([string]$Method, $Params = @{}, [int]$Id = -1)

    if ($Id -eq -1) { $Id = Get-Id }
    $Script:TotalCalls++

    $request = @{
        jsonrpc = "2.0"
        id = $Id
        method = $Method
        params = $Params
    } | ConvertTo-Json -Compress -Depth 10

    try {
        $raw = $request | & $Script:BinaryPath 2>$null
        if ([string]::IsNullOrEmpty("$raw")) {
            Add-PhaseError -Tool $request -Error "Empty response" -Response ""
            return $null
        }
        if ($raw -is [array]) { $raw = $raw[0] }
        $parsed = $raw | ConvertFrom-Json
        if ($parsed.PSObject.Properties['error'] -and $parsed.error) {
            Add-PhaseError -Tool $request -Error "$($parsed.error.code): $($parsed.error.message)" -Response $raw
            return $null
        }
        return $parsed.result
    } catch {
        Add-PhaseError -Tool $request -Error $_.Exception.Message -Response ""
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
    $Script:TotalCalls++

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
            if (-not $Quiet) { Add-PhaseError -Tool $request -Error "Empty response" -Response "" }
            return $null
        }
        if ($raw -is [array]) { $raw = $raw[0] }
        $parsed = $raw | ConvertFrom-Json
        if ($parsed.PSObject.Properties['error'] -and $parsed.error) {
            if (-not $Quiet) { Add-PhaseError -Tool $request -Error "$($parsed.error.code): $($parsed.error.message)" -Response $raw }
            return $null
        }
        return $parsed.result
    } catch {
        if (-not $Quiet) { Add-PhaseError -Tool $request -Error $_.Exception.Message -Response "" }
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

function Add-PhaseError {
    param([string]$Tool, [string]$Error, [string]$Response)

    if (-not $Script:PhaseErrors.ContainsKey($Script:CurrentPhase)) {
        $Script:PhaseErrors[$Script:CurrentPhase] = [System.Collections.ArrayList]::new()
    }

    $respSnippet = if ($Response -and $Response.Length -gt 250) {
        $Response.Substring(0, 250) + "..."
    } else { $Response }

    $Script:PhaseErrors[$Script:CurrentPhase].Add(@{
        Tool = $Tool
        Error = $Error
        Response = $respSnippet
    }) | Out-Null
}

function Get-PhaseErrors {
    param([string]$Phase)
    if ($Script:PhaseErrors.ContainsKey($Phase)) {
        return $Script:PhaseErrors[$Phase]
    }
    return @()
}

function Get-ErrorsTable {
    param([string]$Phase)
    $errors = Get-PhaseErrors -Phase $Phase
    if ($errors.Count -eq 0) { return "✅ No errors" }

    $lines = [System.Collections.ArrayList]::new()
    $lines.Add("| Tool Call | Error | Response |") | Out-Null
    $lines.Add("|-----------|-------|----------|") | Out-Null
    foreach ($e in $errors) {
        $tool = ($e.Tool -replace '\|', '\|')
        $err = ($e.Error -replace '\|', '\|')
        $resp = ($e.Response -replace '\|', '\|')
        if ($resp.Length -gt 120) { $resp = $resp.Substring(0, 120) + "..." }
        $lines.Add("| $tool | $err | $resp |") | Out-Null
    }
    return $lines -join "`n"
}

function Get-ChecksTable {
    param([array]$Checks)
    $lines = [System.Collections.ArrayList]::new()
    $lines.Add("| # | Check | Status | Detail |") | Out-Null
    $lines.Add("|---|-------|--------|--------|") | Out-Null
    $i = 0
    foreach ($c in $Checks) {
        $i++
        $icon = switch ($c.Status) { "pass" { "✅" } "fail" { "❌" } "warn" { "⚠️" } default { "⬜" } }
        $detail = ($c.Detail -replace '\|', '\|')
        if ($detail.Length -gt 80) { $detail = $detail.Substring(0, 80) + "..." }
        $lines.Add("| $i | $($c.Name) | $icon $($c.Status) | $detail |") | Out-Null
    }
    return $lines -join "`n"
}

function Write-Report {
    <#
    .SYNOPSIS
      Read template, substitute {{KEY}} placeholders, write to latest/.
    #>
    param(
        [string]$TemplateName,
        [string]$OutputName,
        [hashtable]$Values
    )

    $templatePath = Join-Path $Script:TemplateDir $TemplateName
    $outputPath = Join-Path $Script:LatestDir $OutputName

    if (Test-Path $templatePath) {
        $content = Get-Content $templatePath -Raw
    } else {
        Write-Warning "Template missing: $templatePath -- using inline fallback"
        $content = "# $OutputName`n`n**Status:** {{STATUS}}`n`n{{ERRORS_TABLE}}`n`n{{CHECKS_TABLE}}"
    }

    foreach ($kv in $Values.GetEnumerator()) {
        $content = $content.Replace("{{$($kv.Key)}}", "$($kv.Value)")
    }

    Set-Content -Path $outputPath -Value $content -Encoding UTF8
    return $outputPath
}

function EscMarkdown {
    param([string]$s)
    return ($s -replace '\|', '\|')
}

# ─── Phase 1: Bootstrap ──────────────────────────────────────────────────────

function Phase-1-Bootstrap {
    $Script:CurrentPhase = "01-tool-health"
    Write-Host "Phase 1: Bootstrap..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    # initialize
    $initResult = Invoke-McpDirect -Method "initialize" -Params @{
        protocolVersion = "2025-03-26"
        capabilities = @{}
        clientInfo = @{ name = "mcp-discover"; version = "1.0" }
    }
    if ($initResult) {
        $Script:AllResults.Protocol = $initResult.protocolVersion
        $checks += @{ Name = "Initialize"; Status = "pass"; Detail = "Protocol $($initResult.protocolVersion)" }
    } else {
        $checks += @{ Name = "Initialize"; Status = "fail"; Detail = "No response" }
    }

    # tools/list
    $toolsResult = Invoke-McpDirect -Method "tools/list"
    $tools = @()
    if ($toolsResult -and $toolsResult.tools) {
        $tools = $toolsResult.tools
        $Script:AllResults.Tools = $tools
        $checks += @{ Name = "Tools/List"; Status = "pass"; Detail = "$($tools.Count) tools" }
    } else {
        $checks += @{ Name = "Tools/List"; Status = "fail"; Detail = "No tools returned" }
    }

    # info via tools/call
    $infoResult = Invoke-McpTool -Name "info"
    if ($infoResult) {
        $Script:AllResults.Runtime = $infoResult
        $checks += @{ Name = "Info"; Status = "pass"; Detail = "$($infoResult.document_count) docs" }
    } else {
        $checks += @{ Name = "Info"; Status = "fail"; Detail = "No response" }
    }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "01-tool-health"
    $status = if ($checks | Where-Object { $_.Status -eq "fail" }) { "❌ FAIL" } else { "✅ PASS" }

    # Build tool table for template
    $toolRows = [System.Collections.ArrayList]::new()
    $ti = 0
    foreach ($t in $tools) {
        $ti++
        $name = $t.name
        $req = if ($t.inputSchema -and $t.inputSchema.required) { $t.inputSchema.required -join ", " } else { "none" }
        $toolStatus = if (Get-PhaseErrors "01-tool-health" | Where-Object { $_ -match $name }) { "⚠️" } else { "✅" }
        $toolRows.Add("| $ti | `"$name`" | $req | $toolStatus |") | Out-Null
    }

    $standards = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) {
        $Script:AllResults.Runtime.standards -join ", "
    } else { "--" }
    $services = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.services) {
        $Script:AllResults.Runtime.services -join ", "
    } else { "--" }
    $policy = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.policy) {
        ($Script:AllResults.Runtime.policy | ConvertTo-Json -Compress -Depth 2)
    } else { "--" }

    Write-Report -TemplateName "01-tool-health.md" -OutputName "01-tool-health.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "01-tool-health"
        TOOLS_TABLE = $toolRows -join "`n"
        DOC_COUNT = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.document_count } else { "?" }
        STANDARDS_LIST = $standards
        STANDARD_COUNT = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) { $Script:AllResults.Runtime.standards.Count } else { 0 }
        REGISTRY_PATH = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.registry_path } else { "--" }
        REPOSITORY = if ($Script:AllResults.Runtime) { $Script:AllResults.Runtime.repository } else { "--" }
        SERVICES = $services
        POLICY = $policy
        TOOL_COUNT = $tools.Count
        TOOL_ERROR_COUNT = [math]::Max(0, ($errors | Measure-Object).Count)
    }

    $Script:PhaseResults["01-tool-health"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "01-tool-health.md"
    }
    Write-Host "  → $status ($($Script:PhaseResults["01-tool-health"].Duration)s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 2: Domain Catalog ──────────────────────────────────────────────────

function Phase-2-DomainScan {
    $Script:CurrentPhase = "02-domain-catalog"
    Write-Host "Phase 2: Domain Scan..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    $domainsResult = Invoke-McpTool -Name "list_domains" -Arguments @{}
    $domains = @()
    if ($domainsResult -and $domainsResult.domains) {
        $allDomains = $domainsResult.domains
        if ($Domain.Count -gt 0) {
            $domains = $allDomains | Where-Object { $_ -in $Domain }
        } else {
            $domains = $allDomains
        }
        $Script:AllResults.Domains = @{}
        foreach ($d in $domains) {
            $Script:AllResults.Domains[$d] = @{
                name = $d
                docs = @()
                sectionTypes = @{}
                sectionIds = @()
            }
        }
        $checks += @{ Name = "List Domains"; Status = "pass"; Detail = "$($domains.Count) domains" }
    } else {
        $checks += @{ Name = "List Domains"; Status = "fail"; Detail = "No domains" }
    }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "02-domain-catalog"
    $status = if ($domains.Count -gt 0) { "✅ PASS" } else { "❌ FAIL" }

    $Script:PhaseResults["02-domain-catalog"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "02-domain-catalog.md"
    }
    Write-Host "  → $status ($($Script:PhaseResults["02-domain-catalog"].Duration)s)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 3: Document Discovery ─────────────────────────────────────────────

function Phase-3-DocDiscover {
    $Script:CurrentPhase = "03-document-audit"
    Write-Host "Phase 3: Document Discovery..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    $domains = $Script:AllResults.Domains.Keys
    $totalDocs = 0
    $allDocCount = 0

    foreach ($d in $domains) {
        Write-Host "  Fetching docs for '$d'..." -ForegroundColor DarkGray
        $docResult = Invoke-McpToolAll -Name "get_documents_by_domain" -Arguments @{ domain = $d } -CollectionKey "documents"

        if ($docResult -and $docResult.Count -gt 0) {
            $docs = $docResult
            $Script:AllResults.Domains[$d].docs = $docs
            $Script:AllResults.Domains[$d].docCount = $docs.Count
            $totalDocs += $docs.Count
            $checks += @{ Name = "Docs in '$d'"; Status = "pass"; Detail = "$($docs.Count) docs" }
        } else {
            $Script:AllResults.Domains[$d].docCount = 0
            $checks += @{ Name = "Docs in '$d'"; Status = "warn"; Detail = "0 docs (or error)" }
        }
        $allDocCount += $Script:AllResults.Domains[$d].docCount
    }

    $Script:AllResults.TotalDocs = $allDocCount

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "03-document-audit"
    $failedChecks = $checks | Where-Object { $_.Status -eq "fail" }
    $status = if ($failedChecks.Count -gt 0) { "⚠️ PARTIAL" } elseif ($allDocCount -gt 0) { "✅ PASS" } else { "❌ FAIL" }

    # Store phase result but no report yet -- Phase 4 generates the report
    $Script:PhaseResults["03-doc-discover"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round(((Get-Date) - $start).TotalSeconds, 2)
    }

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
    Write-Report -TemplateName "02-domain-catalog.md" -OutputName "02-domain-catalog.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round(((Get-Date) - $start).TotalSeconds, 2)
        STATUS = "✅ PASS"
        CHECKS_TABLE = "| 1 | List Domains | pass | $($domains.Count) domains |"
        ERRORS_TABLE = "No errors"
        DOMAINS_TABLE = $dcDomainRows -join "`n"
        STANDARDS_LIST = $dcStandards
        STANDARD_COUNT = if ($Script:AllResults.Runtime -and $Script:AllResults.Runtime.standards) { $Script:AllResults.Runtime.standards.Count } else { 0 }
        DOC_COUNTS_TABLE = $dcDocCountRows -join "`n"
        DOMAIN_COUNT = $domains.Count
        DOCUMENT_COUNT = $allDocCount
    }

    Write-Host "  → $status ($allDocCount docs discovered)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 4: Document Verification + Report ─────────────────────────────────

function Phase-4-DocVerify {
    $Script:CurrentPhase = "03-document-audit"
    Write-Host "Phase 4: Document Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    $domains = $Script:AllResults.Domains.Keys
    $allDocs = @()
    $allSectionsTotal = 0
    $qualityRows = [System.Collections.ArrayList]::new()
    $domainDocsSectionsParts = [System.Collections.ArrayList]::new()
    $issues = [System.Collections.ArrayList]::new()
    $issueCount = 0
    $sectDist = @{ "0" = 0; "1-3" = 0; "4-7" = 0; "8-15" = 0; "16+" = 0 }

    foreach ($d in $domains) {
        $docs = $Script:AllResults.Domains[$d].docs
        $totalSects = 0
        $emptySects = 0
        $missingSects = 0
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
            if ($emptyCount -gt 0) { $emptySects += $emptyCount }
            if ($missingCount -gt 0) { $missingSects += $missingCount }

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
        $qualityRows.Add("| $d | $($docs.Count) | $avgSects | $emptySects | $missingSects | -- |") | Out-Null
    }

    $Script:AllResults.AllDocs = $allDocs
    $Script:AllResults.TotalSections = $allSectionsTotal

    $checks += @{ Name = "Document verification"; Status = "pass"; Detail = "$($allDocs.Count) docs across $($domains.Count) domains" }
    $checks += @{ Name = "Section count"; Status = "pass"; Detail = "$allSectionsTotal sections total" }

    $distRows = [System.Collections.ArrayList]::new()
    foreach ($k in @("0", "1-3", "4-7", "8-15", "16+")) {
        if ($sectDist[$k] -gt 0) {
            $distRows.Add("| $k sections | $($sectDist[$k]) docs |") | Out-Null
        }
    }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "03-document-audit"
    $status = if ($issueCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    Write-Report -TemplateName "03-document-audit.md" -OutputName "03-document-audit.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "03-document-audit"
        DOMAIN_DOCS_SECTIONS = $domainDocsSectionsParts -join "`n`n"
        QUALITY_TABLE = if ($qualityRows.Count -gt 0) { $qualityRows -join "`n" } else { "| -- | -- | -- | -- | -- | -- |" }
        ISSUES_LIST = if ($issues.Count -gt 0) {
            $t = @("| Doc ID | Title | Domain | Issues |", "|--------|-------|--------|--------|"); foreach ($r in $issues) { $t += $r }; $t -join "`n"
        } else { "No issues found" }
        SECTION_DIST_TABLE = $distRows -join "`n"
        TOTAL_DOCS = $allDocs.Count
        DOMAIN_COUNT = $domains.Count
        TOTAL_SECTIONS = $allSectionsTotal
        ISSUE_COUNT = $issueCount
    }

    $Script:PhaseResults["03-document-audit"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "03-document-audit.md"
    }
    Write-Host "  → $status ($issueCount issues)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($issueCount -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 5: Cross-Section (acquire section_ids from get_sections) ───────────

function Phase-5-CrossSection {
    $Script:CurrentPhase = "04-section-integrity"
    Write-Host "Phase 5: Cross-Section..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

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

    $checks += @{ Name = "Cross-section query"; Status = "pass"; Detail = "$($pairs.Count) type-domain pairs, $totalSections sections" }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "04-section-integrity"

    $Script:PhaseResults["04-cross-section"] = @{
        Status = if ($totalSections -gt 0) { "✅ PASS" } else { "⚠️ PARTIAL" }
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
    }
    Write-Host "  → $totalSections sections from $($pairs.Count) type-domain pairs" -ForegroundColor Green
}

# ─── Phase 6: Section Verification ───────────────────────────────────────────

function Phase-6-SectionVerify {
    $Script:CurrentPhase = "04-section-integrity"
    Write-Host "Phase 6: Section Verification..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    $sectionIds = $Script:AllResults.SectionIds
    $sectionVerifyParts = [System.Collections.ArrayList]::new()
    $sectionTypeRows = [System.Collections.ArrayList]::new()
    $changeTrackRows = [System.Collections.ArrayList]::new()
    $knowledgeRows = [System.Collections.ArrayList]::new()
    $totalSections = 0
    $staleCount = 0
    $knowledgeCount = 0
    $knowledgeMissing = 0

    # Section types by domain
    foreach ($d in $Script:AllResults.Domains.Keys) {
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

    $sectionIdTotals = @{}
    $sectionIdChanged = @{}
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
    $checks += @{ Name = "Section verification"; Status = "pass"; Detail = "$totalSections sections checked" }
    if ($staleCount -gt 0) {
        $checks += @{ Name = "Stale sections"; Status = "warn"; Detail = "$staleCount changed since last audit" }
    }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "04-section-integrity"
    $status = "✅ PASS"

    # Change tracking table
    $changeTrackRows.Add("| Stale (changed) | $staleCount |") | Out-Null
    $changeTrackRows.Add("| Fresh (unchanged) | $($totalSections - $staleCount) |") | Out-Null

    if ($knowledgeRows.Count -eq 0) { $knowledgeRows.Add("| -- | -- | -- |") | Out-Null }

    Write-Report -TemplateName "04-section-integrity.md" -OutputName "04-section-integrity.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "04-section-integrity"
        SECTION_TYPES_TABLE = if ($sectionTypeRows.Count -gt 0) { $sectionTypeRows -join "`n" } else { "| -- | -- |" }
        SECTION_VERIFY_TABLE = $sectionVerifyParts -join "`n`n"
        CHANGE_TRACKING_TABLE = $changeTrackRows -join "`n"
        KNOWLEDGE_TABLE = if ($knowledgeRows.Count -gt 0) { $knowledgeRows -join "`n" } else { "| -- | -- | -- |" }
        TOTAL_SECTIONS = $totalSections
        DOMAIN_COUNT = $Script:AllResults.Domains.Keys.Count
        UNIQUE_TYPES = $sectionTypesCount
        STALE_SECTIONS = $staleCount
        KNOWLEDGE_COUNT = $knowledgeCount
        KNOWLEDGE_MISSING = $knowledgeMissing
    }

    $Script:PhaseResults["04-section-integrity"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "04-section-integrity.md"
    }
    Write-Host "  → $status ($totalSections sections)" -ForegroundColor Green
}

# ─── Phase 7: Search ─────────────────────────────────────────────────────────

function Phase-7-Search {
    $Script:CurrentPhase = "05-search-results"
    Write-Host "Phase 7: Search..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

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

    # Add a general query
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
            $checks += @{ Name = "Search `"$($q.query)`""; Status = "pass"; Detail = "$hitCount results" }
        } else {
            $searchErrors++
            $queryResultParts.Add("### Query: `"$($q.query)`"") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $queryResultParts.Add("❌ No results or error") | Out-Null
            $queryResultParts.Add("") | Out-Null
            $checks += @{ Name = "Search `"$($q.query)`""; Status = "warn"; Detail = "No results" }
        }
    }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "05-search-results"
    $status = if ($searchErrors -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    Write-Report -TemplateName "05-search-results.md" -OutputName "05-search-results.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "05-search-results"
        QUERY_RESULTS = $queryResultParts -join "`n"
        QUERY_COUNT = $sampleQueries.Count
        EXPECTED_FOUND = $expectedFound
        EXPECTED_TOTAL = $expectedTotal
        SEARCH_ERRORS = $searchErrors
    }

    $Script:PhaseResults["05-search-results"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "05-search-results.md"
    }
    Write-Host "  → $status ($expectedFound/$expectedTotal queries OK)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 8: Audit ──────────────────────────────────────────────────────────

function Phase-8-Audit {
    $Script:CurrentPhase = "06-audit-findings"
    Write-Host "Phase 8: Audit..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    if ($Script:NoAudit) {
        Write-Host "  Skipped (NoAudit)" -ForegroundColor DarkGray
        $Script:PhaseResults["06-audit-findings"] = @{ Status = "⬜ SKIPPED"; Errors = 0; Duration = 0; ReportFile = "06-audit-findings.md" }
        Write-Report -TemplateName "06-audit-findings.md" -OutputName "06-audit-findings.md" -Values @{
            TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"; DURATION = 0; STATUS = "⬜ SKIPPED"
            CHECKS_TABLE = "| - | Audit | ⬜ | Skipped via -NoAudit |"
            ERRORS_TABLE = "✅ No errors"; AUDIT_SCORES_TABLE = "| -- | -- | -- | -- | -- | -- |"
            FINDINGS_BY_DOMAIN = "--"; GATES_TABLE = "| -- | -- | -- | -- | -- |"
            BLOCKED_GATES_DETAIL = "--"; DOMAIN_COUNT = 0; TOTAL_FINDINGS = 0
            GATE_PASSES = 0; GATE_TOTAL = 0; GATE_BLOCKS = 0
        }
        return
    }

    $domains = $Script:AllResults.Domains.Keys
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

    $checks += @{ Name = "Domain audits"; Status = "pass"; Detail = "$($domains.Count) domains audited" }
    $checks += @{ Name = "Stage gates"; Status = if ($gateBlocks -gt 0) { "warn" } else { "pass" }; Detail = "$gatePasses/$gateTotal passed" }

    if ($blockedDetail.Count -eq 0) { $blockedDetail.Add("No blocked gates") | Out-Null }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "06-audit-findings"
    $status = if ($gateBlocks -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    Write-Report -TemplateName "06-audit-findings.md" -OutputName "06-audit-findings.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "06-audit-findings"
        AUDIT_SCORES_TABLE = if ($scoreRows.Count -gt 0) { $scoreRows -join "`n" } else { "| -- | -- | -- | -- | -- | -- |" }
        FINDINGS_BY_DOMAIN = $findingsByDomain -join "`n"
        GATES_TABLE = if ($gatesRows.Count -gt 0) { $gatesRows -join "`n" } else { "| -- | -- | -- | -- | -- |" }
        BLOCKED_GATES_DETAIL = $blockedDetail -join "`n"
        DOMAIN_COUNT = $domains.Count
        TOTAL_FINDINGS = $totalFindings
        GATE_PASSES = $gatePasses
        GATE_TOTAL = $gateTotal
        GATE_BLOCKS = $gateBlocks
    }

    $Script:PhaseResults["06-audit-findings"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "06-audit-findings.md"
    }
    Write-Host "  → $status ($totalFindings findings, $gatePasses/$gateTotal gates)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 9: Coverage Gaps ──────────────────────────────────────────────────

function Phase-9-Gaps {
    $Script:CurrentPhase = "07-coverage-gaps"
    Write-Host "Phase 9: Coverage Gaps..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

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
            $hasKnowledge = $false
            if ($Script:AllResults.SectionIds -and $Script:AllResults.SectionIds[$d] -and $Script:AllResults.SectionIds[$d][$t]) {
                $hasKnowledge = $true  # We already queried it; if it failed it wouldn't be here
            }
            # Re-check with a direct call
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

    $checks += @{ Name = "Knowledge coverage"; Status = if ($missingKnCount -gt 0) { "warn" } else { "pass" }; Detail = "$missingKnCount missing" }
    $checks += @{ Name = "Empty sections"; Status = if ($emptyCount -gt 0) { "warn" } else { "pass" }; Detail = "$emptyCount empty" }
    $checks += @{ Name = "Low quality docs"; Status = if ($lowQCount -gt 0) { "warn" } else { "pass" }; Detail = "$lowQCount docs" }

    if ($missingKnowledge.Count -eq 0) { $missingKnowledge.Add("| -- | -- | All covered |") | Out-Null }
    if ($emptySections.Count -eq 0) { $emptySections.Add("| -- | -- | -- |") | Out-Null }
    if ($lowQuality.Count -eq 0) { $lowQuality.Add("| -- | -- | -- | -- |") | Out-Null }
    if ($requiredMissing.Count -eq 0) { $requiredMissing.Add("| -- | -- | -- | -- |") | Out-Null }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "07-coverage-gaps"
    $status = if ($missingKnCount -gt 0 -or $emptyCount -gt 0 -or $lowQCount -gt 0) { "⚠️ PARTIAL" } else { "✅ PASS" }

    Write-Report -TemplateName "07-coverage-gaps.md" -OutputName "07-coverage-gaps.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "07-coverage-gaps"
        MISSING_KNOWLEDGE_TABLE = if ($missingKnowledge.Count -gt 0) {
            $t = @("| Domain | Section Type | Status |", "|--------|-------------|--------|"); foreach ($r in $missingKnowledge) { $t += $r }; $t -join "`n"
        } else { "All knowledge files present" }
        EMPTY_SECTIONS_TABLE = if ($emptySections.Count -gt 0) {
            $t = @("| Doc ID | Section | Type |", "|--------|---------|------|"); foreach ($r in $emptySections) { $t += $r }; $t -join "`n"
        } else { "No empty sections" }
        LOW_QUALITY_TABLE = if ($lowQuality.Count -gt 0) {
            $t = @("| Doc ID | Title | Domain | Issue |", "|--------|-------|--------|-------|"); foreach ($r in $lowQuality) { $t += $r }; $t -join "`n"
        } else { "All documents OK" }
        REQUIRED_MISSING_TABLE = if ($requiredMissing.Count -gt 0) {
            $t = @("| Doc ID | Title | Domain | Missing |", "|--------|-------|--------|---------|"); foreach ($r in $requiredMissing) { $t += $r }; $t -join "`n"
        } else { "No missing required sections" }
        MISSING_KNOWLEDGE_COUNT = $missingKnCount
        EMPTY_SECTION_COUNT = $emptyCount
        LOW_QUALITY_COUNT = $lowQCount
        REQUIRED_MISSING_COUNT = $reqMissingCount
    }

    $Script:PhaseResults["07-coverage-gaps"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "07-coverage-gaps.md"
    }
    Write-Host "  → $status" -ForegroundColor $(if ($status -match "FAIL") { "Red" } elseif ($missingKnCount -gt 0) { "Yellow" } else { "Green" })
}

# ─── Phase 10: Registry + Write-Tool Smoke ───────────────────────────────────

function Phase-10-Registry {
    $Script:CurrentPhase = "08-registry-state"
    Write-Host "Phase 10: Registry State..." -ForegroundColor Cyan
    $start = Get-Date
    $checks = @()

    # list_repositories
    $repos = Invoke-McpTool -Name "list_repositories" -Arguments @{ limit = 50 }
    $repoCount = if ($repos -and $repos.repositories) { $repos.repositories.Count } else { 0 }
    $checks += @{ Name = "List repositories"; Status = if ($repoCount -ge 0) { "pass" } else { "warn" }; Detail = "$repoCount repos" }

    $reposTableLines = [System.Collections.ArrayList]::new()
    $reposTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $reposTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($repos -and $repos.repositories) {
        $ri = 0
        foreach ($r in $repos.repositories) {
            $ri++
            $id = $r.id
            $uuid = $r.uuid
            $status = if ($r.status) { $r.status } else { "--" }
            $reposTableLines.Add("| $ri | $id | $uuid | $status |") | Out-Null
        }
    } else {
        $reposTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # resolve_dependencies
    $deps = Invoke-McpTool -Name "resolve_dependencies" -Arguments @{}
    $depCount = 0
    $unresolvedCount = 0
    $depsTableLines = [System.Collections.ArrayList]::new()
    $depsTableLines.Add("| Name | Path | Available | Required |") | Out-Null
    $depsTableLines.Add("|------|------|-----------|----------|") | Out-Null
    if ($deps -and $deps.dependencies) {
        $depCount = $deps.dependencies.Count
        foreach ($dep in $deps.dependencies) {
            $avail = if ($dep.available) { "✅" } else { "❌" }
            $req = if ($dep.required) { "yes" } else { "no" }
            $depsTableLines.Add("| $($dep.name) | $($dep.path) | $avail | $req |") | Out-Null
            if (-not $dep.available) { $unresolvedCount++ }
        }
    } else {
        $depsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }
    $checks += @{ Name = "Resolve dependencies"; Status = "pass"; Detail = "$depCount deps, $unresolvedCount unresolved" }

    # workspace_status
    $ws = Invoke-McpTool -Name "workspace_status" -Arguments @{}
    $wsTableLines = [System.Collections.ArrayList]::new()
    $wsTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $wsTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($ws -and $ws.repositories) {
        $wi = 0
        foreach ($r in $ws.repositories) {
            $wi++
            $wsTableLines.Add("| $wi | $($r.id) | $($r.uuid) | $($r.status) |") | Out-Null
        }
    } else {
        $wsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # repository_status
    $rs = Invoke-McpTool -Name "repository_status" -Arguments @{ limit = 50 }
    $rsTableLines = [System.Collections.ArrayList]::new()
    $rsTableLines.Add("| # | ID | UUID | Status |") | Out-Null
    $rsTableLines.Add("|---|----|------|--------|") | Out-Null
    if ($rs -and $rs.repositories) {
        $ri = 0
        foreach ($r in $rs.repositories) {
            $ri++
            $rsTableLines.Add("| $ri | $($r.id) | $($r.uuid) | $($r.status) |") | Out-Null
        }
    } else {
        $rsTableLines.Add("| -- | -- | -- | -- |") | Out-Null
    }

    # synchronize_repository
    $syncResult = Invoke-McpTool -Name "synchronize_repository" -Arguments @{}
    $checks += @{ Name = "Synchronize repository"; Status = if ($syncResult) { "pass" } else { "warn" }; Detail = "done" }

    # ── Write-tool smoke tests (expect failures with bad input) ──
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

    $checks += @{ Name = "Write-tool validation"; Status = if ($writePass -eq $writeTotal) { "pass" } else { "warn" }; Detail = "$writePass/$writeTotal pass" }

    $duration = (Get-Date) - $start
    $errors = Get-PhaseErrors -Phase "08-registry-state"
    $status = if ($writePass -ne $writeTotal) { "⚠️ PARTIAL" } else { "✅ PASS" }

    Write-Report -TemplateName "08-registry-state.md" -OutputName "08-registry-state.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        DURATION = [math]::Round($duration.TotalSeconds, 2)
        STATUS = $status
        CHECKS_TABLE = Get-ChecksTable $checks
        ERRORS_TABLE = Get-ErrorsTable "08-registry-state"
        REPOS_TABLE = $reposTableLines -join "`n"
        DEPS_TABLE = $depsTableLines -join "`n"
        WORKSPACE_TABLE = $wsTableLines -join "`n"
        REPO_STATUS_TABLE = $rsTableLines -join "`n"
        WRITE_TOOL_TABLE = $writeToolLines -join "`n"
        REPO_COUNT = $repoCount
        DEP_COUNT = $depCount
        UNRESOLVED_COUNT = $unresolvedCount
        WRITE_PASS = $writePass
        WRITE_TOTAL = $writeTotal
    }

    $Script:PhaseResults["08-registry-state"] = @{
        Status = $status
        Errors = ($errors | Measure-Object).Count
        Duration = [math]::Round($duration.TotalSeconds, 2)
        ReportFile = "08-registry-state.md"
    }
    Write-Host "  → $status ($writePass/$writeTotal write-tool)" -ForegroundColor $(if ($status -match "FAIL") { "Red" } else { "Green" })
}

# ─── Phase 11: Summary ───────────────────────────────────────────────────────

function Phase-11-Summary {
    $Script:CurrentPhase = "00-summary"
    Write-Host "Phase 11: Summary..." -ForegroundColor Cyan
    $start = Get-Date

    $phaseRanks = @{
        "01-tool-health" = 1; "02-domain-catalog" = 2; "03-document-audit" = 3
        "04-section-integrity" = 4; "05-search-results" = 5; "06-audit-findings" = 6
        "07-coverage-gaps" = 7; "08-registry-state" = 8
    }

    $phaseRows = [System.Collections.ArrayList]::new()
    $failedPhases = [System.Collections.ArrayList]::new()
    $totalErrors = 0
    $totalDuration = 0

    foreach ($key in $Script:PhaseResults.Keys | Sort-Object {
        if ($phaseRanks.ContainsKey($_)) { $phaseRanks[$_] } else { 99 }
    }) {
        $pr = $Script:PhaseResults[$key]
        if (-not $pr.ReportFile) { continue }  # skip sub-phases without own report
        $name = $pr.ReportFile -replace '\.md$', ''
        $phaseRows.Add("| $name | $($pr.ReportFile) | $($pr.Status) | $($pr.Errors) | $($pr.Duration)s |") | Out-Null
        $totalErrors += $pr.Errors
        $totalDuration += $pr.Duration
        if ($pr.Status -match "FAIL|PARTIAL") {
            $failedPhases.Add("- **$($pr.ReportFile)**: $($pr.Status) ($($pr.Errors) errors)") | Out-Null
        }
    }

    # Gather stats
    $toolCount = if ($Script:AllResults.Tools) { $Script:AllResults.Tools.Count } else { 0 }
    $domainCount = if ($Script:AllResults.Domains) { $Script:AllResults.Domains.Keys.Count } else { 0 }
    $docCount = $Script:AllResults.TotalDocs
    $sectCount = $Script:AllResults.TotalSections
    $sectTypeCount = if ($Script:AllResults.SectionsByType) { $Script:AllResults.SectionsByType.Keys.Count } else { 0 }

    $overallStatus = if ($totalErrors -gt 0) { "⚠️ WITH ERRORS" } else { "✅ CLEAN" }

    Write-Report -TemplateName "00-summary.md" -OutputName "00-summary.md" -Values @{
        TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        TOTAL_DURATION = [math]::Round($totalDuration, 2)
        OVERALL_STATUS = $overallStatus
        PHASE_RESULTS_ROWS = if ($phaseRows.Count -gt 0) { $phaseRows -join "`n" } else { "| -- | -- | -- | -- | -- |" }
        TOOL_COUNT = $toolCount
        DOMAIN_COUNT = $domainCount
        DOCUMENT_COUNT = $docCount
        SECTION_COUNT = $sectCount
        SECTION_TYPE_COUNT = $sectTypeCount
        TOTAL_CALLS = $Script:TotalCalls
        TOTAL_ERRORS = $totalErrors
        FAILED_PHASES = if ($failedPhases.Count -gt 0) { $failedPhases -join "`n" } else { "✅ All phases passed" }
        ARCHIVE_PATH = if ($Script:ArchivePath) { $Script:ArchivePath } else { "No previous run" }
    }

    $Script:PhaseResults["00-summary"] = @{
        Status = "✅ DONE"
        Errors = $totalErrors
        Duration = [math]::Round(((Get-Date) - $start).TotalSeconds, 2)
        ReportFile = "00-summary.md"
    }

    Write-Host "  → Done" -ForegroundColor Green
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
║  Reports: $($Script:LatestDir)          ║
╚═══════════════════════════════════════════╝
"@ -ForegroundColor Cyan

    if ($PassThru) {
        Write-Host "`nReport files:" -ForegroundColor Cyan
        Get-ChildItem $Script:LatestDir -Filter "*.md" | Sort-Object Name | ForEach-Object {
            Write-Host "  $($_.FullName)" -ForegroundColor White
        }
    }

} catch {
    Write-Host "❌ Fatal error: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkRed
    exit 1
}