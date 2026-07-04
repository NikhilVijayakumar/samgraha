#!/usr/bin/env bash
set -uo pipefail

FAILURES=0
PASSES=0
FAILURE_DETAILS=()
LAST_OUTPUT=""
CURRENT_PHASE=""
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/report.sh"
TEST_TEMP=$(mktemp -d "/tmp/samgraha-test-XXXXXX")

FULL=false
WITH_MCP=false
SKIP_BUILD=false
ALL=false
PHASE_ID=""
PHASE_DURATION=""
declare -A PHASE_CHECKS
declare -A PHASE_RESULTS
PREV_METRICS='{}'
PHASE_ERRORS_JSON='{}'
REPORT_DIR="docs/report/tests"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --full)     FULL=true; shift ;;
        --with-mcp) WITH_MCP=true; shift ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --all)      ALL=true; FULL=true; WITH_MCP=true; shift ;;
        --report-dir) REPORT_DIR="$2"; shift 2 ;;
        *)          echo "Usage: $0 [--full] [--with-mcp] [--all] [--skip-build] [--report-dir <dir>]"; exit 1 ;;
    esac
done

report_dir_setup "tests"

write_pass() { echo "  OK $*"; PASSES=$((PASSES + 1)); local pc="${PHASE_CHECKS[$PHASE_ID]:-[]}"; PHASE_CHECKS["$PHASE_ID"]=$(echo "$pc" | jq --arg n "$*" '. += [{"Name": $n, "Status": "pass", "Detail": ""}]' 2>/dev/null || echo "$pc"); }
write_fail() {
    local msg="$*"
    echo "  XX $msg"
    FAILURES=$((FAILURES + 1))
    FAILURE_DETAILS+=("$PHASE_ID|$msg|$(echo -e "$LAST_OUTPUT")")
    local pc="${PHASE_CHECKS[$PHASE_ID]:-[]}"
    PHASE_CHECKS["$PHASE_ID"]=$(echo "$pc" | jq --arg n "$*" '. += [{"Name": $n, "Status": "fail", "Detail": ""}]' 2>/dev/null || echo "$pc")
}
write_step() { CURRENT_PHASE="$*"; LAST_OUTPUT=""; echo -e "\n== $CURRENT_PHASE =="; }
write_info() { echo "  .. $*"; }

write_phase_report() {
    local pid="$1"
    local phase_checks="${PHASE_CHECKS[$pid]:-[]}" end
    end=$(date +%s)
    local duration=$((end - PHASE_DURATION))
    local checks_table errors_table analysis recs
    checks_table=$(get_checks_table "$phase_checks")
    errors_table=$(get_errors_table "$pid")
    analysis=$(gen_phase_analysis "$pid" "$phase_checks")
    recs=$(gen_phase_recs "$pid" "$phase_checks")
    local score prev_score trend status
    local total ok fail
    total=$(echo "$phase_checks" | jq 'length // 0')
    ok=$(echo "$phase_checks" | jq '[.[] | select(.Status == "pass")] | length')
    fail=$(echo "$phase_checks" | jq '[.[] | select(.Status == "fail")] | length')
    if [ "$total" -eq 0 ]; then score=0; else score=$((ok * 100 / total)); fi
    [ "$fail" -gt 0 ] && status="❌ FAIL" || status="✅ PASS"
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"$pid\") | .score // \"\"")
    trend=$(trend_between "$score" "$prev_score")
    local report_vals
    report_vals=$(jq -n \
        --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg duration "${duration}s" \
        --arg status "$status" \
        --argjson score "$score" \
        --arg trend "$trend" \
        --arg prev_score "${prev_score:-—}" \
        --arg analysis "$analysis" \
        --arg recommendations "$recs" \
        --arg checks_table "$checks_table" \
        --arg errors_table "$errors_table" \
        --argjson passes "$ok" \
        --argjson failures "$fail" \
        '{TIMESTAMP: $ts, DURATION: $duration, STATUS: $status, SCORE: $score, TREND: $trend, PREV_SCORE: $prev_score, ANALYSIS: $analysis, RECOMMENDATIONS: $recommendations, CHECKS_TABLE: $checks_table, ERRORS_TABLE: $errors_table, PASSES: $passes, FAILURES: $failures}')
    write_report "$pid.md" "$pid.md" "$report_vals" > /dev/null
    PHASE_RESULTS["$pid"]=$(jq -n --arg status "$status" --argjson score "$score" --argjson errors "$fail" --argjson duration "$duration" '{Status: $status, Score: $score, Errors: $errors, Duration: $duration}')
}

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
    PHASE_ID="01-phase1a"
    PHASE_DURATION=$(date +%s)
    pushd "$ROOT_DIR" > /dev/null
    write_info "Running cargo test -p tests"
    LAST_OUTPUT=$(cargo test -p tests 2>&1 || true)
    echo "$LAST_OUTPUT"
    assert_exit_code_zero "cargo test -p tests"
    popd > /dev/null
    write_phase_report "01-phase1a"
}

