#!/usr/bin/env bash
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/report.sh"

# ─── Defaults ──────────────────────────────────────────────────────────────────
BUILD=false
BINARY_PATH=""
REPORT_DIR="docs/report/manual-audit"
DOMAINS=()
MAX_DOCS=0
MAX_SECTIONS=0
NO_SECTION_CONTENT=false
NO_AUDIT=false
PASS_THRU=false

# ─── Argument Parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --build) BUILD=true; shift ;;
        --binary-path) BINARY_PATH="$2"; shift 2 ;;
        --report-dir) REPORT_DIR="$2"; shift 2 ;;
        --domain) DOMAINS+=("$2"); shift 2 ;;
        --max-docs) MAX_DOCS="$2"; shift 2 ;;
        --max-sections) MAX_SECTIONS="$2"; shift 2 ;;
        --no-section-content) NO_SECTION_CONTENT=true; shift ;;
        --no-audit) NO_AUDIT=true; shift ;;
        --pass-thru) PASS_THRU=true; shift ;;
        *)
            echo "Usage: $0 [--build] [--binary-path <path>] [--report-dir <dir>]" >&2
            echo "          [--domain <d>]... [--max-docs <n>] [--max-sections <n>]" >&2
            echo "          [--no-section-content] [--no-audit] [--pass-thru]" >&2
            exit 1 ;;
    esac
done

# ─── Scoring Config ─────────────────────────────────────────────────────────────
SCORE_WEIGHTS=(10 5 15 15 5 20 10 20)  # phases 1-8, sum=100

# ─── Requirement Checks ────────────────────────────────────────────────────────
if ! command -v jq &>/dev/null; then echo "ERROR: jq is required" >&2; exit 1; fi
if ! command -v python3 &>/dev/null; then echo "ERROR: python3 is required" >&2; exit 1; fi

# ─── Binary Resolution ─────────────────────────────────────────────────────────
if $BUILD; then
    echo "Building mcp binary..."
    pushd "$ROOT_DIR" > /dev/null
    cargo build --bin mcp 2>&1
    rc=$?
    popd > /dev/null
    if [ $rc -ne 0 ]; then echo "cargo build failed" >&2; exit 1; fi
    BINARY_PATH="$ROOT_DIR/target/debug/mcp"
elif [ -n "$BINARY_PATH" ]; then
    true
else
    if [ -f "$ROOT_DIR/target/debug/mcp" ]; then
        BINARY_PATH="$ROOT_DIR/target/debug/mcp"
    elif [ -f "$ROOT_DIR/target/release/mcp" ]; then
        BINARY_PATH="$ROOT_DIR/target/release/mcp"
    else
        echo "No mcp binary found. Use --build or --binary-path." >&2
        exit 1
    fi
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found: $BINARY_PATH" >&2
    exit 1
fi
echo "MCP binary: $BINARY_PATH"

# ─── Report Directory Setup ────────────────────────────────────────────────────
report_dir_setup "mcp"
ARCHIVE_PATH="${ARCHIVE_PATH:-}"

# ─── Global State ──────────────────────────────────────────────────────────────
NEXT_ID=1
TOTAL_CALLS=0
CURRENT_PHASE=""
ALL_RESULTS='{"Protocol":null,"Tools":[],"Runtime":null,"Domains":{},"TotalDocs":0,"AllDocs":[],"TotalSections":0,"SectionIds":{},"SectionsByType":{}}'
PHASE_ERRORS_JSON='{}'
declare -A PHASE_RESULTS
PREV_METRICS='{}'
PREV_ERRORS=0

# ─── Core Functions ────────────────────────────────────────────────────────────

get_id() {
    local id=$NEXT_ID
    NEXT_ID=$((NEXT_ID + 1))
    echo "$id"
}

# (add_phase_error, get_phase_errors_json, get_errors_table, get_checks_table, write_report, esc_md sourced from scripts/lib/report.sh)

invoke_mcp_direct() {
    local method="$1" params="${2:-}" id
    [ -z "$params" ] && params='{}'
    id=$(get_id)
    TOTAL_CALLS=$((TOTAL_CALLS + 1))
    local request
    request=$(jq -nc --argjson id "$id" --arg method "$method" --argjson params "$params" '{
        jsonrpc: "2.0", id: $id, method: $method, params: $params
    }')
    local raw
    raw=$(echo "$request" | "$BINARY_PATH" 2>/dev/null)
    if [ -z "$raw" ]; then
        add_phase_error "$request" "Empty response" ""
        return 1
    fi
    local err
    err=$(echo "$raw" | jq -r '.error // empty')
    if [ -n "$err" ] && [ "$err" != "null" ]; then
        local code msg
        code=$(echo "$err" | jq -r '.code')
        msg=$(echo "$err" | jq -r '.message')
        add_phase_error "$request" "$code: $msg" "$raw"
        return 1
    fi
    local result
    result=$(echo "$raw" | jq -c '.result // empty')
    if [ -z "$result" ] || [ "$result" = "null" ]; then
        return 1
    fi
    echo "$result"
}

invoke_mcp_tool() {
    local name="$1" arguments="${2:-}" quiet="${3:-false}" id
    [ -z "$arguments" ] && arguments='{}'
    id=$(get_id)
    TOTAL_CALLS=$((TOTAL_CALLS + 1))
    local request
    request=$(jq -nc --argjson id "$id" --arg name "$name" --argjson arguments "$arguments" '{
        jsonrpc: "2.0", id: $id, method: "tools/call",
        params: {name: $name, arguments: $arguments}
    }')
    local raw
    raw=$(echo "$request" | "$BINARY_PATH" 2>/dev/null)
    if [ -z "$raw" ]; then
        if [ "$quiet" != "true" ]; then
            add_phase_error "$request" "Empty response" ""
        fi
        return 1
    fi
    local err
    err=$(echo "$raw" | jq -r '.error // empty')
    if [ -n "$err" ] && [ "$err" != "null" ]; then
        if [ "$quiet" != "true" ]; then
            local code msg
            code=$(echo "$err" | jq -r '.code')
            msg=$(echo "$err" | jq -r '.message')
            add_phase_error "$request" "$code: $msg" "$raw"
        fi
        return 1
    fi
    local result
    result=$(echo "$raw" | jq -c '.result // empty')
    if [ -z "$result" ] || [ "$result" = "null" ]; then
        return 1
    fi
    echo "$result"
}

invoke_mcp_tool_all() {
    local name="$1" arguments="$2" collection_key="$3" page_size="${4:-100}" quiet="${5:-false}"
    local all='[]' offset=0 has_more=true
    while $has_more; do
        local paginated
        paginated=$(echo "$arguments" | jq -c --argjson limit "$page_size" --argjson offset "$offset" \
            '. + {limit: $limit, offset: $offset}')
        local result
        result=$(invoke_mcp_tool "$name" "$paginated" "$quiet") || break
        [ -z "$result" ] && break
        local items
        items=$(echo "$result" | jq -c ".${collection_key} // []")
        [ "$items" = "null" ] && break
        # Merge via temp file to avoid ARG_MAX on large payloads
        local tmp_merge
        tmp_merge=$(mktemp)
        printf '%s' "$items" > "$tmp_merge"
        all=$(echo "$all" | jq -c --slurpfile more "$tmp_merge" '. + $more[0]')
        rm -f "$tmp_merge"
        has_more=$(echo "$result" | jq -r '.has_more // false')
        offset=$((offset + page_size))
        local count
        count=$(echo "$items" | jq 'length')
        [ "$count" -lt "$page_size" ] && has_more=false
    done
    echo "$all"
}

# ─── Scoring / Trend / Analysis Helpers ────────────────────────────────────────

save_metrics_json() {
    local all_jq
    all_jq=$(echo "${ALL_RESULTS:-null}" | jq -c 'if type == "object" then . else null end' 2>/dev/null || echo "null")
    local ps_json total_score
    ps_json=$(build_phase_scores_json)
    total_score=$(echo "$ps_json" | jq 'if length > 0 then ([.[] | .score] | add / length | floor) else 0 end')
    local metrics
    metrics=$(jq -n \
        --arg ts "$(date '+%Y-%m-%d %H:%M:%S')" \
        --argjson ps "$ps_json" \
        --argjson ts_score "${total_score:-0}" \
        --argjson tc "$(echo "$all_jq" | jq '(.Tools | length) // 0')" \
        --argjson dc "$(echo "$all_jq" | jq '.TotalDocs // 0')" \
        --argjson sc "$(echo "$all_jq" | jq '.TotalSections // 0')" \
        --argjson stc "$(echo "$all_jq" | jq '(.SectionsByType | length) // 0')" \
        --argjson dmc "$(echo "$all_jq" | jq '(.Domains | length) // 0')" \
        --argjson ec "$PREV_ERRORS" \
        --argjson ttl "$TOTAL_CALLS" \
        '{
            timestamp: $ts,
            phase_scores: $ps,
            total_score: $ts_score,
            metrics: {
                tool_count: $tc, doc_count: $dc, section_count: $sc,
                section_type_count: $stc, domain_count: $dmc,
                error_count: $ec, total_calls: $ttl
            }
        }')
    printf '%s' "$metrics" > "$(metrics_json_path "$LATEST_DIR")"
}

build_phase_scores_json() {
    local order=(01-tool-health 02-domain-catalog 03-document-audit 04-section-integrity 05-search-results 06-audit-findings 07-coverage-gaps 08-registry-state)
    local arr='[]'
    for key in "${order[@]}"; do
        local pr="${PHASE_RESULTS[$key]:-}"
        [ -z "$pr" ] && continue
        arr=$(echo "$arr" | jq -c \
            --arg key "$key" \
            --argjson score "$(echo "$pr" | jq '(.Score // 0) | floor')" \
            --arg status "$(echo "$pr" | jq -r '.Status // "?"')" \
            --argjson errors "$(echo "$pr" | jq '(.Errors // 0)')" \
            --argjson dur "$(echo "$pr" | jq '(.Duration // 0)')" \
            '. + [{phase: $key, score: $score, status: $status, errors: $errors, duration: $dur}]')
    done
    echo "$arr"
}

# (load_previous_metrics, compute_trend, get_prev_metric, trend_between, format_score_line, gen_phase_analysis, gen_phase_recs sourced from scripts/lib/report.sh)

# ─── Phase 1: Bootstrap ────────────────────────────────────────────────────────

