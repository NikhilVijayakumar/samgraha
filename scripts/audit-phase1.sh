#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKUP_PATH="$ROOT_DIR/samgraha.toml.phase1bak"
REPORT_DIR="docs/report/manual-audit"

source "$ROOT_DIR/scripts/lib/report.sh"

usage() {
    echo "Usage: $0 [--keep] [--restore] [--report-dir <path>]" >&2
    exit 1
}

KEEP=false
RESTORE=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --keep)       KEEP=true; shift ;;
        --restore)    RESTORE=true; shift ;;
        --report-dir) REPORT_DIR="$2"; shift 2 ;;
        *)            usage ;;
    esac
done

cd "$ROOT_DIR"

report_dir_setup "audit"
load_previous_metrics "$ARCHIVE_DIR" || true

PHASE_ID="01-phase1"
PHASE_START=$(date +%s)
PHASE_CHECKS='[]'
PHASE_ERRORS_JSON='{}'

if $RESTORE; then
    if [[ ! -f "$BACKUP_PATH" ]]; then
        echo "No backup found at $BACKUP_PATH" >&2
        exit 1
    fi
    mv "$BACKUP_PATH" "samgraha.toml"
    echo "Config restored from backup"
    PHASE_CHECKS=$(echo "$PHASE_CHECKS" | jq '. += [{"Name": "Config restored", "Status": "pass", "Detail": ""}]')
    PHASE_CHECKS=$(echo "$PHASE_CHECKS" | jq '. += [{"Name": "Phase 1 audit completed", "Status": "pass", "Detail": ""}]')
    PSTATUS="✅ PASS"
    PSCORE=100
else
    if [[ -f "$BACKUP_PATH" ]]; then
        echo "WARN stale backup found -- restoring first" >&2
        mv "$BACKUP_PATH" "samgraha.toml"
    fi
    cp "samgraha.toml" "$BACKUP_PATH"
    echo "Config backed up -> samgraha.toml.phase1bak"
    PHASE_CHECKS=$(echo "$PHASE_CHECKS" | jq '. += [{"Name": "Config backed up", "Status": "pass", "Detail": ""}]')
    PHASE_CHECKS=$(echo "$PHASE_CHECKS" | jq '. += [{"Name": "Phase 1 audit started", "Status": "pass", "Detail": ""}]')
    PSTATUS="⬜ STARTED"
    PSCORE=0
    echo "Run Phase 1 commands, then: $0 --restore"
fi

PHASE_END=$(date +%s)
PDURATION=$((PHASE_END - PHASE_START))

TOTAL=$(echo "$PHASE_CHECKS" | jq 'length')
OK=$(echo "$PHASE_CHECKS" | jq '[.[] | select(.Status == "pass")] | length')
FAIL=$(echo "$PHASE_CHECKS" | jq '[.[] | select(.Status == "fail")] | length')

CHECKS_TABLE=$(get_checks_table "$PHASE_CHECKS")
ERRORS_TABLE=$(get_errors_table "01-phase1")
ANALYSIS=$(gen_phase_analysis "01-phase1" "$PHASE_CHECKS")
RECS=$(gen_phase_recs "01-phase1" "$PHASE_CHECKS")
PREV_SCORE=$(get_prev_metric ".phase_scores[] | select(.phase == \"01-phase1\") | .score // \"\"")
TREND=$(trend_between "$PSCORE" "$PREV_SCORE")

report_vals=$(jq -n \
    --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
    --arg duration "${PDURATION}s" \
    --arg status "$PSTATUS" \
    --argjson score "$PSCORE" \
    --arg trend "$TREND" \
    --arg prev_score "${PREV_SCORE:-—}" \
    --arg analysis "$ANALYSIS" \
    --arg recommendations "$RECS" \
    --arg checks_table "$CHECKS_TABLE" \
    --arg errors_table "$ERRORS_TABLE" \
    --argjson passes "$OK" \
    --argjson failures "$FAIL" \
    '{TIMESTAMP: $ts, DURATION: $duration, STATUS: $status, SCORE: $score, TREND: $trend, PREV_SCORE: $prev_score, ANALYSIS: $analysis, RECOMMENDATIONS: $recommendations, CHECKS_TABLE: $checks_table, ERRORS_TABLE: $errors_table, PASSES: $passes, FAILURES: $failures}')
write_report "01-phase1.md" "01-phase1.md" "$report_vals" > /dev/null

if $RESTORE; then
    arr=$(jq -n --arg key "01-phase1" --argjson score $PSCORE --arg status "$PSTATUS" --argjson errors $FAIL --argjson dur $PDURATION '[{phase: $key, score: $score, status: $status, errors: $errors, duration: $dur}]')
    metrics=$(jq -n \
        --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
        --argjson ps "$arr" \
        --argjson ts_score "$PSCORE" \
        --argjson passes "$OK" \
        --argjson failures "$FAIL" \
        --argjson duration "$PDURATION" \
        '{timestamp: $ts, phase_scores: $ps, total_score: $ts_score, metrics: {passes: $passes, failures: $failures, duration: $duration}}')
    printf '%s' "$metrics" > "$(metrics_json_path "$LATEST_DIR")"

    phase_row="| 01-phase1 | ${PSCORE}/100 | $PSTATUS | $FAIL | ${PDURATION}s |"
    summary_vals=$(jq -n \
        --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg status "$PSTATUS" \
        --argjson duration "$PDURATION" \
        --argjson score "$PSCORE" \
        --arg trend "$TREND" \
        --arg prev_score "${PREV_SCORE:-—}" \
        --arg analysis "$ANALYSIS" \
        --arg recommendations "$RECS" \
        --arg phase_rows "$phase_row" \
        --argjson passes "$OK" \
        --argjson failures "$FAIL" \
        '{TIMESTAMP: $ts, STATUS: $status, DURATION: $duration, SCORE: $score, TREND: $trend, PREV_SCORE: $prev_score, ANALYSIS: $analysis, RECOMMENDATIONS: $recommendations, PHASE_RESULTS_ROWS: $phase_rows, FAILED_PHASES: "—", PASSES: $passes, FAILURES: $failures}')
    write_report "00-summary.md" "00-summary.md" "$summary_vals" > /dev/null

    echo ""
    echo "Report files:" >&2
    for f in "$LATEST_DIR"/*.md; do
        echo "  $f" >&2
    done
fi

if $KEEP && [ -f "$BACKUP_PATH" ]; then
    rm "$BACKUP_PATH"
fi