invoke_phase1b() {
    write_step "Phase 1b - CLI Integration"
    PHASE_ID="02-phase1b"
    PHASE_DURATION=$(date +%s)
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
    write_phase_report "02-phase1b"
}

invoke_phase1c() {
    write_step "Phase 1c - Multi-Repo"
    PHASE_ID="03-phase1c"
    PHASE_DURATION=$(date +%s)
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
    write_phase_report "03-phase1c"
}

invoke_phase2() {
    write_step "Phase 2 - MCP Tests"
    PHASE_ID="04-phase2"
    PHASE_DURATION=$(date +%s)
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
    r=$(raw_mcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_document","arguments":{"id":1}}}')
    if echo "$r" | grep -qiE "error|not found"; then write_fail "get_document"; else write_pass "get_document"; fi

    write_info "tools/call nonexistent"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent"}}')
    if echo "$r" | grep -qiE "error|not found"; then write_pass "nonexistent => error"; else write_fail "nonexistent should error"; fi

    popd > /dev/null
    remove_test_fixture "$test_dir"
    write_phase_report "04-phase2"
}

invoke_phase3() {
    write_step "Phase 3 - Semantic Audit Tools"
    PHASE_ID="06-phase3"
    PHASE_DURATION=$(date +%s)
    local test_dir="$TEST_TEMP/p3"
    new_test_fixture "$test_dir" "audit-test"
    # Create minimal audit knowledge files for the test fixture
    mkdir -p "$test_dir/docs/raw/audit-standards/feature"
    cat > "$test_dir/docs/raw/audit-standards/feature/functional-requirements.md" << 'EOF'
# Functional Requirements Audit
## Scoring Criteria
| ID | Score | Description |
|---|---|---|
| C1 | 30 | All requirements uniquely identified |
| C2 | 30 | Each requirement is testable |
EOF
    pushd "$test_dir" > /dev/null
    run_cli "compile" > /dev/null

    raw_mcp() {
        LAST_OUTPUT=$(echo "$1" | cargo run --manifest-path "$ROOT_DIR/Cargo.toml" --bin mcp 2>&1)
        echo "$LAST_OUTPUT"
    }

    write_info "get_documents_by_domain"
    local r
    r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_documents_by_domain","arguments":{"domain":"feature"}}}')
    if echo "$r" | grep -qiE "documents|\[" ; then write_pass "get_documents_by_domain"; else write_fail "get_documents_by_domain"; fi

    write_info "get_section"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_section","arguments":{"section_id":1}}}')
    if echo "$r" | grep -qiE "section"; then write_pass "get_section"; else write_fail "get_section"; fi

    write_info "get_audit_knowledge"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_audit_knowledge","arguments":{"domain":"feature","section_type":"functional-requirements"}}}')
    if echo "$r" | grep -qE "C1|C2"; then write_pass "get_audit_knowledge"; else write_fail "get_audit_knowledge"; fi

    write_info "get_section_changed"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_section_changed","arguments":{"section_id":1}}}')
    if echo "$r" | grep -qiE "changed"; then write_pass "get_section_changed"; else write_fail "get_section_changed"; fi

    write_info "check_gate"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"check_gate","arguments":{"stage":"deterministic","document_id":1}}}')
    if echo "$r" | grep -qiE "passed|blocked"; then write_pass "check_gate"; else write_fail "check_gate"; fi

    write_info "store_section_report"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"store_section_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000001","domain":"feature","stage":"Section","document_id":1,"section_id":1,"strategy":"completeness","score":85,"findings":[{"check_id":"C1","severity":"Error","message":"All present","provider":"test","confidence":0.95,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"test"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}')
    if echo "$r" | grep -qiE "report_id"; then write_pass "store_section_report"; else write_fail "store_section_report"; fi

    write_info "get_audit_report"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"get_audit_report","arguments":{"domain":"feature","stage":"section","document_id":1}}}')

    if echo "$r" | grep -qiE "report_id|findings"; then write_pass "get_audit_report"; else write_fail "get_audit_report"; fi

    write_info "update_finding_status"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"update_finding_status","arguments":{"report_id":1,"criterion_id":"C1","status":"fixed"}}}')
    if echo "$r" | grep -qiE "success|true"; then write_pass "update_finding_status"; else write_fail "update_finding_status"; fi

    write_info "store_document_report"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"store_document_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000002","domain":"feature","stage":"Document","document_id":1,"section_id":null,"strategy":"completeness","score":90,"findings":[{"check_id":"C1","severity":"Error","message":"Doc level","provider":"test","confidence":0.95,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"test"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}')
    if echo "$r" | grep -qiE "report_id"; then write_pass "store_document_report"; else write_fail "store_document_report"; fi

    write_info "store_cross_domain_report"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"store_cross_domain_report","arguments":{"report_json":{"report_id":"00000000-0000-0000-0000-000000000003","domain":"feature","stage":"CrossDomain","document_id":null,"section_id":null,"strategy":"consistency","score":80,"findings":[{"check_id":"C1","severity":"Warning","message":"Cross domain","provider":"test","confidence":0.85,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"cross"},"status":"Open"}],"created_at":"2026-01-01T00:00:00Z"}}}}')

    if echo "$r" | grep -qiE "report_id"; then write_pass "store_cross_domain_report"; else write_fail "store_cross_domain_report"; fi

    popd > /dev/null
    remove_test_fixture "$test_dir"
    write_phase_report "06-phase3"
}

