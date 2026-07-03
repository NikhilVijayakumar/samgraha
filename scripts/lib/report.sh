# ─── Shared Report Utilities (sourced by mcp-discover.sh, run-tests.sh, audit-phase1.sh) ───

metrics_json_path() {
    echo "$1/metrics.json"
}

esc_md() {
    echo "$1" | sed 's/|/\\|/g'
}

write_report() {
    local template_name="$1" output_name="$2" values_json="$3"
    local template_path="$TEMPLATE_DIR/$template_name"
    local output_path="$LATEST_DIR/$output_name"
    local content
    if [ -f "$template_path" ]; then
        content=$(<"$template_path")
    else
        echo "WARNING: Template missing: $template_path -- using inline fallback" >&2
        content=$(printf '# %s\n\n**Status:** {{STATUS}}\n\n{{ERRORS_TABLE}}\n\n{{CHECKS_TABLE}}' "$output_name")
    fi
    local tmp_tpl tmp_vals
    tmp_tpl=$(mktemp)
    tmp_vals=$(mktemp)
    printf '%s' "$content" > "$tmp_tpl"
    printf '%s' "$values_json" > "$tmp_vals"
    python3 -c "
import sys, json
with open('$tmp_tpl') as f: tpl = f.read()
with open('$tmp_vals') as f: vals = json.load(f)
for k, v in vals.items():
    tpl = tpl.replace('{{' + k + '}}', str(v))
with open('$output_path', 'w') as f: f.write(tpl)
"
    rm -f "$tmp_tpl" "$tmp_vals"
    echo "$output_path"
}

add_phase_error() {
    local tool="$1" err="$2" resp="$3"
    local snippet="${resp:0:250}"
    if [ ${#resp} -gt 250 ]; then snippet+="..."; fi
    PHASE_ERRORS_JSON=$(echo "$PHASE_ERRORS_JSON" | jq -c \
        --arg phase "$CURRENT_PHASE" \
        --arg tool "$tool" \
        --arg err "$err" \
        --arg resp "$snippet" \
        '.[$phase] += [{"Tool": $tool, "Error": $err, "Response": $resp}]')
}

get_phase_errors_json() {
    local phase="$1"
    echo "$PHASE_ERRORS_JSON" | jq -c ".[\"$phase\"] // []"
}

get_errors_table() {
    local phase="$1"
    local errs
    errs=$(get_phase_errors_json "$phase")
    local count
    count=$(echo "$errs" | jq 'length')
    if [ "$count" -eq 0 ]; then
        echo "✅ No errors"
        return
    fi
    echo "$errs" | jq -r '[
        "| Tool Call | Error | Response |",
        "|-----------|-------|----------|"
    ] + [.[] | "| " + (.Tool | gsub("\\|"; "\\|")) + " | " +
        (.Error | gsub("\\|"; "\\|")) + " | " +
        (.Response[0:120] | gsub("\\|"; "\\|")) + " |"
    ] | join("\n")'
}

get_checks_table() {
    local checks_json="$1"
    local count
    count=$(echo "$checks_json" | jq 'length')
    if [ "$count" -eq 0 ]; then
        echo "| -- | -- | -- | -- |"
        return
    fi
    echo "$checks_json" | jq -r '[
        "| # | Check | Status | Detail |",
        "|---|-------|--------|--------|"
    ] + (to_entries | map(
        "| " + (.key + 1 | tostring) + " | " +
        (.value.Name | gsub("\\|"; "\\|")) + " | " +
        (if .value.Status == "pass" then "✅ " else
         if .value.Status == "fail" then "❌ " else
         if .value.Status == "warn" then "⚠️ " else "⬜ " end end end) +
        (.value.Status) + " | " +
        (.value.Detail[0:80] | gsub("\\|"; "\\|")) + " |"
    )) | join("\n")'
}

load_previous_metrics() {
    local archive_dir="$1"
    [ ! -d "$archive_dir" ] && return 1
    local newest=""
    while IFS= read -r d; do
        [ -d "$archive_dir/$d" ] && newest="$d"
    done < <(ls -1 "$archive_dir" 2>/dev/null | sort -r | head -1)
    [ -z "$newest" ] && return 1
    local mpath="$(metrics_json_path "$archive_dir/$newest")"
    [ ! -f "$mpath" ] && return 1
    PREV_METRICS=$(<"$mpath")
    echo "  Loaded previous metrics from $archive_dir/$newest" >&2
}

