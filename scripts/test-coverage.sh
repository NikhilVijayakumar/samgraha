#!/usr/bin/env bash
# Runs unit + e2e tests with coverage and writes a TestRunReport JSON
# (see crates/schemas/src/test_run.rs, docs/raw/audit/test-report.schema.json)
# for Coverage Audit (CV6) / Implementation Audit (I8) to read via the
# repo's [pipelines.test] contract.
#
# Single instrumented `cargo llvm-cov --workspace --json` run covers both lib
# unit tests and the `tests` crate's e2e_*.rs integration tests in one pass;
# stderr carries the normal cargo-test log (parsed for pass/fail), stdout
# carries the coverage JSON (parsed for line coverage %).
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_PATH="${1:-docs/report/test-results.json}"
STDOUT_FILE=$(mktemp "${TMPDIR:-/tmp}/samgraha-llvmcov-stdout-XXXXXX.json")
STDERR_FILE=$(mktemp "${TMPDIR:-/tmp}/samgraha-llvmcov-stderr-XXXXXX.log")
trap 'rm -f "$STDOUT_FILE" "$STDERR_FILE"' EXIT

cd "$ROOT_DIR"
echo "Running unit+e2e tests under cargo-llvm-cov..."
cargo llvm-cov --workspace --json 1>"$STDOUT_FILE" 2>"$STDERR_FILE"
cat "$STDERR_FILE"

unit_passed=0; unit_failed=0; unit_skipped=0
e2e_passed=0; e2e_failed=0; e2e_skipped=0
unit_failures="[]"
e2e_failures="[]"
bucket=""

while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*Running\ unittests ]]; then
        bucket="unit"; continue
    fi
    if [[ "$line" =~ ^[[:space:]]*Running\ tests[\\/] ]]; then
        bucket="e2e"; continue
    fi
    [[ -z "$bucket" ]] && continue

    if [[ "$line" =~ ^test\ (\S+)\ \.\.\.\ FAILED ]]; then
        # ponytail: name only, no scraped panic message — see test-coverage.ps1's
        # matching comment for why (message stays "" in the JSON, schema default).
        name="${BASH_REMATCH[1]}"
        entry=$(jq -n --arg n "$name" '{name: $n, message: ""}')
        if [[ "$bucket" == "unit" ]]; then
            unit_failures=$(echo "$unit_failures" | jq --argjson e "$entry" '. + [$e]')
        else
            e2e_failures=$(echo "$e2e_failures" | jq --argjson e "$entry" '. + [$e]')
        fi
        continue
    fi

    if [[ "$line" =~ test\ result:\ (ok|FAILED)\.\ ([0-9]+)\ passed\;\ ([0-9]+)\ failed\;\ ([0-9]+)\ ignored\; ]]; then
        p="${BASH_REMATCH[2]}"; f="${BASH_REMATCH[3]}"; s="${BASH_REMATCH[4]}"
        if [[ "$bucket" == "unit" ]]; then
            unit_passed=$((unit_passed + p)); unit_failed=$((unit_failed + f)); unit_skipped=$((unit_skipped + s))
        else
            e2e_passed=$((e2e_passed + p)); e2e_failed=$((e2e_failed + f)); e2e_skipped=$((e2e_skipped + s))
        fi
    fi
done < "$STDERR_FILE"

unit_total=$((unit_passed + unit_failed + unit_skipped))
e2e_total=$((e2e_passed + e2e_failed + e2e_skipped))

coverage_percent=$(jq -r '.data[0].totals.lines.percent // empty' "$STDOUT_FILE" 2>/dev/null)
if [[ -n "$coverage_percent" ]]; then
    coverage_percent=$(printf '%.1f' "$coverage_percent")
else
    coverage_percent="null"
    echo "Warning: could not parse coverage JSON — coverage_percent will be null"
fi

full_report_path="$ROOT_DIR/$REPORT_PATH"
mkdir -p "$(dirname "$full_report_path")"
jq -n \
    --argjson unit_total "$unit_total" --argjson unit_passed "$unit_passed" --argjson unit_failed "$unit_failed" --argjson unit_skipped "$unit_skipped" --argjson unit_failures "$unit_failures" \
    --argjson e2e_total "$e2e_total" --argjson e2e_passed "$e2e_passed" --argjson e2e_failed "$e2e_failed" --argjson e2e_skipped "$e2e_skipped" --argjson e2e_failures "$e2e_failures" \
    --argjson coverage_percent "$coverage_percent" \
    '{
        unit: { total: $unit_total, passed: $unit_passed, failed: $unit_failed, skipped: $unit_skipped, failures: $unit_failures },
        e2e:  { total: $e2e_total,  passed: $e2e_passed,  failed: $e2e_failed,  skipped: $e2e_skipped,  failures: $e2e_failures },
        coverage_percent: $coverage_percent
    }' > "$full_report_path"

echo "Wrote $full_report_path"
echo "unit: $unit_passed/$unit_total passed, e2e: $e2e_passed/$e2e_total passed, coverage: ${coverage_percent}%"

if [[ "$unit_failed" -gt 0 || "$e2e_failed" -gt 0 ]]; then
    exit 1
fi
exit 0
