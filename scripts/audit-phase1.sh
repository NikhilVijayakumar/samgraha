#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKUP_PATH="$ROOT_DIR/samgraha.toml.phase1bak"

usage() {
    echo "Usage: $0 [--keep] [--restore]" >&2
    exit 1
}

KEEP=false
RESTORE=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --keep)    KEEP=true; shift ;;
        --restore) RESTORE=true; shift ;;
        *)         usage ;;
    esac
done

cd "$ROOT_DIR"

if $RESTORE; then
    if [[ ! -f "$BACKUP_PATH" ]]; then
        echo "No backup found at $BACKUP_PATH" >&2
        exit 1
    fi
    mv "$BACKUP_PATH" "samgraha.toml"
    echo "Config restored from backup"
    exit 0
fi

if [[ -f "$BACKUP_PATH" ]]; then
    echo "WARN stale backup found -- restoring first" >&2
    mv "$BACKUP_PATH" "samgraha.toml"
fi

cp "samgraha.toml" "$BACKUP_PATH"
echo "Config backed up -> samgraha.toml.phase1bak"
echo "Run Phase 1 commands, then: $0 --restore"
