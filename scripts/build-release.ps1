$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$root = (Resolve-Path $root).Path

# Read .env -- single source of truth, no CLI overrides
$expiryDays  = 30
$expiryHours = 0
$outputDir   = ""

$envFile = Join-Path $root ".env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and $line -notlike '#*' -and $line -like '*=*') {
            $kv = $line.Split('=', 2)
            $k  = $kv[0].Trim()
            $v  = $kv[1].Trim().Trim('"', "'")
            switch ($k) {
                "SAMGRAHA_EXPIRY_DAYS"  { $expiryDays  = [int]$v }
                "SAMGRAHA_EXPIRY_HOURS" { $expiryHours = [int]$v }
                "OUTPUT_DIR"            { $outputDir   = $v }
            }
        }
    }
}

# Resolve output dir -- prefer absolute path from .env
if (-not $outputDir) {
    Write-Warning "OUTPUT_DIR not set in .env -- falling back to .\release. Set an absolute path in .env."
    $outputDir = ".\release"
}
if (-not [System.IO.Path]::IsPathRooted($outputDir)) {
    Write-Warning "OUTPUT_DIR '$outputDir' is relative -- resolving from project root. Use an absolute path in .env."
    $outputDir = Join-Path $root $outputDir
}
$outputDir = (New-Item -ItemType Directory -Force $outputDir).FullName

# Compute expiry label for display and launcher comment
# build.rs owns baking this into the binary -- scripts just show the same value
if ($expiryDays -eq -1) {
    $expiryLabel   = "never"
    $expiryComment = "no expiry"
} else {
    $hrs = if ($expiryHours -eq -1) { 0 } else { $expiryHours }
    $expiry = (Get-Date).ToUniversalTime().AddDays($expiryDays).AddHours($hrs)
    $expiryLabel   = $expiry.ToString("yyyy-MM-ddTHH:mm:ssZ")
    $expiryComment = "expires $expiryLabel"
}
Write-Host "Expiry: $expiryLabel  (days=$expiryDays, hours=$expiryHours)" -ForegroundColor Cyan

# Build -- build.rs reads .env and bakes SAMGRAHA_EXPIRY into the binary
Write-Host "Building mcp.exe + cli.exe (release)..." -ForegroundColor Yellow
& cargo build --release --bin mcp --bin cli --manifest-path "$root\Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

# Package directory
$pkgDir = Join-Path $outputDir "samgraha"
if (Test-Path $pkgDir) { Remove-Item -Recurse -Force $pkgDir }
foreach ($d in @("$pkgDir\bin", "$pkgDir\docs\raw", "$pkgDir\.samgraha")) {
    New-Item -ItemType Directory -Force $d | Out-Null
}

# Copy binaries
Copy-Item "$root\target\release\mcp.exe" "$pkgDir\bin\"
Copy-Item "$root\target\release\cli.exe" "$pkgDir\bin\"

# Copy config + universal standards only (samgraha-specific docs stay in the source repo)
Copy-Item "$root\samgraha.toml" "$pkgDir\"
foreach ($dir in @("standards", "audit", "audit-standards")) {
    if (Test-Path "$root\docs\raw\$dir") {
        Copy-Item -Recurse "$root\docs\raw\$dir" "$pkgDir\docs\raw\" -Force
    }
}

# === Built-in Knowledge Sources ===
$builtinSources = @(
    @{ name = "standards"; path = "docs/raw/standards" },
    @{ name = "help"; path = "docs/raw/product-guide" }
)

foreach ($src in $builtinSources) {
    $rawPath = Join-Path $root $src.path
    if (-not (Test-Path $rawPath)) {
        Write-Warning "$($src.name) source not found at $rawPath -- skipping"
        continue
    }
    Write-Host "==> Compiling $($src.name) documentation..." -ForegroundColor Yellow
    & "$pkgDir\bin\cli.exe" compile --config "$root\samgraha.toml" $rawPath --domain $($src.name) --force
    if ($LASTEXITCODE -ne 0) {
        throw "$($src.name) compile failed (exit $LASTEXITCODE)"
    }
    $dbSource = Join-Path $rawPath ".samgraha\knowledge.db"
    # load_builtin_stores() (crates/services/src/builtin.rs) looks next to the running
    # binary (current_exe().parent()), i.e. bin/ — not the package root.
    $dbTarget = Join-Path "$pkgDir\bin" "$($src.name).db"
    if (Test-Path $dbSource) {
        Copy-Item $dbSource $dbTarget -Force
        Write-Host "  -> $dbTarget" -ForegroundColor Cyan
    } else {
        throw "$($src.name) compile produced no knowledge.db at $dbSource"
    }
}

# Launcher scripts
$runCmdLines = @(
    '@echo off',
    "rem Samgraha MCP - $expiryComment",
    '"%~dp0bin\mcp.exe" %*'
)
$runCmd = $runCmdLines -join "`r`n"
Set-Content -Path "$pkgDir\run-mcp.cmd" -Value $runCmd -Encoding ASCII

$runShLines = @(
    '#!/usr/bin/env sh',
    "# Samgraha MCP - $expiryComment",
    'exec "$(dirname "$0")/bin/mcp.exe" "$@"'
)
$runSh = $runShLines -join "`n"
Set-Content -Path "$pkgDir\run-mcp.sh" -Value $runSh -Encoding ASCII

# Checksums
$mcpHash = (Get-FileHash "$pkgDir\bin\mcp.exe" -Algorithm SHA256).Hash.ToLower()
$cliHash = (Get-FileHash "$pkgDir\bin\cli.exe" -Algorithm SHA256).Hash.ToLower()
$sumsLines = @(
    "$mcpHash  bin/mcp.exe",
    "$cliHash  bin/cli.exe"
)
$sums = $sumsLines -join "`r`n"
Set-Content -Path "$pkgDir\SHA256SUMS" -Value $sums -Encoding ASCII

$mcpSize = [int]((Get-Item "$pkgDir\bin\mcp.exe").Length / 1KB)
$cliSize = [int]((Get-Item "$pkgDir\bin\cli.exe").Length / 1KB)

Write-Host "`n=== Release packaged ===" -ForegroundColor Green
Write-Host "  Location: $pkgDir" -ForegroundColor Cyan
Write-Host ("  mcp.exe:  {0}KB  ({1})" -f $mcpSize, $mcpHash) -ForegroundColor Cyan
Write-Host ("  cli.exe:  {0}KB  ({1})" -f $cliSize, $cliHash) -ForegroundColor Cyan
Write-Host "  Expiry:   $expiryLabel" -ForegroundColor Yellow
Write-Host '  Use:      Get-Content input.json | .\run-mcp.cmd' -ForegroundColor Gray
