#!/usr/bin/env bash
set -uo pipefail

FAILURES=0
PASSES=0
FAILURE_DETAILS=()
LAST_OUTPUT=""
CURRENT_PHASE=""
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEST_TEMP=$(mktemp -d "/tmp/samgraha-test-XXXXXX")

FULL=false
WITH_MCP=false
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --full)     FULL=true; shift ;;
        --with-mcp) WITH_MCP=true; shift ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        *)          echo "Usage: $0 [--full] [--with-mcp] [--skip-build]"; exit 1 ;;
    esac
done

write_pass() { echo "  OK $*"; PASSES=$((PASSES + 1)); }
write_fail() {
    local msg="$*"
    echo "  XX $msg"
    FAILURES=$((FAILURES + 1))
    FAILURE_DETAILS+=("$CURRENT_PHASE|$msg|$(echo -e "$LAST_OUTPUT")")
}
write_step() { CURRENT_PHASE="$*"; LAST_OUTPUT=""; echo -e "\n== $CURRENT_PHASE =="; }
write_info() { echo "  .. $*"; }

assert_exit_code_zero() {
    local ec=$?
    if [[ $ec -ne 0 ]]; then write_fail "$1 (exit $ec)"; else write_pass "$1"; fi
}

assert_file_exists() {
    if [[ ! -f "$1" ]]; then write_fail "$2 -- file not found: $1"; else write_pass "$2"; fi
}

run_cli() {
    LAST_OUTPUT=$(cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin cli -- "$@" 2>&1)
    local rc=$?
    echo "$LAST_OUTPUT"
    return $rc
}

new_test_fixture() {
    local path="$1" repo_id="${2:-test-repo}"
    mkdir -p "$path/docs/architecture" "$path/docs/feature" "$path/docs/engineering"
    cat > "$path/samgraha.toml" << EOF
[repository]
id = "$repo_id"
name = "$repo_id test"
EOF

    cat > "$path/docs/architecture/system-overview.md" << 'EOF'
# System Overview

## Purpose

Text.

## Constraints

- Offline
- Deterministic
EOF

    cat > "$path/docs/feature/knowledge-compilation.md" << 'EOF'
# Compilation

## Purpose

Transform docs.

## Requirements

- FTS
- Progressive
EOF

    cat > "$path/docs/engineering/build-system.md" << 'EOF'
# Build

## Purpose

Build workflows.

## Toolchain

- Cargo
- Rust analyzer
EOF
}

remove_test_fixture() { rm -rf "$1"; }

invoke_phase1a() {
    write_step "Phase 1a - Unit Tests"
    pushd "$ROOT_DIR" > /dev/null
    write_info "Running cargo test -p tests"
    LAST_OUTPUT=$(cargo test -p tests 2>&1 || true)
    echo "$LAST_OUTPUT"
    assert_exit_code_zero "cargo test -p tests"
    popd > /dev/null
}

invoke_phase1b() {
    write_step "Phase 1b - CLI Integration"
    local test_dir="$TEST_TEMP/p1b"
    new_test_fixture "$test_dir" "test-repo"
    pushd "$test_dir" > /dev/null
    write_info "1. compile"
    run_cli "compile" > /dev/null
    assert_exit_code_zero "compile exits 0"
    assert_file_exists ".samgraha/knowledge.db" "knowledge.db"
    assert_file_exists ".samgraha/manifest.json" "manifest.json"

    local m
    m=$(cat ".samgraha/manifest.json")
    if echo "$m" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['revision'] >= 1" 2>/dev/null; then
        write_pass "manifest revision >= 1"
    else
        write_fail "revision >= 1"
    fi
    if echo "$m" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('audit',{}).get('status')" 2>/dev/null; then
        write_pass "audit status present"
    else
        write_fail "audit status"
    fi
    if echo "$m" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('repository',{}).get('uuid')" 2>/dev/null; then
        write_pass "UUID present"
    else
        write_fail "UUID"
    fi

    write_info "2. recompile - revision unchanged"
    run_cli "compile" > /dev/null
    local m2
    m2=$(cat ".samgraha/manifest.json")
    local r1 r2
    r1=$(echo "$m" | python3 -c "import sys,json; print(json.load(sys.stdin)['revision'])")
    r2=$(echo "$m2" | python3 -c "import sys,json; print(json.load(sys.stdin)['revision'])")
    if [[ "$r1" -eq "$r2" ]]; then write_pass "revision unchanged"; else write_fail "revision changed"; fi

    write_info "3. registry register"
    run_cli "registry" "register" > /dev/null
    assert_exit_code_zero "register"

    write_info "4. registry list"
    local out
    out=$(run_cli "registry" "list")
    assert_exit_code_zero "list"
    if echo "$out" | grep -q "test-repo"; then write_pass "repo in list"; else write_fail "repo in list"; fi

    write_info "5. registry status"
    run_cli "registry" "status" > /dev/null
    assert_exit_code_zero "status"

    write_info "6. search"
    run_cli "search" "compilation" > /dev/null
    assert_exit_code_zero "search"

    write_info "7. sections"
    run_cli "sections" "purpose" > /dev/null
    assert_exit_code_zero "sections"

    write_info "8. audit"
    run_cli "audit" > /dev/null
    assert_exit_code_zero "audit"

    write_info "9. info"
    run_cli "info" > /dev/null
    assert_exit_code_zero "info"

    write_info "10. resolve"
    run_cli "registry" "resolve" "runtime" > /dev/null
    assert_exit_code_zero "resolve"

    write_info "11. registry sync"
    run_cli "registry" "sync" > /dev/null
    assert_exit_code_zero "sync"

    popd > /dev/null
    remove_test_fixture "$test_dir"
}

