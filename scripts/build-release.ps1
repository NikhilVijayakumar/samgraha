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
foreach ($d in @("$pkgDir\bin", "$pkgDir\.samgraha")) {
    New-Item -ItemType Directory -Force $d | Out-Null
}

# Copy binaries
Copy-Item "$root\target\release\mcp.exe" "$pkgDir\bin\"
Copy-Item "$root\target\release\cli.exe" "$pkgDir\bin\"

# Copy config
Copy-Item "$root\samgraha.toml" "$pkgDir\"

# Ship reference schema -- not read at runtime (register_standard/step
# execution create + migrate .samgraha/knowledge.db on demand via the
# inline Rust migrations in core_schema.rs), just documentation for
# anyone integrating with the raw DB directly.
New-Item -ItemType Directory -Force "$pkgDir\schema\registration" | Out-Null
New-Item -ItemType Directory -Force "$pkgDir\schema\knowledge" | Out-Null
Copy-Item "$root\schema\registration\*.sql" "$pkgDir\schema\registration\" -Force
Copy-Item "$root\schema\knowledge\*.sql" "$pkgDir\schema\knowledge\" -Force
Write-Host "  -> schema/registration/, schema/knowledge/ (reference schema)" -ForegroundColor Cyan

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