phase_1_bootstrap() {
    CURRENT_PHASE="01-tool-health"
    echo "Phase 1: Bootstrap..."
    local start
    start=$(date +%s)
    local checks='[]'

    # initialize
    local init_params
    init_params=$(jq -nc '{
        protocolVersion: "2025-03-26",
        capabilities: {},
        clientInfo: {name: "mcp-discover", version: "1.0"}
    }')
    local init_result
    init_result=$(invoke_mcp_direct "initialize" "$init_params")
    if [ -n "$init_result" ]; then
        local pv
        pv=$(echo "$init_result" | jq -r '.protocolVersion // "?"')
        ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg pv "$pv" '.Protocol = $pv')
        checks=$(echo "$checks" | jq --arg pv "$pv" '. += [{"Name": "Initialize", "Status": "pass", "Detail": "Protocol \($pv)"}]')
    else
        checks=$(echo "$checks" | jq '. += [{"Name": "Initialize", "Status": "fail", "Detail": "No response"}]')
    fi

    # tools/list
    local tools_result
    tools_result=$(invoke_mcp_direct "tools/list" "{}")
    local tools='[]'
    if [ -n "$tools_result" ]; then
        tools=$(echo "$tools_result" | jq -c '.tools // []')
        local tc
        tc=$(echo "$tools" | jq 'length')
        ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --argjson tools "$tools" '.Tools = $tools')
        checks=$(echo "$checks" | jq --argjson tc "$tc" '. += [{"Name": "Tools/List", "Status": "pass", "Detail": "\($tc) tools"}]')
    else
        checks=$(echo "$checks" | jq '. += [{"Name": "Tools/List", "Status": "fail", "Detail": "No tools returned"}]')
    fi

    # info via tools/call
    local info_result
    info_result=$(invoke_mcp_tool "info" "{}")
    if [ -n "$info_result" ]; then
        local dc
        dc=$(echo "$info_result" | jq -r '.document_count // "?"')
        ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --argjson info "$info_result" '.Runtime = $info')
        checks=$(echo "$checks" | jq --arg dc "$dc" '. += [{"Name": "Info", "Status": "pass", "Detail": "\($dc) docs"}]')
    else
        checks=$(echo "$checks" | jq '. += [{"Name": "Info", "Status": "fail", "Detail": "No response"}]')
    fi

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "01-tool-health")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local has_fail
    has_fail=$(echo "$checks" | jq '[.[] | select(.Status == "fail")] | length')
    local status="✅ PASS"
    [ "$has_fail" -gt 0 ] && status="❌ FAIL"

    # Build tool table
    local tool_rows
    tool_rows=$(echo "$tools" | jq -r '
        [to_entries[] | "| \(.key + 1) | \"\(.value.name)\" | " +
        (if .value.inputSchema and .value.inputSchema.required
         then (.value.inputSchema.required | join(", "))
         else "none" end) + " | ✅ |"
        ] | join("\n")')

    local standards_list="--"
    local standard_count=0
    local registry_path="--"
    local repository_name="--"
    local services="--"
    local policy="--"

    if [ "$info_result" != "null" ] && [ -n "$info_result" ]; then
        standards_list=$(echo "$info_result" | jq -r '(.standards // []) | join(", ")')
        [ -z "$standards_list" ] && standards_list="--"
        standard_count=$(echo "$info_result" | jq -r '(.standards // []) | length')
        registry_path=$(echo "$info_result" | jq -r '.registry_path // "--"')
        repository_name=$(echo "$info_result" | jq -r '.repository // "--"')
        services=$(echo "$info_result" | jq -r '(.services // []) | join(", ")')
        [ -z "$services" ] && services="--"
        policy=$(echo "$info_result" | jq -c '.policy // "--"')
    fi

    local doc_count
    doc_count=$(echo "$ALL_RESULTS" | jq -r '.Runtime.document_count // "?"')

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "01-tool-health")

    # Score: % of checks passing, penalized by errors
    local tc
    tc=$(echo "$checks" | jq 'length')
    local pass_c
    pass_c=$(echo "$checks" | jq '[.[] | select(.Status == "pass")] | length')
    local score=0
    [ "$tc" -gt 0 ] && score=$((pass_c * 100 / tc))
    [ "$error_count" -gt 0 ] && score=$((score - error_count * 5))
    [ "$score" -lt 0 ] && score=0

    local analysis
    analysis=$(gen_phase_analysis "01-tool-health" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "01-tool-health" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"01-tool-health\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")
    local prev_doc_count
    prev_doc_count=$(get_prev_metric ".metrics.doc_count // \"\"")
    local doc_trend
    doc_trend=$(trend_between "$doc_count" "$prev_doc_count")
    local healthy_tool_count
    healthy_tool_count=$(echo "$tools" | jq 'length')

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg TOOLS_TABLE "$tool_rows" \
        --arg DOC_COUNT "$doc_count" \
        --arg STANDARDS_LIST "$standards_list" \
        --arg STANDARD_COUNT "$standard_count" \
        --arg REGISTRY_PATH "$registry_path" \
        --arg REPOSITORY "$repository_name" \
        --arg SERVICES "$services" \
        --arg POLICY "$policy" \
        --arg TOOL_COUNT "$(echo "$tools" | jq 'length')" \
        --arg TOOL_ERROR_COUNT "$error_count" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        --arg PROTOCOL_VERSION "$(echo "$ALL_RESULTS" | jq -r '.Protocol // "?"')" \
        --arg HEALTHY_TOOL_COUNT "$healthy_tool_count" \
        --arg PREV_DOC_COUNT "${prev_doc_count:-}" \
        --arg DOC_TREND "$doc_trend" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          TOOLS_TABLE: $TOOLS_TABLE, DOC_COUNT: $DOC_COUNT,
          STANDARDS_LIST: $STANDARDS_LIST, STANDARD_COUNT: $STANDARD_COUNT,
          REGISTRY_PATH: $REGISTRY_PATH, REPOSITORY: $REPOSITORY,
          SERVICES: $SERVICES, POLICY: $POLICY,
          TOOL_COUNT: $TOOL_COUNT, TOOL_ERROR_COUNT: $TOOL_ERROR_COUNT,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE, PROTOCOL_VERSION: $PROTOCOL_VERSION,
          HEALTHY_TOOL_COUNT: $HEALTHY_TOOL_COUNT,
          PREV_DOC_COUNT: $PREV_DOC_COUNT, DOC_TREND: $DOC_TREND}')
    write_report "01-tool-health.md" "01-tool-health.md" "$report_vals" > /dev/null

    PHASE_RESULTS["01-tool-health"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "01-tool-health.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status (${duration}s)"
}

# ─── Phase 2: Domain Catalog ───────────────────────────────────────────────────

phase_2_domain_scan() {
    CURRENT_PHASE="02-domain-catalog"
    echo "Phase 2: Domain Scan..."
    local start
    start=$(date +%s)
    local checks='[]'

    local domains_result
    domains_result=$(invoke_mcp_tool "list_domains" "{}")
    local all_domain_names=()

    if [ -n "$domains_result" ]; then
        all_domain_names=()
        while IFS= read -r d; do
            all_domain_names+=("$d")
        done < <(echo "$domains_result" | jq -r '.domains[] // empty')

        local filtered_domains=()
        if [ ${#DOMAINS[@]} -gt 0 ]; then
            for d in "${all_domain_names[@]}"; do
                for filter in "${DOMAINS[@]}"; do
                    if [ "$d" = "$filter" ]; then
                        filtered_domains+=("$d")
                        break
                    fi
                done
            done
        else
            filtered_domains=("${all_domain_names[@]}")
        fi

        ALL_RESULTS=$(echo "$ALL_RESULTS" | jq '.Domains = {}')
        for d in "${filtered_domains[@]}"; do
            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg d "$d" '.Domains[$d] = {name: $d, docs: [], sectionTypes: {}, sectionIds: [], docCount: 0}')
        done

        checks=$(echo "$checks" | jq --argjson cnt "${#filtered_domains[@]}" \
            '. += [{"Name": "List Domains", "Status": "pass", "Detail": "\($cnt) domains"}]')
    else
        checks=$(echo "$checks" | jq '. += [{"Name": "List Domains", "Status": "fail", "Detail": "No domains"}]')
    fi

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "02-domain-catalog")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="❌ FAIL"
    if [ ${#all_domain_names[@]} -gt 0 ]; then status="✅ PASS"; fi

    # Score: 100 if any domains found, proportional to count, min 20 per domain
    local score=0
    local dc=${#all_domain_names[@]}
    [ "$dc" -gt 0 ] && score=$((dc * 25 > 100 ? 100 : dc * 25))
    [ "$error_count" -gt 0 ] && score=$((score - error_count * 10))
    [ "$score" -lt 0 ] && score=0

    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"02-domain-catalog\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    PHASE_RESULTS["02-domain-catalog"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "02-domain-catalog.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status (${duration}s)"
}

# ─── Phase 3: Document Discovery ───────────────────────────────────────────────

phase_3_doc_discover() {
    CURRENT_PHASE="03-document-audit"
    echo "Phase 3: Document Discovery..."
    local start
    start=$(date +%s)
    local checks='[]'

    local domain_names=()
    while IFS= read -r d; do
        domain_names+=("$d")
    done < <(echo "$ALL_RESULTS" | jq -r '.Domains | keys[]')

    local all_doc_count=0

    for d in "${domain_names[@]}"; do
        echo "  Fetching docs for '$d'..."
        local args
        args=$(jq -nc --arg d "$d" '{domain: $d}')
        local docs_json
        docs_json=$(invoke_mcp_tool_all "get_documents_by_domain" "$args" "documents")

        if [ -n "$docs_json" ] && [ "$docs_json" != "[]" ]; then
            local dc
            dc=$(echo "$docs_json" | jq 'length')
            local tmp_docs
            tmp_docs=$(mktemp)
            printf '%s' "$docs_json" > "$tmp_docs"
            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg d "$d" --slurpfile docs "$tmp_docs" \
                '.Domains[$d].docs = $docs[0] | .Domains[$d].docCount = ($docs[0] | length)')
            rm -f "$tmp_docs"
            all_doc_count=$((all_doc_count + dc))
            checks=$(echo "$checks" | jq --arg d "$d" --argjson dc "$dc" \
                '. += [{"Name": "Docs in '\''\($d)'\''", "Status": "pass", "Detail": "\($dc) docs"}]')
        else
            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg d "$d" '.Domains[$d].docCount = 0')
            checks=$(echo "$checks" | jq --arg d "$d" \
                '. += [{"Name": "Docs in '\''\($d)'\''", "Status": "skip", "Detail": "0 docs (or error)"}]')
        fi
    done

    ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --argjson td "$all_doc_count" '.TotalDocs = $td')

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "03-document-audit")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="❌ FAIL"
    if [ "$all_doc_count" -gt 0 ]; then status="✅ PASS"; fi
    local has_fail_checks
    has_fail_checks=$(echo "$checks" | jq '[.[] | select(.Status == "fail")] | length')
    [ "$has_fail_checks" -gt 0 ] && status="⚠️ PARTIAL"

    # Domain catalog report
    local dc_domain_rows=""
    local dc_doc_count_rows=""
    local di=0
    for d in "${domain_names[@]}"; do
        di=$((di + 1))
        local cnt
        cnt=$(echo "$ALL_RESULTS" | jq -r ".Domains[\"$d\"].docCount // 0")
        dc_domain_rows+="| $di | \"$d\" | $cnt | $cnt |"$'\n'
        dc_doc_count_rows+="| $d | $cnt |"$'\n'
    done

    local dc_standards="--"
    local sc=0
    local runtime
    runtime=$(echo "$ALL_RESULTS" | jq -c '.Runtime // null')
    if [ "$runtime" != "null" ]; then
        dc_standards=$(echo "$runtime" | jq -r '(.standards // []) | map("- \(.)") | join("\n")')
        [ -z "$dc_standards" ] && dc_standards="--"
        sc=$(echo "$runtime" | jq -r '(.standards // []) | length')
    fi

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "03-document-audit")

    # For the report, show all checks from both phase 2 and 3
    local all_checks
    all_checks=$(echo "$checks" | jq '. += [{"Name": "List Domains", "Status": "pass", "Detail": "'"${#domain_names[@]}"' domains"}]')
    local all_checks_table
    all_checks_table=$(get_checks_table "$all_checks")

    local phase2_errors
    phase2_errors=$(get_phase_errors_json "02-domain-catalog")
    local p2_error_rows
    p2_error_rows=$(echo "$phase2_errors" | jq -r '
        if length == 0 then "No errors" else
        (["| Tool Call | Error | Response |", "|-----------|-------|----------|"] +
        (.[] | "| " + (.Tool | gsub("\\|"; "\\|")) + " | " +
               (.Error | gsub("\\|"; "\\|")) + " | " +
               (.Response[0:120] | gsub("\\|"; "\\|")) + " |")) | join("\n") end')

    # Phase 2 score from stored results
    local p2_score
    p2_score=$(echo "${PHASE_RESULTS["02-domain-catalog"]:-}" | jq -r '.Score // 0')
    local p2_prev
    p2_prev=$(get_prev_metric ".phase_scores[] | select(.phase == \"02-domain-catalog\") | .score // \"\"")
    local p2_trend
    p2_trend=$(trend_between "$p2_score" "$p2_prev")
    local p2_analysis
    p2_analysis=$(gen_phase_analysis "02-domain-catalog" "$all_checks")
    local p2_recs
    p2_recs=$(gen_phase_recs "02-domain-catalog" "$all_checks")

    local prev_doc_count
    prev_doc_count=$(get_prev_metric ".metrics.doc_count // \"\"")
    local doc_trend
    doc_trend=$(trend_between "$all_doc_count" "$prev_doc_count")
    local prev_domain_count
    prev_domain_count=$(get_prev_metric ".metrics.domain_count // \"\"")
    local domain_trend
    domain_trend=$(trend_between "${#domain_names[@]}" "$prev_domain_count")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "✅ PASS" \
        --arg CHECKS_TABLE "$all_checks_table" \
        --arg ERRORS_TABLE "$p2_error_rows" \
        --arg DOMAINS_TABLE "$dc_domain_rows" \
        --arg STANDARDS_LIST "$dc_standards" \
        --arg STANDARD_COUNT "$sc" \
        --arg DOC_COUNTS_TABLE "$dc_doc_count_rows" \
        --arg DOMAIN_COUNT "${#domain_names[@]}" \
        --arg DOCUMENT_COUNT "$all_doc_count" \
        --arg SCORE "$p2_score" \
        --arg TREND "$p2_trend" \
        --arg ANALYSIS "$p2_analysis" \
        --arg RECOMMENDATIONS "$p2_recs" \
        --arg PREV_SCORE "${p2_prev:-}" \
        --arg PREV_DOC_COUNT "${prev_doc_count:-}" \
        --arg DOC_TREND "$doc_trend" \
        --arg PREV_DOMAIN_COUNT "${prev_domain_count:-}" \
        --arg DOMAIN_TREND "$domain_trend" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          DOMAINS_TABLE: $DOMAINS_TABLE, STANDARDS_LIST: $STANDARDS_LIST,
          STANDARD_COUNT: $STANDARD_COUNT, DOC_COUNTS_TABLE: $DOC_COUNTS_TABLE,
          DOMAIN_COUNT: $DOMAIN_COUNT, DOCUMENT_COUNT: $DOCUMENT_COUNT,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE, PREV_DOC_COUNT: $PREV_DOC_COUNT,
          DOC_TREND: $DOC_TREND,
          PREV_DOMAIN_COUNT: $PREV_DOMAIN_COUNT, DOMAIN_TREND: $DOMAIN_TREND}')
    write_report "02-domain-catalog.md" "02-domain-catalog.md" "$report_vals" > /dev/null

    # Score: doc discovery rate compared to expectation
    local score=0
    local domains_with_docs
    domains_with_docs=$(echo "$ALL_RESULTS" | jq '[.Domains | to_entries[] | select(.value.docCount > 0)] | length')
    local expected=$(( domains_with_docs * 3 ))  # expect at least 3 docs per active domain
    [ "$expected" -lt 1 ] && expected=1
    if [ "$all_doc_count" -ge "$expected" ]; then score=100
    elif [ "$all_doc_count" -gt 0 ]; then score=$((all_doc_count * 100 / expected))
    fi
    [ "$error_count" -gt 0 ] && score=$((score - error_count * 5))
    [ "$score" -lt 0 ] && score=0

    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"03-document-audit\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    PHASE_RESULTS["03-doc-discover"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, Score: $score}')

    echo "  → Score: $score/100 $trend — $status ($all_doc_count docs discovered)"
}