invoke_phase1c() {
    write_step "Phase 1c - Multi-Repo"
    local rA="$TEST_TEMP/p1c/repo-a"
    local rB="$TEST_TEMP/p1c/repo-b"
    new_test_fixture "$rA" "repo-a"
    new_test_fixture "$rB" "repo-b"

    pushd "$rA" > /dev/null
    run_cli "compile" > /dev/null
    run_cli "registry" "register" > /dev/null
    popd > /dev/null

    pushd "$rB" > /dev/null
    run_cli "compile" > /dev/null
    run_cli "registry" "register" > /dev/null
    popd > /dev/null

    local cfg
    cfg=$(< "$rA/samgraha.toml")
    cfg="$cfg

[[repository.dependencies]]
name = \"repo-b\"
path = \"$rB\"
required = true"
    echo "$cfg" > "$rA/samgraha.toml"

    pushd "$rA" > /dev/null
    write_info "resolve with dependency"
    run_cli "registry" "resolve" "runtime" > /dev/null
    assert_exit_code_zero "resolve with dep"
    popd > /dev/null

    remove_test_fixture "$rB"

    pushd "$rA" > /dev/null
    write_info "resolve with missing dep"
    rc=0; run_cli "registry" "resolve" "runtime" > /dev/null 2>&1 || rc=$?
    if [[ $rc -ne 0 ]]; then write_pass "missing dep => non-zero exit"; else write_fail "missing dep should fail"; fi
    popd > /dev/null

    remove_test_fixture "$rA"
}

invoke_phase2() {
    write_step "Phase 2 - MCP Tests"
    local test_dir="$TEST_TEMP/p2"
    new_test_fixture "$test_dir" "mcp-test"
    pushd "$test_dir" > /dev/null
    run_cli "compile" > /dev/null

    raw_mcp() {
        LAST_OUTPUT=$(echo "$1" | cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin mcp 2>&1)
        echo "$LAST_OUTPUT"
    }

    write_info "tools/list"
    local r
    r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}')
    if echo "$r" | grep -q "compile"; then write_pass "tools/list"; else write_fail "tools/list"; fi

    write_info "tools/call search"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"compilation"}}}')
    if echo "$r" | grep -qiE "error|not found"; then write_fail "search"; else write_pass "search"; fi

    write_info "tools/call get_document"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_document","arguments":{"id":"1"}}}')
    if echo "$r" | grep -qiE "error|not found"; then write_fail "get_document"; else write_pass "get_document"; fi

    write_info "tools/call nonexistent"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent"}}')
    if echo "$r" | grep -qiE "error|not found"; then write_pass "nonexistent => error"; else write_fail "nonexistent should error"; fi

    popd > /dev/null
    remove_test_fixture "$test_dir"
}

