param(
    [switch]$Keep,
    [switch]$Restore
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $PSScriptRoot
$BackupPath = Join-Path $RootDir "samgraha.toml.phase1bak"

Push-Location $RootDir
try {
    if ($Restore) {
        if (-not (Test-Path $BackupPath)) {
            Write-Host "No backup found at $BackupPath" -ForegroundColor Red
            exit 1
        }
        Move-Item $BackupPath "samgraha.toml" -Force
        Write-Host "Config restored from backup" -ForegroundColor Green
        return
    }

    if (Test-Path $BackupPath) {
        Write-Host "WARN stale backup found -- restoring first" -ForegroundColor Yellow
        Move-Item $BackupPath "samgraha.toml" -Force
    }

    Copy-Item "samgraha.toml" $BackupPath
    Write-Host "Config backed up -> samgraha.toml.phase1bak" -ForegroundColor Green
    Write-Host "Run Phase 1 commands, then: .\scripts\audit-phase1.ps1 -Restore" -ForegroundColor Cyan
} finally {
    Pop-Location
}