# ─── Phase 4: Document Verification ────────────────────────────────────────────

phase_4_doc_verify() {
    CURRENT_PHASE="03-document-audit"
    echo "Phase 4: Document Verification..."
    local start
    start=$(date +%s)
    local checks='[]'

    local domain_names=()
    while IFS= read -r d; do
        domain_names+=("$d")
    done < <(echo "$ALL_RESULTS" | jq -r '.Domains | keys[]')

    local all_sections_total=0
    local quality_rows=""
    local issues_rows=""
    local issue_count=0
    local doc_parts=""

    declare -A sect_dist
    sect_dist["0"]=0; sect_dist["1-3"]=0; sect_dist["4-7"]=0; sect_dist["8-15"]=0; sect_dist["16+"]=0

    for d in "${domain_names[@]}"; do
        local docs_json
        docs_json=$(echo "$ALL_RESULTS" | jq -c ".Domains[\"$d\"].docs // []")
        local doc_count
        doc_count=$(echo "$docs_json" | jq 'length')

        local total_sects=0 empty_sects=0 missing_sects=0
        local doc_parts=""
        doc_parts+="### $d"$'\n\n'
        doc_parts+="| Doc ID | Title | Sections | Coverage | Issues |"$'\n'
        doc_parts+="|--------|-------|----------|----------|--------|"$'\n'

        for doc_idx in $(seq 0 $((doc_count - 1))); do
            local doc
            doc=$(echo "$docs_json" | jq -c ".[$doc_idx]")
            local doc_id title quality body
            doc_id=$(echo "$doc" | jq -r '.id // "?"')
            title=$(echo "$doc" | jq -r '.title // "?"' | sed 's/|/\\|/g')
            quality=$(echo "$doc" | jq -c '.quality // null')
            body=$(echo "$doc" | jq -c '.body // null')

            local sect_count=0 empty_count=0 missing_count=0 coverage=0 req_count=0
            if [ "$quality" != "null" ]; then
                sect_count=$(echo "$quality" | jq -r '.total_section_count // 0')
                empty_count=$(echo "$quality" | jq -r '.empty_section_count // 0')
                missing_count=$(echo "$quality" | jq -r '.missing_section_count // 0')
                coverage=$(echo "$quality" | jq -r '.coverage // 0')
                req_count=$(echo "$quality" | jq -r '.required_section_count // 0')
            fi

            total_sects=$((total_sects + sect_count))
            [ "$empty_count" -gt 0 ] && empty_sects=$((empty_sects + empty_count))
            [ "$missing_count" -gt 0 ] && missing_sects=$((missing_sects + missing_count))

            # Section distribution
            if [ "$sect_count" -eq 0 ]; then sect_dist["0"]=$((sect_dist["0"] + 1))
            elif [ "$sect_count" -le 3 ]; then sect_dist["1-3"]=$((sect_dist["1-3"] + 1))
            elif [ "$sect_count" -le 7 ]; then sect_dist["4-7"]=$((sect_dist["4-7"] + 1))
            elif [ "$sect_count" -le 15 ]; then sect_dist["8-15"]=$((sect_dist["8-15"] + 1))
            else sect_dist["16+"]=$((sect_dist["16+"] + 1))
            fi

            # Collect section types per domain
            if [ "$body" != "null" ]; then
                local body_val
                body_val=$(echo "$body" | jq -c 'to_entries | first | .value // null')
                if [ "$body_val" != "null" ]; then
                    local sections_arr
                    sections_arr=$(echo "$body_val" | jq -c '.sections // []')
                    local sc
                    sc=$(echo "$sections_arr" | jq 'length')
                    for si in $(seq 0 $((sc - 1))); do
                        local st
                        st=$(echo "$sections_arr" | jq -r ".[$si].semantic_type // empty")
                        if [ -n "$st" ]; then
                            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg d "$d" --arg st "$st" \
                                '.Domains[$d].sectionTypes[$st] += 1')
                            all_sections_total=$((all_sections_total + 1))
                        fi
                    done
                fi
            fi

            # Issues
            local doc_issues=()
            if [ "$empty_count" -gt 0 ]; then doc_issues+=("$empty_count empty sections"); issue_count=$((issue_count + 1)); fi
            if [ "$missing_count" -gt 0 ]; then doc_issues+=("$missing_count missing sections"); issue_count=$((issue_count + 1)); fi
            local cov_int
            cov_int=$(echo "$coverage * 100 / 1" | bc 2>/dev/null || echo "$coverage")
            if [ "$(echo "$coverage < 0.5" | bc 2>/dev/null || echo 0)" -eq 1 ]; then
                doc_issues+=("low coverage ($coverage)")
                issue_count=$((issue_count + 1))
            fi
            local doc_hash
            doc_hash=$(echo "$doc" | jq -r '.hash // ""')
            if [ -z "$doc_hash" ]; then
                doc_issues+=("no hash")
                issue_count=$((issue_count + 1))
            fi

            local issue_str
            if [ ${#doc_issues[@]} -gt 0 ]; then
                local IFS='; '
                issue_str="${doc_issues[*]}"
                unset IFS
            else
                issue_str="✅"
            fi

            local cov_pct
            cov_pct=$(echo "$coverage * 100" | bc 2>/dev/null || echo "$coverage")
            cov_pct="${cov_pct%.*}"
            [ -z "$cov_pct" ] && cov_pct="0"
            [ "$coverage" = "0" ] && cov_pct="0"

            doc_parts+="| $doc_id | \"$title\" | $sect_count | ${cov_pct}% | $issue_str |"$'\n'

            if [ ${#doc_issues[@]} -gt 0 ]; then
                local IFS='; '
                issues_rows+="| $doc_id | \"$title\" | $d | ${doc_issues[*]} |"$'\n'
                unset IFS
            fi

            # Store in AllDocs
            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --argjson doc "$doc" --arg domain "$d" \
                '.AllDocs += [{"doc": $doc, "domain": $domain}]')
        done

        local avg_sects=0
        [ "$doc_count" -gt 0 ] && avg_sects=$((total_sects / doc_count))
        quality_rows+="| $d | $doc_count | $avg_sects | $empty_sects | $missing_sects | -- |"$'\n'
    done

    ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --argjson ts "$all_sections_total" '.TotalSections = $ts')

    checks=$(echo "$checks" | jq --argjson dc "$(echo "$ALL_RESULTS" | jq '.AllDocs | length')" \
        --argjson dm "${#domain_names[@]}" \
        '. += [{"Name": "Document verification", "Status": "pass", "Detail": "\($dc) docs across \($dm) domains"}]')
    checks=$(echo "$checks" | jq --argjson ts "$all_sections_total" \
        '. += [{"Name": "Section count", "Status": "pass", "Detail": "\($ts) sections total"}]')

    local dist_rows=""
    for k in "0" "1-3" "4-7" "8-15" "16+"; do
        local v=${sect_dist[$k]}
        if [ "$v" -gt 0 ]; then
            dist_rows+="| $k sections | $v docs |"$'\n'
        fi
    done

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "03-document-audit")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="✅ PASS"
    [ "$issue_count" -gt 0 ] && status="⚠️ PARTIAL"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "03-document-audit")

    local all_docs_count
    all_docs_count=$(echo "$ALL_RESULTS" | jq '.AllDocs | length')

    local issues_table="No issues found"
    if [ -n "$issues_rows" ]; then
        issues_table="| Doc ID | Title | Domain | Issues |"$'\n'"|--------|-------|--------|--------|"$'\n'"$issues_rows"
    fi

    [ -z "$dist_rows" ] && dist_rows="| -- | -- |"

    # Score: weighted by coverage, empty sections penalty, missing sections penalty
    local score=0
    if [ "$all_sections_total" -gt 0 ] || [ "$all_docs_count" -gt 0 ]; then
        local cov_score=50 empty_penalty=0 missing_penalty=0
        # Coverage score based on issue count
        local max_issues=$((all_docs_count * 3))
        [ "$max_issues" -lt 1 ] && max_issues=1
        local issue_ratio=$((issue_count * 100 / max_issues))
        cov_score=$((50 - issue_ratio / 2))
        [ "$cov_score" -lt 0 ] && cov_score=0
        # Empty sections penalty
        empty_penalty=$((empty_sects * 5))
        missing_penalty=$((missing_sects * 5))
        score=$((cov_score + 50 - empty_penalty - missing_penalty))
    fi
    [ "$score" -lt 0 ] && score=0
    [ "$score" -gt 100 ] && score=100

    local analysis
    analysis=$(gen_phase_analysis "03-document-audit" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "03-document-audit" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"03-document-audit\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")
    local prev_sect_count
    prev_sect_count=$(get_prev_metric ".metrics.section_count // \"\"")
    local sect_trend
    sect_trend=$(trend_between "$all_sections_total" "$prev_sect_count")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg DOMAIN_DOCS_SECTIONS "$doc_parts" \
        --arg QUALITY_TABLE "$quality_rows" \
        --arg ISSUES_LIST "$issues_table" \
        --arg SECTION_DIST_TABLE "$dist_rows" \
        --arg TOTAL_DOCS "$all_docs_count" \
        --arg DOMAIN_COUNT "${#domain_names[@]}" \
        --arg TOTAL_SECTIONS "$all_sections_total" \
        --arg ISSUE_COUNT "$issue_count" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        --arg PREV_SECT_COUNT "${prev_sect_count:-}" \
        --arg SECT_TREND "$sect_trend" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          DOMAIN_DOCS_SECTIONS: $DOMAIN_DOCS_SECTIONS,
          QUALITY_TABLE: $QUALITY_TABLE, ISSUES_LIST: $ISSUES_LIST,
          SECTION_DIST_TABLE: $SECTION_DIST_TABLE,
          TOTAL_DOCS: $TOTAL_DOCS, DOMAIN_COUNT: $DOMAIN_COUNT,
          TOTAL_SECTIONS: $TOTAL_SECTIONS, ISSUE_COUNT: $ISSUE_COUNT,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE,
          PREV_SECT_COUNT: $PREV_SECT_COUNT, SECT_TREND: $SECT_TREND}')
    write_report "03-document-audit.md" "03-document-audit.md" "$report_vals" > /dev/null

    PHASE_RESULTS["03-document-audit"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "03-document-audit.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status ($issue_count issues)"
}

