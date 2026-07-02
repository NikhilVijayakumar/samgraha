#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TMP=$(mktemp -d "/tmp/samgraha-demo-XXXXXX")
CONFIG_BACKUP="$ROOT_DIR/samgraha.toml.bak"

KEEP=false
RESOLVE=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --keep)    KEEP=true; shift ;;
        --resolve) RESOLVE=true; shift ;;
        *)         echo "Usage: $0 [--keep] [--resolve]"; exit 1 ;;
    esac
done

cleanup() {
    if ! $KEEP; then
        echo "Cleaning up $TMP" >&2
        rm -rf "$TMP"
    else
        echo "Fixture kept at $TMP (use --keep)" >&2
    fi
}
trap cleanup EXIT

echo "Creating fixture at $TMP"

mkdir -p "$TMP/docs/architecture" "$TMP/docs/feature" "$TMP/docs/engineering"

cat > "$TMP/samgraha.toml" << 'EOF'
[repository]
id = "astra"
name = "astra test"
EOF

cat > "$TMP/docs/architecture/system-overview.md" << 'EOF'
# System Overview

## Purpose

Text.

## Constraints

- Offline
- Deterministic
EOF

cat > "$TMP/docs/feature/knowledge-compilation.md" << 'EOF'
# Compilation

## Purpose

Transform docs.

## Requirements

- FTS
- Progressive
EOF

cat > "$TMP/docs/engineering/build-system.md" << 'EOF'
# Build

## Purpose

Build workflows.

## Toolchain

- Cargo
- Rust analyzer
EOF

cd "$TMP"

echo -e "\nCompiling astra..."
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin cli -- compile

echo -e "\nRegistering astra..."
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin cli -- registry register

echo -e "\nRegistry list:"
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin cli -- registry list

if $RESOLVE; then
    echo -e "\n--- Phase 1.5: Dependency Resolution ---"

    cd "$ROOT_DIR"

    # Ensure config is always restored, even on error.
    cleanup_config() {
        if [ -f "$CONFIG_BACKUP" ]; then
            cp "$CONFIG_BACKUP" "samgraha.toml"
            rm -f "$CONFIG_BACKUP"
            echo "  OK samgraha.toml restored"
        fi
    }
    trap cleanup_config EXIT

    cp "samgraha.toml" "$CONFIG_BACKUP"

    TOML_PATH="${TMP//\/\//\/}"
    cat >> "samgraha.toml" << EOF

[[repository.dependencies]]
name = "astra"
path = "$TOML_PATH"
required = true
EOF

    echo -e "\nResolving dependencies..."
    if cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin cli -- registry resolve runtime; then
        echo "  OK resolve succeeded"
    else
        echo "  XX resolve failed"
    fi

    cleanup_config
    trap - EXIT

    cd "$TMP"
fi
