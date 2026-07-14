#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Read .env — single source of truth, no CLI overrides
EXPIRY_DAYS=30
EXPIRY_HOURS=0
OUTPUT_DIR=""

ENV_FILE="$ROOT_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
    while IFS='=' read -r key val; do
        key="${key#"${key%%[![:space:]]*}"}"; key="${key%"${key##*[![:space:]]}"}"
        val="${val#"${val%%[![:space:]]*}"}"; val="${val%"${val##*[![:space:]]}"}"
        val="${val#\"}"; val="${val%\"}"
        val="${val#\'}"; val="${val%\'}"
        [[ -z "$key" || "$key" =~ ^# ]] && continue
        case "$key" in
            SAMGRAHA_EXPIRY_DAYS)  EXPIRY_DAYS="$val" ;;
            SAMGRAHA_EXPIRY_HOURS) EXPIRY_HOURS="$val" ;;
            OUTPUT_DIR)            OUTPUT_DIR="$val" ;;
        esac
    done < <(grep -v '^\s*#' "$ENV_FILE" | grep '=')
fi

# Resolve output dir — prefer absolute path from .env
if [[ -z "$OUTPUT_DIR" ]]; then
    echo "WARNING: OUTPUT_DIR not set in .env — falling back to ./release. Set an absolute path in .env." >&2
    OUTPUT_DIR="./release"
fi
OUTPUT_DIR="${OUTPUT_DIR//\\//}"
if [[ "$OUTPUT_DIR" != /* ]]; then
    echo "WARNING: OUTPUT_DIR '$OUTPUT_DIR' is relative — resolving from project root. Use an absolute path in .env." >&2
    OUTPUT_DIR="$ROOT_DIR/${OUTPUT_DIR#./}"
fi
mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"

# Compute expiry label for display and launcher comment
# build.rs owns baking this into the binary — scripts just show the same value
if [[ "$EXPIRY_DAYS" == "-1" ]]; then
    EXPIRY_LABEL="never"
    EXPIRY_COMMENT="no expiry"
else
    HRS=$(( EXPIRY_HOURS == -1 ? 0 : EXPIRY_HOURS ))
    TOTAL_MINS=$(( EXPIRY_DAYS * 1440 + HRS * 60 ))
    EXPIRY_RFC=$(date -u -d "+${TOTAL_MINS} minutes" '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null \
        || date -u -v+${TOTAL_MINS}M '+%Y-%m-%dT%H:%M:%SZ')
    EXPIRY_LABEL="$EXPIRY_RFC"
    EXPIRY_COMMENT="expires $EXPIRY_RFC"
fi
echo "Expiry: $EXPIRY_LABEL  (days=$EXPIRY_DAYS, hours=$EXPIRY_HOURS)"

# Build — build.rs reads .env and bakes SAMGRAHA_EXPIRY into the binary
echo "Building mcp + cli (release)..."
cargo build --release --bin mcp --bin cli --manifest-path "$ROOT_DIR/Cargo.toml"

# Package directory
PKG_DIR="$OUTPUT_DIR/samgraha"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/bin" "$PKG_DIR/docs/raw" "$PKG_DIR/.samgraha"

# Copy binaries
cp "$ROOT_DIR/target/release/mcp" "$PKG_DIR/bin/"
cp "$ROOT_DIR/target/release/cli" "$PKG_DIR/bin/"

# Strip debug info to reduce size
if command -v strip &>/dev/null; then
    strip "$PKG_DIR/bin/mcp" "$PKG_DIR/bin/cli"
fi

# Copy config + universal standards only (samgraha-specific docs stay in the source repo)
cp "$ROOT_DIR/samgraha.toml" "$PKG_DIR/"
for dir in documentation-standards audit audit-standards; do
    if [[ -d "$ROOT_DIR/docs/raw/$dir" ]]; then
        cp -r "$ROOT_DIR/docs/raw/$dir" "$PKG_DIR/docs/raw/"
    fi
done

# === Built-in Knowledge Sources ===
# load_builtin_stores() (crates/services/src/builtin.rs) looks next to the running
# binary (current_exe().parent()), i.e. bin/ — not the package root.
# Only help.db is compiled from source; knowledge.db ships as an empty schema
# that gets populated when a user registers their Knowledge System.
declare -A BUILTIN_SOURCES=(
    [help]="docs/raw/product-guide"
)
for name in "${!BUILTIN_SOURCES[@]}"; do
    raw_path="$ROOT_DIR/${BUILTIN_SOURCES[$name]}"
    if [[ ! -d "$raw_path" ]]; then
        echo "WARNING: $name source not found at $raw_path -- skipping" >&2
        continue
    fi
    echo "==> Compiling $name documentation..."
    "$PKG_DIR/bin/cli" compile --config "$ROOT_DIR/samgraha.toml" "$raw_path" --domain "$name" --force
    db_source="$raw_path/.samgraha/knowledge.db"
    db_target="$PKG_DIR/bin/$name.db"
    if [[ -f "$db_source" ]]; then
        cp "$db_source" "$db_target"
        echo "  -> $db_target"
    else
        echo "ERROR: $name compile produced no knowledge.db at $db_source" >&2
        exit 1
    fi
done

# === Empty Knowledge DB (schema only) ===
# knowledge.db ships with the schema skeleton but no system rows.
# Users populate it by running `standards register` with their Knowledge System.
SCHEMA_DIR="$ROOT_DIR/schema/knowledge-hub"
KNOWLEDGE_DB="$PKG_DIR/bin/knowledge.db"
if [[ -f "$SCHEMA_DIR/knowledge-hub-loader.py" ]]; then
    echo "==> Creating empty knowledge.db (schema only)..."
    python3 -c "
import sqlite3, glob, os
conn = sqlite3.connect('$KNOWLEDGE_DB')
conn.execute('PRAGMA foreign_keys = ON')
for f in sorted(glob.glob('$SCHEMA_DIR/[0-9]*.sql')):
    with open(f) as fh:
        conn.executescript(fh.read())
conn.execute('PRAGMA user_version = 1')
conn.close()
print('  -> $KNOWLEDGE_DB (empty schema)')
"
fi

# Ship schema + loader for Knowledge System registration
mkdir -p "$PKG_DIR/schema/knowledge-hub"
cp "$SCHEMA_DIR"/*.sql "$PKG_DIR/schema/knowledge-hub/"
cp "$SCHEMA_DIR/knowledge-hub-loader.py" "$PKG_DIR/schema/knowledge-hub/"
echo "  -> schema/knowledge-hub/ (loader + schema files)"

# Launcher scripts (Linux build: binaries have no .exe)
cat > "$PKG_DIR/run-mcp.sh" <<SHEOF
#!/usr/bin/env sh
# Samgraha MCP — $EXPIRY_COMMENT
exec "\$(dirname "\$0")/bin/mcp" "\$@"
SHEOF
chmod +x "$PKG_DIR/run-mcp.sh"

cat > "$PKG_DIR/run-mcp.cmd" <<CMDEOF
@echo off
rem Samgraha MCP — $EXPIRY_COMMENT
"%~dp0bin\mcp" %*
CMDEOF

# Checksums
if command -v sha256sum &>/dev/null; then
    sha256sum "$PKG_DIR/bin/mcp" "$PKG_DIR/bin/cli" \
        | sed "s|$PKG_DIR/||" > "$PKG_DIR/SHA256SUMS"
elif command -v shasum &>/dev/null; then
    shasum -a 256 "$PKG_DIR/bin/mcp" "$PKG_DIR/bin/cli" \
        | sed "s|$PKG_DIR/||" > "$PKG_DIR/SHA256SUMS"
fi

MCP_SIZE=$(wc -c < "$PKG_DIR/bin/mcp"); MCP_SIZE=$((MCP_SIZE / 1024))
CLI_SIZE=$(wc -c < "$PKG_DIR/bin/cli"); CLI_SIZE=$((CLI_SIZE / 1024))
MCP_HASH=$(awk 'NR==1{print $1}' "$PKG_DIR/SHA256SUMS" 2>/dev/null || echo "n/a")
CLI_HASH=$(awk 'NR==2{print $1}' "$PKG_DIR/SHA256SUMS" 2>/dev/null || echo "n/a")

echo ""
echo "=== Release packaged ==="
echo "  Location: $PKG_DIR"
echo "  mcp:      ${MCP_SIZE}KB  ($MCP_HASH)"
echo "  cli:      ${CLI_SIZE}KB  ($CLI_HASH)"
echo "  Expiry:   $EXPIRY_LABEL"
echo "  Use:      echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}' | ./run-mcp.sh"