# ─── Phase 5: Cross-Section ────────────────────────────────────────────────────

phase_5_cross_section() {
    CURRENT_PHASE="04-section-integrity"
    echo "Phase 5: Cross-Section..."
    local start
    start=$(date +%s)
    local checks='[]'

    # Collect unique (domain, type) pairs
    local pairs_json
    pairs_json=$(echo "$ALL_RESULTS" | jq -c '[.Domains | to_entries[] |
        .key as $d |
        .value.sectionTypes | to_entries[] |
        {domain: $d, type: .key}]')

    local pair_count
    pair_count=$(echo "$pairs_json" | jq 'length')

    local total_sections=0

    for pi in $(seq 0 $((pair_count - 1))); do
        local pair
        pair=$(echo "$pairs_json" | jq -c ".[$pi]")
        local d t
        d=$(echo "$pair" | jq -r '.domain')
        t=$(echo "$pair" | jq -r '.type')

        local args
        args=$(jq -nc --arg t "$t" --arg d "$d" '{semantic_type: $t, domain: $d}')
        local sects
        sects=$(invoke_mcp_tool_all "get_sections" "$args" "sections" 100)

        if [ -n "$sects" ] && [ "$sects" != "[]" ]; then
            local ids
            ids=$(echo "$sects" | jq -c '[.[] | .id]')
            local sc
            sc=$(echo "$sects" | jq 'length')
            total_sections=$((total_sections + sc))

            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg d "$d" --arg t "$t" --argjson ids "$ids" \
                '.SectionIds[$d][$t] = $ids')

            ALL_RESULTS=$(echo "$ALL_RESULTS" | jq --arg t "$t" --argjson sects "$sects" \
                '.SectionsByType[$t] += $sects')
        fi
    done

    checks=$(echo "$checks" | jq --argjson pc "$pair_count" --argjson ts "$total_sections" \
        '. += [{"Name": "Cross-section query", "Status": "pass", "Detail": "\($pc) type-domain pairs, \($ts) sections"}]')

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "04-section-integrity")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="⚠️ PARTIAL"
    [ "$total_sections" -gt 0 ] && status="✅ PASS"

    # Score: retrieval rate
    local score=0
    if [ "$pair_count" -gt 0 ]; then
        local expected_sects=$((pair_count * 3))
        [ "$total_sections" -ge "$expected_sects" ] && score=100 || score=$((total_sections * 100 / expected_sects))
    fi
    [ "$error_count" -gt 0 ] && score=$((score - error_count * 5))
    [ "$score" -lt 0 ] && score=0

    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"04-section-integrity\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    PHASE_RESULTS["04-cross-section"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, Score: $score}')
    echo "  → $total_sections sections from $pair_count type-domain pairs"
}

# ─── Phase 6: Section Verification ─────────────────────────────────────────────

phase_6_section_verify() {
    CURRENT_PHASE="04-section-integrity"
    echo "Phase 6: Section Verification..."
    local start
    start=$(date +%s)
    local checks='[]'

    local section_type_rows=""
    local verify_lines=""
    local knowledge_rows=""
    local change_track_rows=""
    local total_sections=0
    local stale_count=0
    local knowledge_count=0
    local knowledge_missing=0

    # Section types by domain
    local domain_names=()
    while IFS= read -r d; do
        domain_names+=("$d")
    done < <(echo "$ALL_RESULTS" | jq -r '.Domains | keys[]')
    for d in "${domain_names[@]}"; do
        local types_list
        types_list=$(echo "$ALL_RESULTS" | jq -r --arg d "$d" '
            .Domains[$d].sectionTypes | to_entries |
            map("\(.key) (\(.value))") | join(", ")')
        if [ -n "$types_list" ]; then
            section_type_rows+="| $d | $types_list |"$'\n'
        fi
    done

    # Verify each section_id
    verify_lines+="| Domain | Section ID | Type | get_section | changed |"$'\n'
    verify_lines+="|--------|-----------|------|-------------|---------|"$'\n'
    local verify_count=0 max_verify=500
    local all_unique_types_json
    all_unique_types_json=$(echo "$ALL_RESULTS" | jq -c '[.SectionIds | .. | objects | select(.type?) // empty]')

    local pairs_json
    pairs_json=$(echo "$ALL_RESULTS" | jq -c '[.SectionIds | to_entries[] |
        .key as $d |
        .value | to_entries[] |
        {domain: $d, type: .key, ids: .value}]')

    local pair_count
    pair_count=$(echo "$pairs_json" | jq 'length')

    for pi in $(seq 0 $((pair_count - 1))); do
        local pair
        pair=$(echo "$pairs_json" | jq -c ".[$pi]")
        local d t ids_json
        d=$(echo "$pair" | jq -r '.domain')
        t=$(echo "$pair" | jq -r '.type')
        ids_json=$(echo "$pair" | jq -c '.ids // []')
        local id_count
        id_count=$(echo "$ids_json" | jq 'length')

        for idi in $(seq 0 $((id_count - 1))); do
            [ "$verify_count" -ge "$max_verify" ] && break
            verify_count=$((verify_count + 1))
            local sid
            sid=$(echo "$ids_json" | jq -r ".[$idi]")

            # get_section
            local sect_args
            sect_args=$(jq -nc --arg sid "$sid" '{section_id: $sid}')
            local sect_result
            sect_result=$(invoke_mcp_tool "get_section" "$sect_args")
            local sect_ok="❌"
            if [ -n "$sect_result" ]; then
                local rid
                rid=$(echo "$sect_result" | jq -r '.id // ""')
                [ "$rid" = "$sid" ] && sect_ok="✅"
            fi

            # get_section_changed
            local changed_args
            changed_args=$(jq -nc --arg sid "$sid" '{section_id: $sid}')
            local changed_result
            changed_result=$(invoke_mcp_tool "get_section_changed" "$changed_args")
            local changed="?"
            if [ -n "$changed_result" ]; then
                changed=$(echo "$changed_result" | jq -r '.changed // "?"')
                [ "$changed" = "true" ] && stale_count=$((stale_count + 1))
            fi

            verify_lines+="| $d | $sid | $t | $sect_ok | $changed |"$'\n'
            total_sections=$((total_sections + 1))
        done

        # Audit knowledge per (domain, type) pair
        if ! $NO_AUDIT; then
            local kn_args
            kn_args=$(jq -nc --arg d "$d" --arg t "$t" '{domain: $d, section_type: $t}')
            local kn_result
            kn_result=$(invoke_mcp_tool "get_audit_knowledge" "$kn_args" "true")
            if [ -n "$kn_result" ]; then
                local kn_content
                kn_content=$(echo "$kn_result" | jq -r '.content // ""')
                if [ -n "$kn_content" ]; then
                    knowledge_count=$((knowledge_count + 1))
                    local kn_len=${#kn_content}
                    knowledge_rows+="| $d | $t | ✅ $kn_len chars |"$'\n'
                else
                    knowledge_missing=$((knowledge_missing + 1))
                    knowledge_rows+="| $d | $t | ❌ Missing |"$'\n'
                fi
            else
                knowledge_missing=$((knowledge_missing + 1))
                knowledge_rows+="| $d | $t | ❌ Missing |"$'\n'
            fi
        fi

        [ "$verify_count" -ge "$max_verify" ] && break
    done

    if [ "$total_sections" -eq 0 ]; then
        # Fallback: verify from get_document_section
        echo "  No section_ids found from cross-section; using get_document_section" >&2
        verify_lines=""
        verify_lines+="| Doc ID | Section Index | Heading | Content Available |"$'\n'
        verify_lines+="|--------|--------------|--------|-------------------|"$'\n'

        local all_docs_json
        all_docs_json=$(echo "$ALL_RESULTS" | jq -c '.AllDocs // []')
        local ad_count
        ad_count=$(echo "$all_docs_json" | jq 'length')
        local sample_count=10
        [ "$ad_count" -lt "$sample_count" ] && sample_count=$ad_count

        for adi in $(seq 0 $((sample_count - 1))); do
            local item
            item=$(echo "$all_docs_json" | jq -c ".[$adi]")
            local adoc
            adoc=$(echo "$item" | jq -c '.doc')
            local doc_id ad_title body_val
            doc_id=$(echo "$adoc" | jq -r '.id')
            ad_title=$(echo "$adoc" | jq -r '.title | gsub("\\|"; "\\\\|")')
            body_val=$(echo "$adoc" | jq -c '[.body | to_entries | first | .value // null] | first')

            local sections
            sections=$(echo "$body_val" | jq -c '.sections // []')
            local section_count
            section_count=$(echo "$sections" | jq 'length')
            local max_s=$section_count
            [ "$MAX_SECTIONS" -gt 0 ] && [ "$MAX_SECTIONS" -lt "$max_s" ] && max_s=$MAX_SECTIONS

            for si in $(seq 0 $((max_s - 1))); do
                local heading s_body
                heading=$(echo "$sections" | jq -r ".[$si].heading | gsub(\"\\|\"; \"\\\\|\")")
                s_body=$(echo "$sections" | jq -r ".[$si].body // \"\"")

                if $NO_SECTION_CONTENT; then
                    local content_ok="❌ empty"
                    [ -n "$s_body" ] && content_ok="✅"
                    verify_lines+="| $doc_id | $si | $heading | $content_ok |"$'\n'
                else
                    local ds_args
                    ds_args=$(jq -nc --arg id "$doc_id" --argjson si "$si" '{id: $id, section: $si, limit: 5}')
                    local ds_result
                    ds_result=$(invoke_mcp_tool "get_document_section" "$ds_args")
                    local ds_content=""
                    [ -n "$ds_result" ] && ds_content=$(echo "$ds_result" | jq -r '.content // ""')
                    local content_ok="❌"
                    if [ -n "$ds_content" ]; then
                        local ds_len=${#ds_content}
                        content_ok="✅ ($ds_len chars)"
                    fi
                    verify_lines+="| $doc_id | $si | $heading | $content_ok |"$'\n'
                    total_sections=$((total_sections + 1))
                fi
            done
        done
    fi

    # Count unique types
    local unique_types
    unique_types=$(echo "$ALL_RESULTS" | jq -c '[.SectionIds | .. | objects | select(.type?) // empty]')
    local type_count
    type_count=$(echo "$ALL_RESULTS" | jq -c '[.SectionIds | to_entries[] | .value | to_entries[] | .key] | unique | length')

    change_track_rows+="| Stale (changed) | $stale_count |"$'\n'
    change_track_rows+="| Fresh (unchanged) | $((total_sections - stale_count)) |"$'\n'
    [ -z "$knowledge_rows" ] && knowledge_rows="| -- | -- | -- |"$'\n'

    checks=$(echo "$checks" | jq --argjson ts "$total_sections" \
        '. += [{"Name": "Section verification", "Status": "pass", "Detail": "\($ts) sections checked"}]')
    if [ "$stale_count" -gt 0 ]; then
        checks=$(echo "$checks" | jq --argjson sc "$stale_count" \
            '. += [{"Name": "Stale sections", "Status": "warn", "Detail": "\($sc) changed since last audit"}]')
    fi

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "04-section-integrity")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')
    local status="✅ PASS"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "04-section-integrity")
    [ -z "$section_type_rows" ] && section_type_rows="| -- | -- |"$'\n'

    # Score: verification success rate + knowledge coverage
    local score=0
    if [ "$total_sections" -gt 0 ]; then
        local ver_ok=$((total_sections - stale_count))
        local ver_rate=$((ver_ok * 50 / total_sections))
        local kn_rate=0
        local kn_total=$((knowledge_count + knowledge_missing))
        [ "$kn_total" -gt 0 ] && kn_rate=$((knowledge_count * 50 / kn_total))
        score=$((ver_rate + kn_rate))
    fi
    [ "$error_count" -gt 0 ] && score=$((score - error_count * 3))
    [ "$score" -lt 0 ] && score=0

    local analysis
    analysis=$(gen_phase_analysis "04-section-integrity" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "04-section-integrity" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"04-section-integrity\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")
    local prev_stale
    prev_stale=$(get_prev_metric ".phase_scores[] | select(.phase == \"04-section-integrity\") | .stale // \"\"")
    local stale_trend
    stale_trend=$(trend_between "$stale_count" "$prev_stale")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg SECTION_TYPES_TABLE "$section_type_rows" \
        --arg SECTION_VERIFY_TABLE "$verify_lines" \
        --arg CHANGE_TRACKING_TABLE "$change_track_rows" \
        --arg KNOWLEDGE_TABLE "$knowledge_rows" \
        --arg TOTAL_SECTIONS "$total_sections" \
        --arg DOMAIN_COUNT "${#domain_names[@]}" \
        --arg UNIQUE_TYPES "$type_count" \
        --arg STALE_SECTIONS "$stale_count" \
        --arg KNOWLEDGE_COUNT "$knowledge_count" \
        --arg KNOWLEDGE_MISSING "$knowledge_missing" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        --arg PREV_STALE "${prev_stale:-}" \
        --arg STALE_TREND "$stale_trend" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          SECTION_TYPES_TABLE: $SECTION_TYPES_TABLE,
          SECTION_VERIFY_TABLE: $SECTION_VERIFY_TABLE,
          CHANGE_TRACKING_TABLE: $CHANGE_TRACKING_TABLE,
          KNOWLEDGE_TABLE: $KNOWLEDGE_TABLE,
          TOTAL_SECTIONS: $TOTAL_SECTIONS, DOMAIN_COUNT: $DOMAIN_COUNT,
          UNIQUE_TYPES: $UNIQUE_TYPES, STALE_SECTIONS: $STALE_SECTIONS,
          KNOWLEDGE_COUNT: $KNOWLEDGE_COUNT, KNOWLEDGE_MISSING: $KNOWLEDGE_MISSING,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE,
          PREV_STALE: $PREV_STALE, STALE_TREND: $STALE_TREND}')
    write_report "04-section-integrity.md" "04-section-integrity.md" "$report_vals" > /dev/null

    PHASE_RESULTS["04-section-integrity"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "04-section-integrity.md" \
        --argjson score "$score" \
        --argjson stale "$stale_count" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score, Stale: $stale}')
    echo "  → Score: $score/100 $trend — $status ($total_sections sections)"
}

