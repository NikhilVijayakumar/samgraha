# MCP Capability Tools

## Purpose

Reference documentation for the MCP tools that expose capability dispatch — the mechanism by which samgraha delegates audit, calculation, reporting, scaffolding, and plan generation to system-provided scripts. There is no built-in fallback for any of them — a missing script fails with a clear error naming the capability and the guide to write one.

## Tools

### run_system_validate / run_system_calculate / run_system_scaffold

Runs a system's `validate`/`calculate`/`scaffold` script.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_name` | string | no | Which registered system's standard to attribute this run to. Defaults to whichever system is `is_default`. |
| `repo_root` | string | no | Repository root. Defaults to the session root. |
| `input_json` | string | no | Path to a JSON file to pass via `--in`. Omit for no input. |
| `phase_id` | string | no | Phase id from the system's init plan, for prerequisite gating. |
| `timeout_secs` | integer | no | Seconds before the script is killed. |
| `repo_path` | string | no | Absolute path to a different local repository to target. |

**Returns:** the standard envelope — `{status, message, written}`, plus
capability-specific fields (`calculate` adds `final_score`/`band`/
`breakdown`; `scaffold` adds `created`/`skipped`) — or a blocked-
precondition response if `phase_id` was given and its prerequisites
aren't met.

---

### run_system_report / run_system_plan_generation

Runs a system's `report`/`plan-generation` script. Same params as above,
plus:

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `target` | string | no | Render target (`report` only — e.g. `"document"`, `"checklist"`, `"spec"`). |

**Returns:** the standard envelope — `{status, message, written}`. The
rendered document itself is written directly to whatever path the script
was given; `written` names it. There is no `plan`/`report_path` field —
both capabilities return the same shape.

---

### run_system_script

Generic catch-all — the same dispatch every dedicated tool above uses, with the capability named explicitly.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `capability` | string | yes | One of: `validate`, `calculate`, `report`, `scaffold`, `plan-generation` (hyphenated), `init`. |
| `system_name` | string | no | Same as above. |
| `repo_root` | string | no | Same as above. |
| `input_json` | string | no | Same as above. |
| `phase_id` | string | no | Same as above. |
| `timeout_secs` | integer | no | Same as above. |
| `repo_path` | string | no | Same as above. |

**Returns:** same envelope as the matching dedicated tool.

---

### store_system_plan / get_system_plan

Write/read a system's `init` plan. `store_system_plan` splits the incoming
JSON into the normalized `workflow_use_cases`/`workflow_phases`/
`workflow_phase_dependencies` tables — not stored as one JSON blob.
`get_system_plan` reconstructs the same JSON shape by querying those
tables back.

**`store_system_plan` parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_name` | string | yes | Must match the `system` field inside `plan_json`. |
| `plan_json` | string | yes | The full `init` plan output (§8.4 shape) as a JSON string. |

**Returns:** `{stored: true, system_name, use_cases, total_phases}`

**`get_system_plan` parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_name` | string | yes | Which system's plan to fetch. |
| `repo_path` | string | no | Absolute path to a different local repository to target. |

**Returns:** `{system_name, plan}` — `plan` is the reconstructed `init`
JSON, or `null` if nothing's stored yet.

---

### store_plan_generation_input / get_plan_generation_input

Write/read the semantic determination a `plan-generation` workflow's first
stage produces (§7.2/§8.3 — "what needs generating for this repo, in this
workflow"). This is LLM-produced content, not something samgraha
generates itself.

**`store_plan_generation_input` parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_name` | string | yes | |
| `workflow_id` | string | yes | Matches a use-case id in the system's init plan. |
| `domain_key` | string | no | `null`/omitted = plan-level, not domain-specific. |
| `instance_key` | string | no | Only for multi-instance domains (e.g. a specific `feature`). |
| `input_json` | string | yes | The semantic determination's own output. |

**Returns:** `{stored: true, system_name, workflow_id}` — re-storing the
same key upserts; the previous value shifts into `previous_input_json`
(one-generation-back hedge), it isn't lost outright.

**`get_plan_generation_input` parameters:** same identifying params,
minus `input_json`.

**Returns:** `{input_json, previous_input_json}`, or `{input: null}`.

---

## Discovery

Every tool above resolves its script through four tiers, in order —
**Override** (`samgraha.toml`'s `check_overrides`) → **RepoScript**
(`{repo}/scripts/`) → **LocalScript** (`.samgraha/scripts/`) →
**GlobalScript** (shipped next to the MCP binary). There is no fifth tier.

## Prerequisite Gating

All `run_system_*` tools accept an optional `phase_id`. When provided, the
dispatch system checks:

- Every dependency phase (`workflow_phase_dependencies`) has a matching
  entry in `script_runs`
- That entry hasn't expired (`expiry_rule_json`/`expires_at`/
  `head_commit_at_run`)

If any dependency is missing or expired, the tool returns a blocked-
precondition response (`{blocked: true, reason, phase_id, message,
how_to_run}`) instead of running the script.

## Error Handling

- **Script not found**: returns an error naming the capability and
  pointing at the knowledge-system-author-guide — not a discovery-tier
  dump, and never a silent fallback
- **Script failure/timeout/crash**: returns error status with the
  script's own message
- **Prerequisite blocked**: returns the blocked-precondition JSON above,
  the script never runs

## Related

- [Capability Dispatch Feature](../../feature/capability-dispatch.md)
- [Capability Scripts](../concepts/scripts.md)
- [Knowledge Systems](../concepts/systems.md)