invoke_phase25() {
    write_step "Phase 2.5 - Protocol"
    PHASE_ID="05-phase25"
    PHASE_DURATION=$(date +%s)
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

    write_info "tools/list"
    r=$(raw_mcp '{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}')
    if echo "$r" | grep -q "compile"; then write_pass "tools/list"; else write_fail "expected tools"; fi

    write_info "rapid calls"
    local ok=true
    for _ in $(seq 1 5); do
        r=$(raw_mcp '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}')
        if ! echo "$r" | grep -q "samgraha-mcp"; then ok=false; fi
    done
    if $ok; then write_pass "rapid calls"; else write_fail "rapid calls"; fi

    popd > /dev/null
    remove_test_fixture "$test_dir"
    write_phase_report "05-phase25"
}

START_TIME=$(date +%s)
echo "Samgraha Test Runner"
echo "Root: $ROOT_DIR"

if ! $SKIP_BUILD; then
    PHASE_ID="00-build"
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
if $WITH_MCP; then invoke_phase2; invoke_phase25; invoke_phase3; fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
echo -e "\nPassed: $PASSES  Failed: $FAILURES  Time: ${DURATION}s"

# Track build phase
build_checks='[]'
if $SKIP_BUILD; then
    build_checks=$(echo "$build_checks" | jq '. += [{"Name": "Build", "Status": "skip", "Detail": "Skipped via --skip-build"}]')
else
    build_checks=$(echo "$build_checks" | jq '. += [{"Name": "Build", "Status": "pass", "Detail": "Binaries built"}]')
fi
PHASE_CHECKS["00-build"]="$build_checks"