# ─── Phase 7: Search ───────────────────────────────────────────────────────────

phase_7_search() {
    CURRENT_PHASE="05-search-results"
    echo "Phase 7: Search..."
    local start
    start=$(date +%s)
    local checks='[]'

    local query_result_parts=""
    local expected_found=0 expected_total=0 search_errors=0

    local sample_queries=()

    # Get up to 5 doc titles as search queries
    local all_docs_json
    all_docs_json=$(echo "$ALL_RESULTS" | jq -c '.AllDocs // []')
    local ad_count
    ad_count=$(echo "$all_docs_json" | jq 'length')
    for adi in $(seq 0 $((ad_count - 1))); do
        local t
        t=$(echo "$all_docs_json" | jq -r ".[$adi].doc.title // \"\"")
        local len=${#t}
        if [ "$len" -gt 3 ] && [ "$len" -lt 40 ]; then
            sample_queries+=("$t")
        fi
        [ ${#sample_queries[@]} -ge 5 ] && break
    done

    # Add general queries
    sample_queries+=("purpose")
    sample_queries+=("architecture")

    expected_total=${#sample_queries[@]}

    for q in "${sample_queries[@]}"; do
        local args
        args=$(jq -nc --arg q "$q" '{query: $q, limit: 5}')
        local result
        result=$(invoke_mcp_tool "search" "$args")

        if [ -n "$result" ]; then
            local hit_count total_hits
            hit_count=$(echo "$result" | jq -r '.results | length // 0')
            total_hits=$(echo "$result" | jq -r '.total // 0')
            expected_found=$((expected_found + 1))

            query_result_parts+="### Query: \"$q\""$'\n\n'
            query_result_parts+="| Document | Title | Score |"$'\n'
            query_result_parts+="|----------|-------|-------|"$'\n'
            local hits_json
            hits_json=$(echo "$result" | jq -c '.results // []')
            local hc
            hc=$(echo "$hits_json" | jq 'length')
            for hi in $(seq 0 $((hc - 1))); do
                local hit
                hit=$(echo "$hits_json" | jq -c ".[$hi]")
                local hit_id hit_title score
                hit_id=$(echo "$hit" | jq -r '.document_id // "?"')
                hit_title=$(echo "$hit" | jq -r '.title // "?"' | sed 's/|/\\|/g')
                score=$(echo "$hit" | jq -r '.score // "--"')
                query_result_parts+="| $hit_id | \"$hit_title\" | $score |"$'\n'
            done
            query_result_parts+=$'\n_Results: '"$hit_count shown, $total_hits total"$'_\n\n'
            checks=$(echo "$checks" | jq --arg q "$q" --argjson hc "$hit_count" \
                '. += [{"Name": "Search \"\($q)\"", "Status": "pass", "Detail": "\($hc) results"}]')
        else
            search_errors=$((search_errors + 1))
            query_result_parts+="### Query: \"$q\""$'\n\n'"❌ No results or error"$'\n\n'
            checks=$(echo "$checks" | jq --arg q "$q" \
                '. += [{"Name": "Search \"\($q)\"", "Status": "warn", "Detail": "No results"}]')
        fi
    done

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "05-search-results")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="✅ PASS"
    [ "$search_errors" -gt 0 ] && status="⚠️ PARTIAL"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "05-search-results")

    # Score: % of queries returning results
    local score=0
    [ "$expected_total" -gt 0 ] && score=$((expected_found * 100 / expected_total))

    local analysis
    analysis=$(gen_phase_analysis "05-search-results" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "05-search-results" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"05-search-results\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg QUERY_RESULTS "$query_result_parts" \
        --arg QUERY_COUNT "${#sample_queries[@]}" \
        --arg EXPECTED_FOUND "$expected_found" \
        --arg EXPECTED_TOTAL "$expected_total" \
        --arg SEARCH_ERRORS "$search_errors" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          QUERY_RESULTS: $QUERY_RESULTS, QUERY_COUNT: $QUERY_COUNT,
          EXPECTED_FOUND: $EXPECTED_FOUND, EXPECTED_TOTAL: $EXPECTED_TOTAL,
          SEARCH_ERRORS: $SEARCH_ERRORS,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE}')
    write_report "05-search-results.md" "05-search-results.md" "$report_vals" > /dev/null

    PHASE_RESULTS["05-search-results"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "05-search-results.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status ($expected_found/$expected_total queries OK)"
}

# ─── Phase 8: Audit ────────────────────────────────────────────────────────────

phase_8_audit() {
    CURRENT_PHASE="06-audit-findings"
    echo "Phase 8: Audit..."
    local start
    start=$(date +%s)
    local checks='[]'

    if $NO_AUDIT; then
        echo "  Skipped (NoAudit)" >&2

        local report_vals
        report_vals=$(jq -n \
            --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
            --arg DURATION "0" \
            --arg STATUS "⬜ SKIPPED" \
            --arg CHECKS_TABLE "| - | Audit | ⬜ | Skipped via --no-audit |" \
            --arg ERRORS_TABLE "✅ No errors" \
            --arg AUDIT_SCORES_TABLE "| -- | -- | -- | -- | -- | -- |" \
            --arg FINDINGS_BY_DOMAIN "--" \
            --arg GATES_TABLE "| -- | -- | -- | -- | -- |" \
            --arg BLOCKED_GATES_DETAIL "--" \
            --arg DOMAIN_COUNT "0" \
            --arg TOTAL_FINDINGS "0" \
            --arg GATE_PASSES "0" \
            --arg GATE_TOTAL "0" \
            --arg GATE_BLOCKS "0" \
            --arg SCORE "0" \
            --arg TREND "—" \
            --arg ANALYSIS "Audit phase was skipped via --no-audit flag." \
            --arg RECOMMENDATIONS "Run without --no-audit to assess audit health." \
            --arg PREV_SCORE "" \
            '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
              CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
              AUDIT_SCORES_TABLE: $AUDIT_SCORES_TABLE,
              FINDINGS_BY_DOMAIN: $FINDINGS_BY_DOMAIN,
              GATES_TABLE: $GATES_TABLE,
              BLOCKED_GATES_DETAIL: $BLOCKED_GATES_DETAIL,
              DOMAIN_COUNT: $DOMAIN_COUNT, TOTAL_FINDINGS: $TOTAL_FINDINGS,
              GATE_PASSES: $GATE_PASSES, GATE_TOTAL: $GATE_TOTAL,
              GATE_BLOCKS: $GATE_BLOCKS,
              SCORE: $SCORE, TREND: $TREND,
              ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
              PREV_SCORE: $PREV_SCORE}')
        write_report "06-audit-findings.md" "06-audit-findings.md" "$report_vals" > /dev/null

        PHASE_RESULTS["06-audit-findings"]=$(jq -n \
            --arg status "⬜ SKIPPED" \
            --argjson errors 0 \
            --arg duration "0" \
            --arg report "06-audit-findings.md" \
            --argjson score 0 \
            '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
        return
    fi

    local domain_names=()
    while IFS= read -r d; do
        domain_names+=("$d")
    done < <(echo "$ALL_RESULTS" | jq -r '.Domains | keys[]')

    local score_rows=""
    local findings_by_domain=""
    local gates_rows=""
    local blocked_detail=""
    local total_findings=0 gate_passes=0 gate_total=0 gate_blocks=0

    for d in "${domain_names[@]}"; do
        echo "  Auditing '$d'..." >&2

        # audit
        local audit_args
        audit_args=$(jq -nc --arg d "$d" '{domain: $d}')
        local audit_result
        audit_result=$(invoke_mcp_tool "audit" "$audit_args")

        local overall_score=null
        local findings='[]'
        if [ -n "$audit_result" ]; then
            overall_score=$(echo "$audit_result" | jq -r '.score // null')
            findings=$(echo "$audit_result" | jq -c '.findings // []')
            local fc
            fc=$(echo "$findings" | jq 'length')
            total_findings=$((total_findings + fc))
        fi

        # Stage reports
        local det_score="--" sec_score="--" doc_score="--" cd_score="--"
        for stage in deterministic section document cross_domain; do
            local report_args
            report_args=$(jq -nc --arg d "$d" --arg stage "$stage" '{domain: $d, stage: $stage}')
            local report_result
            report_result=$(invoke_mcp_tool "get_audit_report" "$report_args")
            if [ -n "$report_result" ]; then
                local s
                s=$(echo "$report_result" | jq -r '.score // "--"')
                case "$stage" in
                    deterministic) det_score=$s ;;
                    section) sec_score=$s ;;
                    document) doc_score=$s ;;
                    cross_domain) cd_score=$s ;;
                esac
            fi
        done

        score_rows+="| $d | $overall_score | $det_score | $sec_score | $doc_score | $cd_score |"$'\n'

        # Findings
        local fc
        fc=$(echo "$findings" | jq 'length')
        if [ "$fc" -gt 0 ]; then
            findings_by_domain+="### $d ($fc findings)"$'\n\n'
            findings_by_domain+="| Check ID | Severity | Message |"$'\n'
            findings_by_domain+="|----------|----------|---------|"$'\n'
            local fi=0
            for fi in $(seq 0 $((fc - 1))); do
                [ "$fi" -ge 20 ] && { findings_by_domain+="| ... | ... | _($((fc - 20)) more)_ |"$'\n'; break; }
                local sev msg
                sev=$(echo "$findings" | jq -r ".[$fi].severity // \"--\"" | sed 's/|/\\|/g')
                msg=$(echo "$findings" | jq -r ".[$fi].message // \"--\"" | sed 's/|/\\|/g')
                msg="${msg:0:80}"
                findings_by_domain+="| $(echo "$findings" | jq -r ".[$fi].check_id // \"--\"") | $sev | $msg |"$'\n'
            done
            findings_by_domain+=$'\n'
        else
            findings_by_domain+="### $d"$'\n\n'"No findings"$'\n\n'
        fi

        # Gates
        local det_gate="⚠️" sec_gate="⚠️" doc_gate="⚠️" cd_gate="⚠️"
        for stage in deterministic section document cross_domain; do
            local gate_args
            gate_args=$(jq -nc --arg stage "$stage" '{stage: $stage}')
            local gate_result
            gate_result=$(invoke_mcp_tool "check_gate" "$gate_args")
            gate_total=$((gate_total + 1))
            if [ -n "$gate_result" ]; then
                local blocked
                blocked=$(echo "$gate_result" | jq -r '.blocked // false')
                if [ "$blocked" = "false" ]; then
                    gate_passes=$((gate_passes + 1))
                    case "$stage" in deterministic) det_gate="✅" ;; section) sec_gate="✅" ;; document) doc_gate="✅" ;; cross_domain) cd_gate="✅" ;; esac
                elif [ "$blocked" = "true" ]; then
                    gate_blocks=$((gate_blocks + 1))
                    local reason
                    reason=$(echo "$gate_result" | jq -r '.reason // "blocked"')
                    case "$stage" in deterministic) det_gate="❌" ;; section) sec_gate="❌" ;; document) doc_gate="❌" ;; cross_domain) cd_gate="❌" ;; esac
                    blocked_detail+="| $d | $stage | $reason |"$'\n'
                fi
            fi
        done
        gates_rows+="| $d | $det_gate | $sec_gate | $doc_gate | $cd_gate |"$'\n'
    done

    checks=$(echo "$checks" | jq --argjson dc "${#domain_names[@]}" \
        '. += [{"Name": "Domain audits", "Status": "pass", "Detail": "\($dc) domains audited"}]')
    checks=$(echo "$checks" | jq --argjson gp "$gate_passes" --argjson gt "$gate_total" \
        '. += [{"Name": "Stage gates", "Status": (if $gp == $gt then "pass" else "warn" end), "Detail": "\($gp)/\($gt) passed"}]')

    [ -z "$blocked_detail" ] && blocked_detail="No blocked gates"$'\n'

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "06-audit-findings")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="✅ PASS"
    [ "$gate_blocks" -gt 0 ] && status="⚠️ PARTIAL"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "06-audit-findings")

    [ -z "$score_rows" ] && score_rows="| -- | -- | -- | -- | -- | -- |"$'\n'
    [ -z "$gates_rows" ] && gates_rows="| -- | -- | -- | -- | -- |"$'\n'

    # Score: gate pass rate (60%) + avg audit score (40%)
    local score=0
    if [ "$gate_total" -gt 0 ]; then
        local gate_score=$((gate_passes * 60 / gate_total))
        score=$gate_score
    fi

    local analysis
    analysis=$(gen_phase_analysis "06-audit-findings" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "06-audit-findings" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"06-audit-findings\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg AUDIT_SCORES_TABLE "$score_rows" \
        --arg FINDINGS_BY_DOMAIN "$findings_by_domain" \
        --arg GATES_TABLE "$gates_rows" \
        --arg BLOCKED_GATES_DETAIL "$blocked_detail" \
        --arg DOMAIN_COUNT "${#domain_names[@]}" \
        --arg TOTAL_FINDINGS "$total_findings" \
        --arg GATE_PASSES "$gate_passes" \
        --arg GATE_TOTAL "$gate_total" \
        --arg GATE_BLOCKS "$gate_blocks" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          AUDIT_SCORES_TABLE: $AUDIT_SCORES_TABLE,
          FINDINGS_BY_DOMAIN: $FINDINGS_BY_DOMAIN,
          GATES_TABLE: $GATES_TABLE,
          BLOCKED_GATES_DETAIL: $BLOCKED_GATES_DETAIL,
          DOMAIN_COUNT: $DOMAIN_COUNT, TOTAL_FINDINGS: $TOTAL_FINDINGS,
          GATE_PASSES: $GATE_PASSES, GATE_TOTAL: $GATE_TOTAL,
          GATE_BLOCKS: $GATE_BLOCKS,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE}')
    write_report "06-audit-findings.md" "06-audit-findings.md" "$report_vals" > /dev/null

    PHASE_RESULTS["06-audit-findings"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "06-audit-findings.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status ($total_findings findings, $gate_passes/$gate_total gates)"
}

# ─── Phase 9: Coverage Gaps ────────────────────────────────────────────────────

phase_9_gaps() {
    CURRENT_PHASE="07-coverage-gaps"
    echo "Phase 9: Coverage Gaps..."
    local start
    start=$(date +%s)
    local checks='[]'

    local missing_knowledge_rows="" empty_sections_rows="" low_quality_rows="" required_missing_rows=""
    local missing_kn_count=0 empty_count=0 low_q_count=0 req_missing_count=0

    # Check knowledge coverage
    local pairs_json
    pairs_json=$(echo "$ALL_RESULTS" | jq -c '[.Domains | to_entries[] |
        .key as $d |
        .value.sectionTypes | to_entries[] |
        {domain: $d, type: .key}]')
    local pc
    pc=$(echo "$pairs_json" | jq 'length')

    for pi in $(seq 0 $((pc - 1))); do
        local pair
        pair=$(echo "$pairs_json" | jq -c ".[$pi]")
        local d t
        d=$(echo "$pair" | jq -r '.domain')
        t=$(echo "$pair" | jq -r '.type')

        if ! $NO_AUDIT; then
            local kn_args
            kn_args=$(jq -nc --arg d "$d" --arg t "$t" '{domain: $d, section_type: $t}')
            local kn_result
            kn_result=$(invoke_mcp_tool "get_audit_knowledge" "$kn_args" "true")
            if [ -z "$kn_result" ] || [ "$(echo "$kn_result" | jq -r '.content // ""')" = "" ]; then
                missing_knowledge_rows+="| $d | $t | ❌ Missing |"$'\n'
                missing_kn_count=$((missing_kn_count + 1))
            fi
        fi
    done

    # Check for empty sections and low quality docs
    local all_docs_json
    all_docs_json=$(echo "$ALL_RESULTS" | jq -c '.AllDocs // []')
    local adc
    adc=$(echo "$all_docs_json" | jq 'length')

    for adi in $(seq 0 $((adc - 1))); do
        local item
        item=$(echo "$all_docs_json" | jq -c ".[$adi]")
        local adoc domain_name
        adoc=$(echo "$item" | jq -c '.doc')
        domain_name=$(echo "$item" | jq -r '.domain')
        local doc_id doc_title quality body_val
        doc_id=$(echo "$adoc" | jq -r '.id // "?"')
        doc_title=$(echo "$adoc" | jq -r '.title // "?"' | sed 's/|/\\|/g')
        quality=$(echo "$adoc" | jq -c '.quality // null')
        body_val=$(echo "$adoc" | jq -c '[.body | to_entries | first | .value // null] | first')

        local empty_count_doc=0 missing_count_doc=0 coverage=1.0
        if [ "$quality" != "null" ]; then
            empty_count_doc=$(echo "$quality" | jq -r '.empty_section_count // 0')
            missing_count_doc=$(echo "$quality" | jq -r '.missing_section_count // 0')
            coverage=$(echo "$quality" | jq -r '.coverage // 1.0')
        fi

        if [ "$empty_count_doc" -gt 0 ]; then
            local cov_pct
            cov_pct=$(echo "$coverage * 100" | bc 2>/dev/null || echo "$coverage")
            cov_pct="${cov_pct%.*}"
            [ -z "$cov_pct" ] && cov_pct="0"
            low_quality_rows+="| $doc_id | \"$doc_title\" | $domain_name | ${empty_count_doc} empty, coverage ${cov_pct}% |"$'\n'
            low_q_count=$((low_q_count + 1))
        fi

        if [ "$missing_count_doc" -gt 0 ]; then
            required_missing_rows+="| $doc_id | \"$doc_title\" | $domain_name | ${missing_count_doc} missing |"$'\n'
            req_missing_count=$((req_missing_count + 1))
        fi

        if [ "$(echo "$coverage < 0.7 && $coverage > 0" | bc 2>/dev/null || echo 0)" -eq 1 ]; then
            local cov_pct2
            cov_pct2=$(echo "$coverage * 100" | bc 2>/dev/null || echo "$coverage")
            cov_pct2="${cov_pct2%.*}"
            [ -z "$cov_pct2" ] && cov_pct2="0"
            low_quality_rows+="| $doc_id | \"$doc_title\" | $domain_name | coverage ${cov_pct2}% |"$'\n'
            low_q_count=$((low_q_count + 1))
        fi

        # Check for empty sections
        if [ "$body_val" != "null" ]; then
            local sections
            sections=$(echo "$body_val" | jq -c '.sections // []')
            local sc
            sc=$(echo "$sections" | jq 'length')
            for si in $(seq 0 $((sc - 1))); do
                local s_body heading s_type
                s_body=$(echo "$sections" | jq -r ".[$si].body // \"\"")
                heading=$(echo "$sections" | jq -r ".[$si].heading // \"\"" | sed 's/|/\\|/g')
                s_type=$(echo "$sections" | jq -r ".[$si].semantic_type // \"\"")
                if [ -n "$s_body" ] && [ "$(echo "$s_body" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')" = "" ]; then
                    empty_sections_rows+="| $doc_id | $si \"$heading\" | $s_type |"$'\n'
                    empty_count=$((empty_count + 1))
                fi
            done
        fi
    done

    checks=$(echo "$checks" | jq --argjson mkc "$missing_kn_count" \
        '. += [{"Name": "Knowledge coverage", "Status": (if $mkc > 0 then "warn" else "pass" end), "Detail": "\($mkc) missing"}]')
    checks=$(echo "$checks" | jq --argjson ec "$empty_count" \
        '. += [{"Name": "Empty sections", "Status": (if $ec > 0 then "warn" else "pass" end), "Detail": "\($ec) empty"}]')
    checks=$(echo "$checks" | jq --argjson lqc "$low_q_count" \
        '. += [{"Name": "Low quality docs", "Status": (if $lqc > 0 then "warn" else "pass" end), "Detail": "\($lqc) docs"}]')

    [ -z "$missing_knowledge_rows" ] && missing_knowledge_rows="| -- | -- | All covered |"$'\n'
    [ -z "$empty_sections_rows" ] && empty_sections_rows="| -- | -- | -- |"$'\n'
    [ -z "$low_quality_rows" ] && low_quality_rows="| -- | -- | -- | -- |"$'\n'
    [ -z "$required_missing_rows" ] && required_missing_rows="| -- | -- | -- | -- |"$'\n'

    local mk_table="| Domain | Section Type | Status |"$'\n'"|--------|-------------|--------|"$'\n'"$missing_knowledge_rows"
    local es_table="| Doc ID | Section | Type |"$'\n'"|--------|---------|------|"$'\n'"$empty_sections_rows"
    local lq_table="| Doc ID | Title | Domain | Issue |"$'\n'"|--------|-------|--------|-------|"$'\n'"$low_quality_rows"
    local rm_table="| Doc ID | Title | Domain | Missing |"$'\n'"|--------|-------|--------|---------|"$'\n'"$required_missing_rows"

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "07-coverage-gaps")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="✅ PASS"
    [ "$missing_kn_count" -gt 0 ] || [ "$empty_count" -gt 0 ] || [ "$low_q_count" -gt 0 ] && status="⚠️ PARTIAL"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "07-coverage-gaps")

    # Score: lower gaps = better; max penalty 100 pts across gap types
    local gap_total=$((missing_kn_count + empty_count + low_q_count + req_missing_count))
    local score=100
    [ "$gap_total" -gt 0 ] && score=$((score - gap_total * 10))
    [ "$score" -lt 0 ] && score=0

    local analysis
    analysis=$(gen_phase_analysis "07-coverage-gaps" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "07-coverage-gaps" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"07-coverage-gaps\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg MISSING_KNOWLEDGE_TABLE "$mk_table" \
        --arg EMPTY_SECTIONS_TABLE "$es_table" \
        --arg LOW_QUALITY_TABLE "$lq_table" \
        --arg REQUIRED_MISSING_TABLE "$rm_table" \
        --arg MISSING_KNOWLEDGE_COUNT "$missing_kn_count" \
        --arg EMPTY_SECTION_COUNT "$empty_count" \
        --arg LOW_QUALITY_COUNT "$low_q_count" \
        --arg REQUIRED_MISSING_COUNT "$req_missing_count" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          MISSING_KNOWLEDGE_TABLE: $MISSING_KNOWLEDGE_TABLE,
          EMPTY_SECTIONS_TABLE: $EMPTY_SECTIONS_TABLE,
          LOW_QUALITY_TABLE: $LOW_QUALITY_TABLE,
          REQUIRED_MISSING_TABLE: $REQUIRED_MISSING_TABLE,
          MISSING_KNOWLEDGE_COUNT: $MISSING_KNOWLEDGE_COUNT,
          EMPTY_SECTION_COUNT: $EMPTY_SECTION_COUNT,
          LOW_QUALITY_COUNT: $LOW_QUALITY_COUNT,
          REQUIRED_MISSING_COUNT: $REQUIRED_MISSING_COUNT,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE}')
    write_report "07-coverage-gaps.md" "07-coverage-gaps.md" "$report_vals" > /dev/null

    PHASE_RESULTS["07-coverage-gaps"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "07-coverage-gaps.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status ($gap_total gaps)"
}

