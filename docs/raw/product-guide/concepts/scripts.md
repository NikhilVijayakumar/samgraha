# Capability Scripts

## Purpose

Capability scripts are executable files provided by knowledge systems that implement one or more of samgraha's six canonical capabilities: `validate`, `calculate`, `report`, `scaffold`, `plan-generation` (hyphenated when serialized — `plan_generation` internally in Rust/filenames), and `init`.

## What Scripts Do

When samgraha needs to audit a document, compute a metric, or generate a report for a domain owned by a knowledge system, it invokes the system's script instead of running built-in logic. The script receives standardized inputs and returns structured outputs.

## The Six Capabilities

| Capability | Input | Output | Purpose |
|---|---|---|---|
| `validate` | Target document + check IDs | Findings with scores | Run audit checks |
| `calculate` | Target + options | Computed metrics | Derive scores or analytics |
| `report` | Target + options | Report file path | Generate human-readable reports |
| `scaffold` | Target + options | Scaffolded files | Generate documentation boilerplate |
| `plan-generation` | Target + options | Rendered plan document | Generate project workflows |
| `init` | System configuration | Initialization status | Set up local system state |

## Script Contract

Every script receives three command-line arguments:

```bash
python scripts/validate.py --repo-root /path/to/repo --in /tmp/input.json --out /tmp/output.json
```

- **`--repo-root`** — the repository root (scripts must stay within this directory)
- **`--in`** — JSON file with target, options, and context
- **`--out`** — path where script writes its JSON result

## Input JSON Schema

```json
{
  "target": {
    "domain": "feature",
    "document_path": "docs/raw/feature/audit-framework.md",
    "check_ids": ["F1", "F2", "F3"]
  },
  "options": {
    "severity_filter": "error"
  },
  "phase_id": "phase-3"
}
```

## Output JSON Schema

```json
{
  "status": "pass",
  "score": 85.0,
  "findings": [
    {
      "criterion_id": "F1",
      "severity": "error",
      "message": "Missing required section",
      "location": {"line": 10}
    }
  ],
  "metadata": {}
}
```

## Script Discovery

Scripts are discovered through four tiers (highest priority first) — by
filesystem probing, not anything declared in `system.yaml`:

1. **Override** — `check_overrides` in `samgraha.toml`
2. **RepoScript** — `{repo}/scripts/<capability>.<ext>`
3. **LocalScript** — `.samgraha/scripts/<capability>.<ext>` (synced copy)
4. **GlobalScript** — scripts shipped next to the MCP binary

There is no fifth tier. A capability with no script at any of the four
fails clearly — samgraha does not fall back to built-in logic for any
capability.

## Writing a Script

See the [Knowledge System Author Guide](../../../knowledge-system-author-guide.md) for complete guidance on writing scripts in any language.

## Related

- [Knowledge Systems](systems.md)
- [System Inheritance](inheritance.md)
- [Capability Dispatch Feature](../../feature/capability-dispatch.md)