get_prev_metric() {
    local query="$1"
    local phase metric
    phase=$(echo "$query" | sed -n 's/.*select(.phase == "\([^"]*\)").*/\1/p')
    metric=$(echo "$query" | sed -n 's/.*| \([^ ]*\) \/\/.*/\1/p' | tr -d '.')
    [ -z "$metric" ] && metric="score"
    echo "${PREV_METRICS:-{\}}" | jq -r --arg p "$phase" --arg m "$metric" \
      '(.phase_scores // []) | .[] | select(.phase == $p) | .[$m] // ""' 2>/dev/null || echo ""
}

trend_between() {
    local c="$1" p="$2"
    [ -z "$p" ] && { echo "—"; return; }
    if [ "$(echo "$c > $p" | bc 2>/dev/null || echo 0)" -eq 1 ]; then echo "↑"
    elif [ "$(echo "$c < $p" | bc 2>/dev/null || echo 0)" -eq 1 ]; then echo "↓"
    else echo "→"; fi
}

compute_trend() {
    local current="$1" previous="$2"
    if [ -z "$previous" ] || [ "$previous" = "null" ]; then echo "—"; return; fi
    if [ "$(echo "$current > $previous" | bc 2>/dev/null)" -eq 1 ]; then echo "↑"
    elif [ "$(echo "$current < $previous" | bc 2>/dev/null)" -eq 1 ]; then echo "↓"
    else echo "→"; fi
}

format_score_line() {
    local label="$1" score="$2" prev="$3" status="$4" errors="$5"
    local trend
    trend=$(trend_between "$score" "$prev")
    printf "| %s | %s/100 | %s/100 | %s | %s | %s |\n" "$label" "$score" "${prev:-—}" "$trend" "$status" "$errors"
}

gen_phase_analysis() {
    local phase="$1" checks="$2"
    [ -z "$checks" ] || [ "$checks" = "null" ] && { echo "No checks data available."; return; }
    local total ok fail warn skip
    total=$(echo "$checks" | jq 'length // 0')
    ok=$(echo "$checks" | jq '[.[] | select(.Status == "pass")] | length')
    fail=$(echo "$checks" | jq '[.[] | select(.Status == "fail")] | length')
    warn=$(echo "$checks" | jq '[.[] | select(.Status == "warn")] | length')
    skip=$(echo "$checks" | jq '[.[] | select(.Status == "skip")] | length')
    if [ "$total" -eq 0 ]; then echo "No checks performed for this phase."; return; fi
    local msg=""
    if [ "$fail" -gt 0 ]; then msg+="❌ $fail/$total checks failed. "
    elif [ "$warn" -gt 0 ]; then msg+="⚠️ $warn/$total checks have warnings. "
    elif [ "$skip" -gt 0 ]; then msg+="ℹ️ $skip/$total checks skipped, $ok passed. "
    else msg+="✅ All $total checks passed. "
    fi
    msg+="$ok passed, $warn warnings, $fail failures."
    [ "$skip" -gt 0 ] && msg+=" $skip skipped."
    echo "$msg"
}

gen_phase_recs() {
    local phase="$1" checks="$2"
    [ -z "$checks" ] || [ "$checks" = "null" ] && { echo "- No data to generate recommendations."; return; }
    local recs=""
    local fail_items
    fail_items=$(echo "$checks" | jq -r '[.[] | select(.Status == "fail") | .Name] | join("│")')
    local warn_items
    warn_items=$(echo "$checks" | jq -r '[.[] | select(.Status == "warn") | .Name] | join("│")')
    if [ -n "$fail_items" ]; then
        local IFS='│'
        for f in $fail_items; do
            recs+="- 🔴 Fix failing check: $f"$'\n'
        done
        unset IFS
    fi
    if [ -n "$warn_items" ]; then
        local IFS='│'
        for w in $warn_items; do
            recs+="- 🟡 Address warning: $w"$'\n'
        done
        unset IFS
    fi
    [ -z "$recs" ] && recs="- ✅ No action required"
    echo "$recs"
}

report_dir_setup() {
    local report_subdir="$1"
    TEMPLATE_DIR="$ROOT_DIR/scripts/templates/$report_subdir"
    LATEST_DIR="$ROOT_DIR/$REPORT_DIR/$report_subdir/latest"
    ARCHIVE_DIR="$ROOT_DIR/$REPORT_DIR/$report_subdir/archive"
    mkdir -p "$TEMPLATE_DIR" "$ARCHIVE_DIR"
    if [ -d "$LATEST_DIR" ]; then
        local ts
        ts=$(date '+%Y-%m-%d_%H%M%S')
        ARCHIVE_PATH="$ARCHIVE_DIR/$ts"
        mv "$LATEST_DIR" "$ARCHIVE_PATH"
        echo "Archived previous run → $ARCHIVE_PATH"
    fi
    mkdir -p "$LATEST_DIR"
}