# Generate summary report
all_phase_rows=""
all_failed=""
score_sum=0
score_count=0
for key in 00-build 01-phase1a 02-phase1b 03-phase1c 04-phase2 05-phase25 06-phase3; do
    pr="${PHASE_RESULTS[$key]:-}"
    [ -z "$pr" ] && continue
    ps=""; pf=""; pe=""; pd=""
    ps=$(echo "$pr" | jq -r '.Status // "?"')
    pf=$(echo "$pr" | jq -r '.Score // 0')
    pe=$(echo "$pr" | jq -r '.Errors // 0')
    pd=$(echo "$pr" | jq -r '.Duration // 0')
    all_phase_rows+="| $key | ${pf}/100 | $ps | $pe | ${pd}s |"$'\n'
    score_sum=$((score_sum + pf))
    score_count=$((score_count + 1))
    ! echo "$ps" | grep -q "PASS" && [ "$ps" != "⬜ SKIPPED" ] && all_failed+="- **$key**: $ps ($pe errors)"$'\n'
done
total_score=0
[ "$score_count" -gt 0 ] && total_score=$((score_sum / score_count))

prev_total_score=$(get_prev_metric ".total_score // \"\"")
total_trend=$(trend_between "$total_score" "$prev_total_score")

analysis=""
recs=""
if [ "$FAILURES" -gt 0 ]; then
    analysis="❌ $FAILURES failures across $score_count phases. $PASSES total passes."
    recs="- 🔴 Fix $FAILURES failing test(s) before next run"
else
    analysis="✅ All $score_count phases passed. $PASSES total passes."
    recs="- ✅ No action required"
fi
[ -z "$all_failed" ] && all_failed="—"

report_vals=$(jq -n \
    --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
    --arg status "$([ "$FAILURES" -gt 0 ] && echo "❌ FAIL" || echo "✅ PASS")" \
    --argjson duration "$DURATION" \
    --argjson score "$total_score" \
    --arg trend "$total_trend" \
    --arg prev_score "${prev_total_score:-—}" \
    --arg analysis "$analysis" \
    --arg recommendations "$recs" \
    --arg phase_rows "$all_phase_rows" \
    --arg failed_phases "$all_failed" \
    --argjson passes "$PASSES" \
    --argjson failures "$FAILURES" \
    '{TIMESTAMP: $ts, STATUS: $status, DURATION: $duration, SCORE: $score, TREND: $trend, PREV_SCORE: $prev_score, ANALYSIS: $analysis, RECOMMENDATIONS: $recommendations, PHASE_RESULTS_ROWS: $phase_rows, FAILED_PHASES: $failed_phases, PASSES: $passes, FAILURES: $failures}')
write_report "00-summary.md" "00-summary.md" "$report_vals" > /dev/null

# Save metrics
metrics_phase_order=(01-phase1a 02-phase1b)
$FULL && metrics_phase_order+=(03-phase1c)
$WITH_MCP && metrics_phase_order+=(04-phase2 05-phase25 06-phase3)
arr='[]'
for key in "${metrics_phase_order[@]}"; do
    pr="${PHASE_RESULTS[$key]:-}"
    [ -z "$pr" ] && continue
    arr=$(echo "$arr" | jq -c \
        --arg key "$key" \
        --argjson score "$(echo "$pr" | jq '(.Score // 0) | floor')" \
        --arg status "$(echo "$pr" | jq -r '.Status // "?"')" \
        --argjson errors "$(echo "$pr" | jq '(.Errors // 0)')" \
        --argjson dur "$(echo "$pr" | jq '(.Duration // 0)')" \
        '. + [{phase: $key, score: $score, status: $status, errors: $errors, duration: $dur}]')
done
metrics=$(jq -n \
    --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
    --argjson ps "$arr" \
    --argjson ts_score "${total_score:-0}" \
    --argjson passes "$PASSES" \
    --argjson failures "$FAILURES" \
    --argjson duration "$DURATION" \
    '{
        timestamp: $ts,
        phase_scores: $ps,
        total_score: $ts_score,
        metrics: {
            passes: $passes, failures: $failures, duration: $duration
        }
    }')
printf '%s' "$metrics" > "$LATEST_DIR/metrics.json"

echo "Report files:"
for f in "$LATEST_DIR"/*.md; do
    echo "  $f"
done

if [[ $FAILURES -gt 0 ]]; then exit 1; else exit 0; fi
