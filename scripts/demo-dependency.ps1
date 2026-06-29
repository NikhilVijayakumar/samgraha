param(
    [switch]$Keep,
    [switch]$Resolve
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $PSScriptRoot
$tmp = "$env:TEMP\samgraha-demo-$(Get-Random)"
$ConfigBackup = Join-Path $RootDir "samgraha.toml.bak"

Write-Host "Creating fixture at $tmp" -ForegroundColor Cyan

New-Item -ItemType Directory -Force "$tmp\docs\architecture", "$tmp\docs\feature", "$tmp\docs\engineering" | Out-Null

@"
[repository]
id = "astra"
name = "astra test"
"@ | Set-Content "$tmp\samgraha.toml"

@"
# System Overview

## Purpose

Text.

## Constraints

- Offline
- Deterministic
"@ | Set-Content "$tmp\docs\architecture\system-overview.md"

@"
# Compilation

## Purpose

Transform docs.

## Requirements

- FTS
- Progressive
"@ | Set-Content "$tmp\docs\feature\knowledge-compilation.md"

@"
# Build

## Purpose

Build workflows.

## Toolchain

- Cargo
- Rust analyzer
"@ | Set-Content "$tmp\docs\engineering\build-system.md"

Push-Location $tmp
try {
    Write-Host "`nCompiling astra..." -ForegroundColor Cyan
    cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- compile
    if ($LASTEXITCODE -ne 0) { throw "compile failed" }

    Write-Host "`nRegistering astra..." -ForegroundColor Cyan
    cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- registry register
    if ($LASTEXITCODE -ne 0) { throw "register failed" }

    Write-Host "`nRegistry list:" -ForegroundColor Cyan
    cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- registry list

    if ($Resolve) {
        # Phase 1.5 — Dependency Resolution
        Write-Host "`n--- Phase 1.5: Dependency Resolution ---" -ForegroundColor Cyan

        # Pop to project root so resolve finds project's registry and docs
        Pop-Location

        try {
            # Backup original config
            Copy-Item "samgraha.toml" $ConfigBackup -Force

            # Add dependency on astra with path to compiled fixture
            $tomlPath = $tmp.Replace('\', '/')
            $cfg = Get-Content "samgraha.toml" -Raw
            $depCfg = "`n[[repository.dependencies]]`nname = `"astra`"`npath = `"$tomlPath`"`nrequired = true"
            $cfg + $depCfg | Set-Content "samgraha.toml"

            Write-Host "`nResolving dependencies..." -ForegroundColor Cyan
            cargo run --manifest-path "$RootDir\Cargo.toml" --bin cli -- registry resolve runtime
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  OK resolve succeeded" -ForegroundColor Green
            } else {
                Write-Host "  XX resolve failed" -ForegroundColor Red
            }

            # Phase 1.15 — Cleanup: restore original config
            Write-Host "`n--- Phase 1.15: Cleanup ---" -ForegroundColor Cyan
            Copy-Item $ConfigBackup "samgraha.toml" -Force
            Write-Host "  OK samgraha.toml restored" -ForegroundColor Green

        } finally {
            if (Test-Path $ConfigBackup) { Remove-Item $ConfigBackup -Force }
        }
        # Push back to fixture for cleanup
        Push-Location $tmp
    }

} finally {
    Pop-Location
    if (-not $Keep) {
        Write-Host "`nCleaning up $tmp" -ForegroundColor DarkGray
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    } else {
        Write-Host "`nFixture kept at $tmp (use -Keep)" -ForegroundColor Yellow
    }
}