# ─── Phase 10: Registry State ──────────────────────────────────────────────────

phase_10_registry() {
    CURRENT_PHASE="08-registry-state"
    echo "Phase 10: Registry State..."
    local start
    start=$(date +%s)
    local checks='[]'

    # list_repositories
    local repos_result
    repos_result=$(invoke_mcp_tool "list_repositories" '{"limit": 50}')
    local repo_count=0
    local repos_table=""
    repos_table+="| # | ID | UUID | Status |"$'\n'
    repos_table+="|---|----|------|--------|"$'\n'
    if [ -n "$repos_result" ]; then
        local repos_arr
        repos_arr=$(echo "$repos_result" | jq -c '.repositories // []')
        repo_count=$(echo "$repos_arr" | jq 'length')
        for ri in $(seq 0 $((repo_count - 1))); do
            local r_id r_uuid r_status
            r_id=$(echo "$repos_arr" | jq -r ".[$ri].id // \"--\"")
            r_uuid=$(echo "$repos_arr" | jq -r ".[$ri].uuid // \"--\"")
            r_status=$(echo "$repos_arr" | jq -r ".[$ri].status // \"--\"")
            repos_table+="| $((ri + 1)) | $r_id | $r_uuid | $r_status |"$'\n'
        done
    else
        repos_table+="| -- | -- | -- | -- |"$'\n'
    fi
    checks=$(echo "$checks" | jq --argjson rc "$repo_count" \
        '. += [{"Name": "List repositories", "Status": "pass", "Detail": "\($rc) repos"}]')

    # resolve_dependencies
    local deps_result
    deps_result=$(invoke_mcp_tool "resolve_dependencies" "{}")
    local dep_count=0 unresolved_count=0
    local deps_table=""
    deps_table+="| Name | Path | Available | Required |"$'\n'
    deps_table+="|------|------|-----------|----------|"$'\n'
    if [ -n "$deps_result" ]; then
        local deps_arr
        deps_arr=$(echo "$deps_result" | jq -c '.dependencies // []')
        dep_count=$(echo "$deps_arr" | jq 'length')
        for di in $(seq 0 $((dep_count - 1))); do
            local dep_name dep_path dep_avail dep_req
            dep_name=$(echo "$deps_arr" | jq -r ".[$di].name // \"--\"")
            dep_path=$(echo "$deps_arr" | jq -r ".[$di].path // \"--\"")
            dep_avail=$(echo "$deps_arr" | jq -r 'if .['"$di"'].available then "✅" else "❌" end')
            dep_req=$(echo "$deps_arr" | jq -r 'if .['"$di"'].required then "yes" else "no" end')
            deps_table+="| $dep_name | $dep_path | $dep_avail | $dep_req |"$'\n'
            local avail_bool
            avail_bool=$(echo "$deps_arr" | jq -r ".[$di].available // false")
            [ "$avail_bool" = "false" ] && unresolved_count=$((unresolved_count + 1))
        done
    else
        deps_table+="| -- | -- | -- | -- |"$'\n'
    fi
    checks=$(echo "$checks" | jq --argjson dc "$dep_count" --argjson uc "$unresolved_count" \
        '. += [{"Name": "Resolve dependencies", "Status": "pass", "Detail": "\($dc) deps, \($uc) unresolved"}]')

    # workspace_status
    local ws_result
    ws_result=$(invoke_mcp_tool "workspace_status" "{}")
    local ws_table=""
    ws_table+="| # | ID | UUID | Status |"$'\n'
    ws_table+="|---|----|------|--------|"$'\n'
    if [ -n "$ws_result" ]; then
        local ws_repos
        ws_repos=$(echo "$ws_result" | jq -c '.repositories // []')
        local wrc
        wrc=$(echo "$ws_repos" | jq 'length')
        for wi in $(seq 0 $((wrc - 1))); do
            local wid wuuid wstat
            wid=$(echo "$ws_repos" | jq -r ".[$wi].id // \"--\"")
            wuuid=$(echo "$ws_repos" | jq -r ".[$wi].uuid // \"--\"")
            wstat=$(echo "$ws_repos" | jq -r ".[$wi].status // \"--\"")
            ws_table+="| $((wi + 1)) | $wid | $wuuid | $wstat |"$'\n'
        done
    else
        ws_table+="| -- | -- | -- | -- |"$'\n'
    fi

    # repository_status
    local rs_result
    rs_result=$(invoke_mcp_tool "repository_status" '{"limit": 50}')
    local rs_table=""
    rs_table+="| # | ID | UUID | Status |"$'\n'
    rs_table+="|---|----|------|--------|"$'\n'
    if [ -n "$rs_result" ]; then
        local rs_repos
        rs_repos=$(echo "$rs_result" | jq -c '.repositories // []')
        local rsc
        rsc=$(echo "$rs_repos" | jq 'length')
        for ri in $(seq 0 $((rsc - 1))); do
            local rrid rruuid rrstat
            rrid=$(echo "$rs_repos" | jq -r ".[$ri].id // \"--\"")
            rruuid=$(echo "$rs_repos" | jq -r ".[$ri].uuid // \"--\"")
            rrstat=$(echo "$rs_repos" | jq -r ".[$ri].status // \"--\"")
            rs_table+="| $((ri + 1)) | $rrid | $rruuid | $rrstat |"$'\n'
        done
    else
        rs_table+="| -- | -- | -- | -- |"$'\n'
    fi

    # synchronize_repository
    local sync_result
    sync_result=$(invoke_mcp_tool "synchronize_repository" "{}")
    checks=$(echo "$checks" | jq --arg r "$([ -n "$sync_result" ] && echo true || echo false)" \
        '. += [{"Name": "Synchronize repository", "Status": (if $r == "true" then "pass" else "warn" end), "Detail": "done"}]')

    # Write-tool smoke tests
    local write_tool_lines=""
    write_tool_lines+="| Tool | Input | Expected | Result |"$'\n'
    write_tool_lines+="|------|-------|----------|--------|"$'\n'
    local write_pass=0 write_total=0

    # store_section_report with empty
    write_total=$((write_total + 1))
    local sr_result
    sr_result=$(invoke_mcp_tool "store_section_report" '{"report_json":{}}' "true")
    if [ -z "$sr_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| store_section_report | \`{}\` | reject bad input | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| store_section_report | \`{}\` | reject bad input | ❌ accepted |"$'\n'
    fi

    # store_document_report with empty
    write_total=$((write_total + 1))
    local dr_result
    dr_result=$(invoke_mcp_tool "store_document_report" '{"report_json":{}}' "true")
    if [ -z "$dr_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| store_document_report | \`{}\` | reject bad input | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| store_document_report | \`{}\` | reject bad input | ❌ accepted |"$'\n'
    fi

    # store_cross_domain_report with empty
    write_total=$((write_total + 1))
    local cr_result
    cr_result=$(invoke_mcp_tool "store_cross_domain_report" '{"report_json":{}}' "true")
    if [ -z "$cr_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| store_cross_domain_report | \`{}\` | reject bad input | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| store_cross_domain_report | \`{}\` | reject bad input | ❌ accepted |"$'\n'
    fi

    # update_finding_status with invalid
    write_total=$((write_total + 1))
    local uf_result
    uf_result=$(invoke_mcp_tool "update_finding_status" '{"report_id":0,"criterion_id":"","status":"invalid"}' "true")
    if [ -z "$uf_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| update_finding_status | invalid data | reject bad input | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| update_finding_status | invalid data | reject bad input | ❌ accepted |"$'\n'
    fi

    # register_repository with empty manifest
    write_total=$((write_total + 1))
    local rg_result
    rg_result=$(invoke_mcp_tool "register_repository" '{"manifest":"{}"}' "true")
    if [ -z "$rg_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| register_repository | \`{}\` manifest | reject bad input | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| register_repository | \`{}\` manifest | reject bad input | ❌ accepted |"$'\n'
    fi

    # unregister_repository with bogus UUID
    write_total=$((write_total + 1))
    local ur_result
    ur_result=$(invoke_mcp_tool "unregister_repository" '{"uuid":"00000000-0000-0000-0000-000000000000"}' "true")
    if [ -z "$ur_result" ]; then
        write_pass=$((write_pass + 1))
        write_tool_lines+="| unregister_repository | bogus UUID | reject not-found | ✅ rejected |"$'\n'
    else
        write_tool_lines+="| unregister_repository | bogus UUID | reject not-found | ❌ accepted |"$'\n'
    fi

    local ws_status="pass"
    [ "$write_pass" -ne "$write_total" ] && ws_status="warn"
    checks=$(echo "$checks" | jq --argjson wp "$write_pass" --argjson wt "$write_total" \
        '. += [{"Name": "Write-tool validation", "Status": "'"$ws_status"'", "Detail": "\($wp)/\($wt) pass"}]')

    local end
    end=$(date +%s)
    local duration=$((end - start))
    local errors_json
    errors_json=$(get_phase_errors_json "08-registry-state")
    local error_count
    error_count=$(echo "$errors_json" | jq 'length')

    local status="✅ PASS"
    [ "$write_pass" -ne "$write_total" ] && status="⚠️ PARTIAL"

    local checks_table
    checks_table=$(get_checks_table "$checks")
    local errors_table
    errors_table=$(get_errors_table "08-registry-state")

    # Score: write-tool pass rate (70%) + dependencies resolved (30%)
    local score=0
    local sub_score=$((write_pass * 70 / (write_total > 0 ? write_total : 1)))
    local dep_score=0
    local dep_total=$((dep_count + unresolved_count))
    [ "$dep_total" -gt 0 ] && dep_score=$((dep_count * 30 / dep_total))
    score=$((sub_score + dep_score))

    local analysis
    analysis=$(gen_phase_analysis "08-registry-state" "$checks")
    local recommendations
    recommendations=$(gen_phase_recs "08-registry-state" "$checks")
    local prev_score
    prev_score=$(get_prev_metric ".phase_scores[] | select(.phase == \"08-registry-state\") | .score // \"\"")
    local trend
    trend=$(trend_between "$score" "$prev_score")

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg DURATION "$duration" \
        --arg STATUS "$status" \
        --arg CHECKS_TABLE "$checks_table" \
        --arg ERRORS_TABLE "$errors_table" \
        --arg REPOS_TABLE "$repos_table" \
        --arg DEPS_TABLE "$deps_table" \
        --arg WORKSPACE_TABLE "$ws_table" \
        --arg REPO_STATUS_TABLE "$rs_table" \
        --arg WRITE_TOOL_TABLE "$write_tool_lines" \
        --arg REPO_COUNT "$repo_count" \
        --arg DEP_COUNT "$dep_count" \
        --arg UNRESOLVED_COUNT "$unresolved_count" \
        --arg WRITE_PASS "$write_pass" \
        --arg WRITE_TOTAL "$write_total" \
        --arg SCORE "$score" \
        --arg TREND "$trend" \
        --arg ANALYSIS "$analysis" \
        --arg RECOMMENDATIONS "$recommendations" \
        --arg PREV_SCORE "${prev_score:-}" \
        '{TIMESTAMP: $TIMESTAMP, DURATION: $DURATION, STATUS: $STATUS,
          CHECKS_TABLE: $CHECKS_TABLE, ERRORS_TABLE: $ERRORS_TABLE,
          REPOS_TABLE: $REPOS_TABLE, DEPS_TABLE: $DEPS_TABLE,
          WORKSPACE_TABLE: $WORKSPACE_TABLE,
          REPO_STATUS_TABLE: $REPO_STATUS_TABLE,
          WRITE_TOOL_TABLE: $WRITE_TOOL_TABLE,
          REPO_COUNT: $REPO_COUNT, DEP_COUNT: $DEP_COUNT,
          UNRESOLVED_COUNT: $UNRESOLVED_COUNT,
          WRITE_PASS: $WRITE_PASS, WRITE_TOTAL: $WRITE_TOTAL,
          SCORE: $SCORE, TREND: $TREND,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS,
          PREV_SCORE: $PREV_SCORE}')
    write_report "08-registry-state.md" "08-registry-state.md" "$report_vals" > /dev/null

    PHASE_RESULTS["08-registry-state"]=$(jq -n \
        --arg status "$status" \
        --argjson errors "$error_count" \
        --arg duration "$duration" \
        --arg report "08-registry-state.md" \
        --argjson score "$score" \
        '{Status: $status, Errors: $errors, Duration: $duration, ReportFile: $report, Score: $score}')
    echo "  → Score: $score/100 $trend — $status ($write_pass/$write_total write-tool)"
}

# ─── Phase 11: Summary ─────────────────────────────────────────────────────────

phase_11_summary() {
    CURRENT_PHASE="00-summary"
    echo "Phase 11: Summary..."
    local start
    start=$(date +%s)

    local phase_order=(01-tool-health 02-domain-catalog 03-document-audit 04-section-integrity 05-search-results 06-audit-findings 07-coverage-gaps 08-registry-state)
    local phase_rows=""
    local failed_phases=""
    local total_errors=0 total_duration=0

    for key in "${phase_order[@]}"; do
        local pr="${PHASE_RESULTS[$key]:-}"
        [ -z "$pr" ] && continue
        local report_file status errors duration
        report_file=$(echo "$pr" | jq -r '.ReportFile // ""')
        [ -z "$report_file" ] && continue
        status=$(echo "$pr" | jq -r '.Status // "?"')
        errors=$(echo "$pr" | jq -r '.Errors // 0')
        duration=$(echo "$pr" | jq -r '.Duration // 0')
        local name="${report_file%.md}"
        phase_rows+="| $name | $report_file | $status | $errors | ${duration}s |"$'\n'
        total_errors=$((total_errors + errors))
        total_duration=$((total_duration + duration))
        if echo "$status" | grep -qE 'FAIL|PARTIAL'; then
            failed_phases+="- **$report_file**: $status ($errors errors)"$'\n'
        fi
    done

    local tool_count domain_count doc_count sect_count sect_type_count
    tool_count=$(echo "$ALL_RESULTS" | jq -r '(.Tools | length) // 0')
    domain_count=$(echo "$ALL_RESULTS" | jq -r '(.Domains | length) // 0')
    doc_count=$(echo "$ALL_RESULTS" | jq -r '.TotalDocs // 0')
    sect_count=$(echo "$ALL_RESULTS" | jq -r '.TotalSections // 0')
    sect_type_count=$(echo "$ALL_RESULTS" | jq -r '(.SectionsByType | length) // 0')

    local overall_status="✅ CLEAN"
    [ "$total_errors" -gt 0 ] && overall_status="⚠️ WITH ERRORS"

    # Compute total score = floor of average of all phase scores
    local score_sum=0 score_count=0
    local -A phase_scores_arr
    for key in "${phase_order[@]}"; do
        local pr="${PHASE_RESULTS[$key]:-}"
        [ -z "$pr" ] && continue
        local ps
        ps=$(echo "$pr" | jq -r '.Score // empty')
        [ -n "$ps" ] && [ "$ps" != "null" ] && score_sum=$((score_sum + ps)) && score_count=$((score_count + 1))
    done
    local total_score=0
    [ "$score_count" -gt 0 ] && total_score=$((score_sum / score_count))

    [ -z "$phase_rows" ] && phase_rows="| -- | -- | -- | -- | -- |"$'\n'
    [ -z "$failed_phases" ] && failed_phases="✅ All phases passed"$'\n'

    local archive_str="${ARCHIVE_PATH:-No previous run}"

    # Load previous metrics for historical comparison
    PREV_METRICS=$(load_previous_metrics "$ARCHIVE_DIR")
    local prev_total_score
    prev_total_score=$(echo "$PREV_METRICS" | jq -r '.total_score // ""')
    local total_trend
    total_trend=$(trend_between "$total_score" "$prev_total_score")

    # Generate overall analysis and recommendations
    local overall_analysis=""
    local overall_recs=""
    if [ "$score_count" -gt 0 ]; then
        local min_score=100 min_phase="" max_score=0 max_phase=""
        for key in "${phase_order[@]}"; do
            local pr="${PHASE_RESULTS[$key]:-}"
            [ -z "$pr" ] && continue
            local ps rpt
            ps=$(echo "$pr" | jq -r '.Score // 0')
            rpt=$(echo "$pr" | jq -r '.ReportFile // ""')
            [ "${ps:-0}" -lt "$min_score" ] && min_score=$ps && min_phase=$rpt
            [ "${ps:-0}" -gt "$max_score" ] && max_score=$ps && max_phase=$rpt
        done
        overall_analysis="Total score **$total_score/100** across $score_count phases. "
        [ -n "$min_phase" ] && overall_analysis+="Lowest: **$min_phase** ($min_score). "
        [ -n "$max_phase" ] && overall_analysis+="Highest: **$max_phase** ($max_score). "
        [ "$total_errors" -gt 0 ] && overall_analysis+="$total_errors total errors detected."
        if [ -n "$prev_total_score" ]; then
            overall_analysis+=" Previous total: **$prev_total_score/100**."
        fi

        local low_phases=0
        for key in "${phase_order[@]}"; do
            local pr="${PHASE_RESULTS[$key]:-}"
            [ -z "$pr" ] && continue
            local ps
            ps=$(echo "$pr" | jq -r '.Score // 0')
            [ "${ps:-0}" -lt 60 ] && low_phases=$((low_phases + 1))
        done
        overall_recs=""
        [ "$low_phases" -gt 0 ] && overall_recs+="- Investigate $low_phases phase(s) scoring below 60."$'\n'
        [ "$total_errors" -gt 0 ] && overall_recs+="- Address $total_errors error(s) across phases."$'\n'
        [ -z "$overall_recs" ] && overall_recs="- All phases performing well. Maintain current practices."$'\n'
    fi

    local report_vals
    report_vals=$(jq -n \
        --arg TIMESTAMP "$(date '+%Y-%m-%d %H:%M:%S')" \
        --arg TOTAL_DURATION "$total_duration" \
        --arg OVERALL_STATUS "$overall_status" \
        --arg PHASE_RESULTS_ROWS "$phase_rows" \
        --arg TOOL_COUNT "$tool_count" \
        --arg DOMAIN_COUNT "$domain_count" \
        --arg DOCUMENT_COUNT "$doc_count" \
        --arg SECTION_COUNT "$sect_count" \
        --arg SECTION_TYPE_COUNT "$sect_type_count" \
        --arg TOTAL_CALLS "$TOTAL_CALLS" \
        --arg TOTAL_ERRORS "$total_errors" \
        --arg FAILED_PHASES "$failed_phases" \
        --arg ARCHIVE_PATH "$archive_str" \
        --arg SCORE "$total_score" \
        --arg TREND "$total_trend" \
        --arg PREV_SCORE "${prev_total_score:-}" \
        --arg ANALYSIS "$overall_analysis" \
        --arg RECOMMENDATIONS "$overall_recs" \
        '{TIMESTAMP: $TIMESTAMP, TOTAL_DURATION: $TOTAL_DURATION,
          OVERALL_STATUS: $OVERALL_STATUS,
          PHASE_RESULTS_ROWS: $PHASE_RESULTS_ROWS,
          TOOL_COUNT: $TOOL_COUNT, DOMAIN_COUNT: $DOMAIN_COUNT,
          DOCUMENT_COUNT: $DOCUMENT_COUNT, SECTION_COUNT: $SECTION_COUNT,
          SECTION_TYPE_COUNT: $SECTION_TYPE_COUNT,
          TOTAL_CALLS: $TOTAL_CALLS, TOTAL_ERRORS: $TOTAL_ERRORS,
          FAILED_PHASES: $FAILED_PHASES, ARCHIVE_PATH: $ARCHIVE_PATH,
          SCORE: $SCORE, TREND: $TREND,
          PREV_SCORE: $PREV_SCORE,
          ANALYSIS: $ANALYSIS, RECOMMENDATIONS: $RECOMMENDATIONS}')
    write_report "00-summary.md" "00-summary.md" "$report_vals" > /dev/null

    # Build and save metrics JSON for next run
    local phase_scores_json
    phase_scores_json=$(build_phase_scores_json)
    save_metrics_json "$total_score" "$phase_scores_json"

    PHASE_RESULTS["00-summary"]=$(jq -n \
        --arg status "✅ DONE" \
        --argjson errors "$total_errors" \
        --argjson score "$total_score" \
        --arg duration "$(( $(date +%s) - start ))" \
        --arg report "00-summary.md" \
        '{Status: $status, Errors: $errors, Score: $score, Duration: $duration, ReportFile: $report}')
    echo "  → Total Score: $total_score/100 $total_trend — Done"
}

# ─── Main ──────────────────────────────────────────────────────────────────────

echo ""
echo "╔═══════════════════════════════════════════╗"
echo "║    Samgraha MCP Discovery                ║"
echo "║    $(date '+%Y-%m-%d %H:%M:%S')              ║"
echo "╚═══════════════════════════════════════════╝"
echo ""

MAIN_START=$(date +%s)

phase_1_bootstrap
phase_2_domain_scan
phase_3_doc_discover
phase_4_doc_verify

ts=$(echo "$ALL_RESULTS" | jq -r '.TotalSections // 0')
if [ "$ts" -gt 0 ]; then
    phase_5_cross_section
    phase_6_section_verify
else
    echo "Phase 5-6: No sections to verify (skipping)"
fi

phase_7_search
phase_8_audit
phase_9_gaps
phase_10_registry
phase_11_summary

MAIN_END=$(date +%s)
MAIN_DURATION=$((MAIN_END - MAIN_START))

echo ""
echo "╔═══════════════════════════════════════════╗"
echo "║  Complete: ${MAIN_DURATION}s                          ║"
echo "║  Reports: $LATEST_DIR"
echo "╚═══════════════════════════════════════════╝"
echo ""

if $PASS_THRU; then
    echo ""
    echo "Report files:"
    for f in "$LATEST_DIR"/*.md; do
        echo "  $f"
    done
fi