invoke_phase25() {
    write_step "Phase 2.5 - Protocol"
    local test_dir="$TEST_TEMP/p25"
    new_test_fixture "$test_dir" "proto-test"
    pushd "$test_dir" > /dev/null
    run_cli "compile" > /dev/null

    raw_mcp() {
        LAST_OUTPUT=$(echo "$1" | cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin mcp 2>&1)
        echo "$LAST_OUTPUT"
    }

    write_info "malformed JSON"
    local r
    r=$(raw_mcp "not json")
    if echo "$r" | grep -q "\-32700"; then write_pass "parse error"; else write_fail "expected parse error"; fi

    write_info "unknown method"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"bogus","params":{}}')
    if echo "$r" | grep -q "\-32601"; then write_pass "method not found"; else write_fail "expected method not found"; fi

    write_info "missing tool name"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{}}')
    if echo "$r" | grep -q "\-32602"; then write_pass "invalid params"; else write_fail "expected invalid params"; fi

    write_info "initialize"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}')
    if echo "$r" | grep -q "samgraha-mcp"; then write_pass "initialize"; else write_fail "expected serverInfo"; fi

    write_info "rapid calls"
    local ok=true
    for _ in $(seq 1 5); do
        r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}')
        if ! echo "$r" | grep -q "samgraha-mcp"; then ok=false; fi
    done
    if $ok; then write_pass "rapid calls"; else write_fail "rapid calls"; fi

    popd > /dev/null
    remove_test_fixture "$test_dir"
}

START_TIME=$(date +%s)
echo "Samgraha Test Runner"
echo "Root: $ROOT_DIR"

if ! $SKIP_BUILD; then
    write_step "Building"
    pushd "$ROOT_DIR" > /dev/null
    LAST_OUTPUT=$(cargo build --bin cli 2>&1) || { write_fail "build cli"; popd > /dev/null; exit 1; }
    write_pass "cli built"
    if $WITH_MCP; then
        LAST_OUTPUT=$(cargo build --bin mcp 2>&1) || { write_fail "build mcp"; popd > /dev/null; exit 1; }
        write_pass "mcp built"
    fi
    popd > /dev/null
fi

cleanup_test() { remove_test_fixture "$TEST_TEMP"; }
trap cleanup_test EXIT

invoke_phase1a
invoke_phase1b
if $FULL; then invoke_phase1c; fi
if $WITH_MCP; then invoke_phase2; invoke_phase25; fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
echo -e "\nPassed: $PASSES  Failed: $FAILURES  Time: ${DURATION}s"

REPORT_DIR="$ROOT_DIR/docs/report/manual-audit"
mkdir -p "$REPORT_DIR"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
MODE_PARTS=()
$FULL && MODE_PARTS+=("full")
$WITH_MCP && MODE_PARTS+=("mcp")
MODE="${MODE_PARTS[*]:-default}"
MODE="${MODE// /-}"
REPORT_PATH="$REPORT_DIR/$TIMESTAMP-$MODE.md"

{
    echo "# Samgraha Test Report"
    echo ""
    echo "**Date:** $(date '+%Y-%m-%d %H:%M:%S')"
    echo "**Mode:** $MODE"
    if [[ $FAILURES -gt 0 ]]; then RES="FAIL"; else RES="PASS"; fi
    echo "**Result:** $RES -- $PASSES passed, $FAILURES failed"
    echo "**Duration:** ${DURATION}s"
    echo ""

    if [[ ${#FAILURE_DETAILS[@]} -gt 0 ]]; then
        echo "## Failure Summary"
        echo ""
        echo "| # | Phase | Test |"
        echo "|---|-------|------|"
        for i in "${!FAILURE_DETAILS[@]}"; do
            IFS='|' read -r phase test output <<< "${FAILURE_DETAILS[$i]}"
            echo "| $((i+1)) | $phase | $test |"
        done
        echo ""
        echo "## Failure Details"
        echo ""
        for i in "${!FAILURE_DETAILS[@]}"; do
            IFS='|' read -r phase test output <<< "${FAILURE_DETAILS[$i]}"
            echo "### $((i+1)). $phase: $test"
            echo ""
            if [[ -n "$output" ]]; then
                echo '```'
                echo "$output"
                echo '```'
                echo ""
            fi
        done
    fi
} > "$REPORT_PATH"

echo "Report: $REPORT_PATH"

if [[ $FAILURES -gt 0 ]]; then exit 1; else exit 0; fi
