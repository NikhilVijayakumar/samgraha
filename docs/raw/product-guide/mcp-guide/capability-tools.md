# MCP Capability Tools

## Purpose

Reference documentation for the MCP tools that expose capability dispatch — the mechanism by which samgraha delegates audit, calculation, reporting, scaffolding, and plan generation to system-provided scripts.

## Tools

### run_system_validate

Runs a system's `validate` script against a target document.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID (e.g., "kriti") |
| `domain` | string | yes | Documentation domain (e.g., "feature") |
| `target` | object | no | `{document_path, check_ids}` |
| `options` | object | no | Custom options passed to script |
| `phase_id` | string | no | Phase ID for prerequisite gating |

**Returns:** `{status, score, findings[]}`

---

### run_system_calculate

Runs a system's `calculate` script to compute derived metrics.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID |
| `domain` | string | yes | Documentation domain |
| `target` | object | no | Target specification |
| `options` | object | no | Calculation options |

**Returns:** `{status, results}`

---

### run_system_report

Runs a system's `report` script to generate reports.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID |
| `domain` | string | yes | Documentation domain |
| `target` | object | no | Target specification |
| `options` | object | no | Report options |

**Returns:** `{status, report_path}`

---

### run_system_scaffold

Runs a system's `scaffold` script to generate documentation boilerplate.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID |
| `domain` | string | yes | Documentation domain |
| `target` | object | no | Target specification |
| `options` | object | no | Scaffold options |

**Returns:** `{status, scaffolded_files[]}`

---

### run_system_plan_generation

Runs a system's `plan_generation` script to generate phased project plans.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID |
| `domain` | string | no | Documentation domain |
| `options` | object | no | Plan generation options |

**Returns:** `{status, plan}`

---

### run_system_script

Generic catch-all tool for running any capability script.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `system_id` | string | yes | Knowledge system ID |
| `capability` | string | yes | Capability name (validate, calculate, report, scaffold, plan_generation, init) |
| `domain` | string | yes | Documentation domain |
| `target` | object | no | Target specification |
| `options` | object | no | Custom options |

**Returns:** `{status, ...}` (varies by capability)

## Prerequisite Gating

All tools accept an optional `phase_id` parameter. When provided, the dispatch system checks:

- All `depends_on` phases for that phase are completed
- TTL expiry rules are not violated
- Head commit matches (if configured)

If prerequisites fail, the tool returns a blocked-precondition response instead of executing the script.

## Error Handling

- **Script not found**: returns error with discovery tier information
- **Script timeout**: returns error after configured timeout (default 30s)
- **Script crash**: returns error with exit code and stderr
- **Invalid output**: returns error with schema validation details
- **Prerequisite blocked**: returns blocked-precondition JSON

## Related

- [Capability Dispatch Feature](../../feature/capability-dispatch.md)
- [Capability Scripts](../concepts/scripts.md)
- [Knowledge Systems](../concepts/systems.md)
