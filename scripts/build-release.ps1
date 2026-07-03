param(
    [int]$ExpiryDays = 30,
    [string]$OutputDir = ".\release"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$root = (Resolve-Path $root).Path

# Resolve output dir relative to project root
if (-not [System.IO.Path]::IsPathRooted($OutputDir)) {
    $OutputDir = Join-Path $root $OutputDir
}
$OutputDir = (New-Item -ItemType Directory -Force $OutputDir).FullName

# Calculate expiry at end of day (23:59:59 UTC)
$expiry = (Get-Date).AddDays($ExpiryDays).ToUniversalTime()
$expiryRfc = $expiry.ToString("yyyy-MM-ddTHH:mm:ssZ")
Write-Host "Building with expiry: $expiryRfc (+${ExpiryDays}d at 23:59:59 UTC)" -ForegroundColor Cyan

# Build release binaries with expiry baked in
Write-Host "Building mcp.exe + cli.exe (release)..." -ForegroundColor Yellow
$env:SAMGRAHA_EXPIRY = $expiryRfc
& cargo build --release --bin mcp --bin cli --manifest-path "$root\Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

# Package directory
$pkgDir = Join-Path $OutputDir "samgraha"
if (Test-Path $pkgDir) { Remove-Item -Recurse -Force $pkgDir }
foreach ($d in @("$pkgDir\bin", "$pkgDir\docs\raw", "$pkgDir\.samgraha")) {
    New-Item -ItemType Directory -Force $d | Out-Null
}

# Copy binaries
Copy-Item "$root\target\release\mcp.exe" "$pkgDir\bin\"
Copy-Item "$root\target\release\cli.exe" "$pkgDir\bin\"

# Copy config + docs
Copy-Item "$root\samgraha.toml" "$pkgDir\"
Copy-Item -Recurse "$root\docs\raw\*" "$pkgDir\docs\raw\" -Force

# Pre-compile knowledge base so target machine skips cargo
Write-Host "Pre-compiling knowledge base..." -ForegroundColor Yellow
Push-Location $pkgDir
& ".\bin\cli.exe" compile --force
Pop-Location

# Launcher scripts
@"
@echo off
rem Samgraha MCP — expires $expiryRfc
"@ | Set-Content -Path "$pkgDir\run-mcp.cmd" -Encoding ASCII

@"
#!/usr/bin/env sh
# Samgraha MCP — expires $expiryRfc
exec "`$(dirname `"`$0`")/bin/mcp" `"`$@`"
"@ | Set-Content -Path "$pkgDir\run-mcp.sh" -Encoding ASCII

$mcpSize = [int]((Get-Item "$pkgDir\bin\mcp.exe").Length / 1KB)
$cliSize = [int]((Get-Item "$pkgDir\bin\cli.exe").Length / 1KB)

Write-Host "`n=== Release packaged ===" -ForegroundColor Green
Write-Host "  Location: $pkgDir" -ForegroundColor Cyan
Write-Host "  mcp.exe:  ${mcpSize}KB" -ForegroundColor Cyan
Write-Host "  cli.exe:  ${cliSize}KB" -ForegroundColor Cyan
Write-Host "  Expires:  $expiryRfc" -ForegroundColor Yellow
Write-Host "  Use:      Get-Content input.json | .\run-mcp.cmd" -ForegroundColor Gray
Write-Host ""
Write-Host "  To extend: .\scripts\build-release.ps1 -ExpiryDays 60 -OutputDir D:\releases" -ForegroundColor Gray
