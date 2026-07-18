# Knowledge System Author Guide

**Purpose**: Everything a system author (e.g. Kriti, or anyone building a
new knowledge system) needs to know to create a system that works with
samgraha's generic script architecture. Covers directory structure,
scripts, templates, workflows, inheritance, and registration.

**Prerequisite reading**: `docs/generic-script-architecture-proposal.md`
(especially §2, §8.1–§8.6) — this document references this proposal's
specs but doesn't repeat them in full.

---

## 1. What Is a System?

A system is a self-contained package that tells samgraha how to work with
a specific kind of documentation. samgraha itself contains **zero domain
knowledge** — it discovers your system's scripts, runs them under a fixed
contract, and stores/relays whatever comes back. Your system provides all
the domain logic as executable scripts.

Example systems: `rust_dev`, `fastapi_dev`, `python_hackathon`,
`eswa_journal`. Each defines its own domains, tiers, scoring formula,
templates, and workflows — samgraha doesn't know or care what any of
those mean.

---

## 2. Directory Structure

```
samgraha/system/{system_name}/
├── system.yaml                          # REQUIRED — identity + inheritance
├── scripts/                             # REQUIRED — capability scripts
│   ├── validate.py                      #   validate capability
│   ├── calculate.py                     #   calculate capability
│   ├── report.py                        #   report capability
│   ├── scaffold.py                      #   scaffold capability
│   ├── plan_generation.py               #   plan-generation capability
│   └── init.py                          #   init capability (self-description)
├── templates/                           # REQUIRED — document templates
│   ├── plan/
│   │   └── PLAN.md.j2                   #   plan document template
│   └── reports/
│       └── domain_report.md.j2          #   per-domain report template
├── documentation-standards/             # REQUIRED — domain definitions
│   ├── 00-domain-relationships.md       #   domain graph (which domains exist, edges)
│   └── {domain-name}.md                 #   one file per domain (sections, rules)
└── audit/                               # OPTIONAL — pre-existing audit rules
    └── {standard}/
        └── audit/
            └── pipelines/
                └── {domain}.yaml        #   YAML-driven audit pipeline
```

### 2.1 What goes where

| Directory/file | Purpose | Required? |
|---|---|---|
| `system.yaml` | System identity, inheritance config | Yes |
| `scripts/` | All capability scripts | Yes |
| `templates/` | Jinja2/Markdown templates for documents | Yes |
| `documentation-standards/` | Domain definitions, section catalogs, rules | Yes |
| `audit/` | YAML-driven audit pipelines (optional, for deterministic checks) | No |

---

## 3. `system.yaml` — System Identity

Every system must have a `system.yaml` at its root. This is the only
file samgraha reads directly from your system — everything else goes
through scripts.

```yaml
# samgraha/system/{system_name}/system.yaml

# Human-readable name (must match the directory name)
name: rust_dev

# What this system does (shown in `list_standards`)
description: "Documentation standards for Rust projects"

# Inheritance (optional) — inherit files from another system
# extends: base_dev

# Files to drop from the inherited system (optional, requires extends)
# drops:
#   - "06-design-standards.md"
#   - "09-feature-design-standards.md"

# Abstract system — can only be used as a base, not directly
# abstract: true
```

### 3.1 Key fields

| Field | Type | Description |
|---|---|---|
| `name` | string | System name, must match directory name |
| `description` | string | Human-readable description |
| `extends` | string | Parent system name (optional) |
| `drops` | list[string] | Files to remove from inherited tree (optional) |
| `abstract` | bool | If true, cannot be used directly — only as a base |

---

## 4. Inheritance — Reducing Duplication

Inheritance lets you create a base system with shared files, then create
child systems that override only what differs. This is the primary
mechanism for reducing duplicate documentation across related systems.

### 4.1 How it works

When you set `extends: base_dev` in your `system.yaml`:

1. samgraha loads all files from `base_dev/`
2. samgraha overlays your system's files on top (overriding matching paths)
3. If you specify `drops`, those files are removed from the merged tree
4. The result is a complete system with your customizations applied

### 4.2 Example: dev class inheritance

```
samgraha/system/
  base_dev/                    # Abstract base — shared across all dev systems
    system.yaml                #   abstract: true
    scripts/
      validate.py              #   Shared validation logic
      calculate.py             #   Shared scoring formula
    documentation-standards/
      00-domain-relationships.md
      01-vision-standards.md
      02-architecture-standards.md
      ...
      16-knowledge-system-standards.md

  rust_dev/                    # Concrete system — extends base_dev
    system.yaml                #   extends: base_dev
    scripts/
      validate.py              #   Override: Rust-specific validation
      # calculate.py inherited from base_dev
    documentation-standards/
      # 06-design-standards.md dropped (Rust doesn't need frontend design)
      # 09-feature-design-standards.md dropped
      # 11-prototype-standards.md dropped

  fastapi_dev/                 # Concrete system — extends base_dev
    system.yaml                #   extends: base_dev
    scripts/
      # All scripts inherited from base_dev
    documentation-standards/
      # 06-design-standards.md dropped
      # 09-feature-design-standards.md dropped
      # 11-prototype-standards.md dropped
```

### 4.3 What can be inherited/overridden

| File type | Inheritable? | Override behavior |
|---|---|---|
| `scripts/*` | Yes | Override individual scripts, inherit the rest |
| `templates/*` | Yes | Override individual templates, inherit the rest |
| `documentation-standards/*` | Yes | Override individual domain files, inherit the rest |
| `system.yaml` | No | Each system has its own |

### 4.4 Inheritance rules

- **Single parent only** — no diamond inheritance
- **Arbitrary depth** — `A extends B extends C extends D` works
- **Circular detection** — samgraha detects and rejects circular chains
- **File-level override** — you only need to provide files that differ
- **Drops are cascading** — dropping a file removes it from the merged tree

---

## 5. Scripts — The Core of Your System

Every capability needs a script. Scripts are executable files (`.py`,
`.sh`, `.ps1`, `.js`) that samgraha discovers and runs under a fixed
contract.

### 5.1 Script contract (all capabilities)

Every script receives:

```
--repo-root <path>    # Root of the repository being operated on
--in <path>           # Path to a JSON input file (may be "{}" if no input)
--out <path>          # Path to write the JSON output envelope
```

Every script must write to `--out`:

```json
{
  "status": "ok" | "error",
  "message": "optional human-readable message",
  "written": ["list/of/files/written/by/script"]
}
```

### 5.2 Capability-specific scripts

#### `validate` — Run validation checks

**Input** (`--in`): The section/document/domain to validate.

**Output** (`--out`): Standard envelope. Additional fields in `output_json`:
```json
{
  "status": "ok",
  "findings": [
    {
      "rule_id": "V001",
      "severity": "error",
      "message": "Missing required section: Architecture",
      "file": "docs/architecture.md",
      "line": null
    }
  ]
}
```

#### `calculate` — Compute scores/ratings

**Input** (`--in`): Raw audit results, scoring rules, score bands.

**Output** (`--out`): Standard envelope. Additional fields in `output_json`:
```json
{
  "status": "ok",
  "final_score": 82,
  "band": "Good",
  "breakdown": {
    "deterministic_document": 90,
    "semantic_section": 74
  }
}
```

The `breakdown` shape is **your system's choice** — samgraha stores it,
never interprets it.

#### `report` — Generate reports

**Input** (`--in`): Assembled audit results, calculate output, stored
semantic commentary.

**Output** (`--out`): Standard envelope. The script writes the actual
report file(s) directly — `written` lists the paths.

```json
{
  "status": "ok",
  "written": ["docs/reports/domain-overview.md"],
  "message": null
}
```

#### `scaffold` — Create file/section skeletons

**Input** (`--in`):
```json
{
  "domain": "feature",
  "instance": "auth",
  "sections": ["overview", "acceptance-criteria"],
  "target_path": "docs/features/auth.md"
}
```

**Output** (`--out`): Standard envelope. Must include `skipped` for
idempotency (§7.4 of the architecture proposal):
```json
{
  "status": "ok",
  "created": ["docs/features/auth.md"],
  "skipped": ["docs/features/auth.md#overview (already exists)"],
  "message": null
}
```

#### `plan-generation` — Determine what needs generating

**Input** (`--in`): Plan template, workflow definition, semantic input.

**Output** (`--out`): Standard envelope. Additional fields in `output_json`:
```json
{
  "status": "ok",
  "determination": "can-proceed",
  "blocked_reasons": [],
  "tasks": [
    {
      "domain": "vision",
      "action": "scaffold",
      "target_path": "docs/vision.md"
    }
  ]
}
```

Or if blocked:
```json
{
  "status": "ok",
  "determination": "blocked",
  "blocked_reasons": [
    "Vision document has not been generated yet"
  ],
  "tasks": []
}
```

#### `init` — Self-description (§8.4)

**Input** (`--in`): `"{}"` (no input needed).

**Output** (`--out`): The full init plan JSON (§8.4 shape):
```json
{
  "system": "rust_dev",
  "use_cases": [
    {
      "id": "new-repo",
      "label": "New repo, no code, no docs",
      "phases": [
        {
          "id": "vision-plan-input",
          "kind": "semantic",
          "description": "Determine what Vision needs, given repo state",
          "depends_on": [],
          "pre_script": null,
          "post_script": null
        },
        {
          "id": "vision-plan-render",
          "kind": "script",
          "script": "scripts/plan_generation.py",
          "depends_on": ["vision-plan-input"],
          "pre_script": null,
          "post_script": null,
          "expiry": null
        },
        {
          "id": "architecture-generate",
          "kind": "script",
          "script": "scripts/scaffold.py",
          "depends_on": ["vision-plan-render"],
          "pre_script": null,
          "post_script": null,
          "expiry": {"type": "ttl", "seconds": 86400}
        }
      ]
    }
  ]
}
```

### 5.3 Script language

Any language that `script_command()` can dispatch:
- `.py` → `python3 script.py ...`
- `.sh` → `sh script.sh ...` (or `.ps1` on Windows)
- `.ps1` → `pwsh script.ps1 ...`
- `.js` → `node script.js ...`
- No extension → run directly as executable

Mix languages freely across a system's scripts — a `rust_dev` system can
have `validate.py` and `calculate.sh` if that's convenient.

### 5.5 Script discovery (5-tier)

samgraha finds scripts through this chain:
1. `check_overrides` in `samgraha.toml` (repo-level override)
2. `{system_dir}/scripts/{capability}.<ext>` (your system's scripts)
3. `.samgraha/scripts/{capability}.<ext>` (local synced copy)
4. `mcp_dir/scripts/{capability}.<ext>` (global defaults)

---

## 6. Templates — Document Rendering

Templates are Jinja2-style files that your `report` and `scaffold`
scripts use to render documents. samgraha never reads templates directly —
your scripts load and render them.

### 6.1 Template locations

```
{system_dir}/templates/
├── plan/
│   └── PLAN.md.j2              # Plan document template
├── reports/
│   ├── domain_report.md.j2     # Per-domain report
│   └── executive_summary.md.j2 # Executive summary
└── scaffold/
    └── section.md.j2           # Section stub template
```

### 6.2 Template variables

Your scripts decide what variables are available. samgraha doesn't
enforce a template schema — your `report` script loads the template and
passes whatever data it needs.

Common pattern:
```python
from jinja2 import Environment, FileSystemLoader

env = Environment(loader=FileSystemLoader("templates/reports"))
template = env.get_template("domain_report.md.j2")

rendered = template.render(
    domain="vision",
    findings=[...],
    score=82,
    band="Good",
)
```

---

## 7. Documentation Standards — Domain Definitions

Each domain your system supports needs a definition file in
`documentation-standards/`. This tells samgraha (and your scripts) what
sections a domain document should have, what rules apply, and how
domains relate to each other.

### 7.1 Domain relationship graph

`00-domain-relationships.md` defines which domains exist and how they
depend on each other:

```markdown
# Domain Relationships

| Domain | Depends On | Description |
|--------|-----------|-------------|
| vision | — | Product vision and problem statement |
| architecture | vision | Technical architecture decisions |
| feature | architecture, vision | Feature specifications |
| implementation | feature | Implementation details |
```

### 7.2 Domain definition files

Each domain gets its own file:

```markdown
# {domain-name}-standards.md

## Sections

| Section | Required | Description |
|---------|----------|-------------|
| Overview | yes | High-level summary |
| Acceptance Criteria | yes | Definition of done |
| Technical Design | no | Implementation approach |

## Rules

### R001: Must have Overview
- Severity: error
- Check: section exists and is non-empty

### R002: Acceptance Criteria must be testable
- Severity: warning
- Check: each criterion starts with a verb (Given/When/Then or similar)
```

---

## 8. Workflows — The Init Plan

Your `init` script returns a workflow (§8.4) that describes all use
cases and their phases. This is the "table of contents" for your system.

### 8.1 Use cases

Every system should define these base use cases (matching §7.1 of the
architecture proposal):

1. **new-repo** — New repo, no code, no docs
2. **new-repo-with-docs** — New repo with some existing documentation
3. **existing-repo-no-docs** — Existing repo, no documentation
4. **existing-repo-impl-only** — Existing repo, implementation only

You can add more use cases specific to your system.

### 8.2 Phase types

| Kind | Description | samgraha behavior |
|------|-------------|-------------------|
| `semantic` | LLM step — caller fills in content via MCP | No script to run; samgraha waits for caller to write result |
| `script` | Deterministic step — samgraha runs your script | Runs the script named in `script` field |

### 8.3 Dependencies

Each phase declares `depends_on` — a list of phase IDs that must complete
first. samgraha checks `script_runs` before allowing execution (§8.6).

```json
{
  "id": "architecture-generate",
  "depends_on": ["vision-plan-render"],
  ...
}
```

If `vision-plan-render` hasn't run (or its output expired), samgraha
returns a blocked response telling the caller what to run first.

### 8.4 Expiry rules

Script phases can declare an `expiry` — how long their output stays
valid:

| Rule | Meaning |
|------|---------|
| `null` | Never expires |
| `{"type": "ttl", "seconds": 86400}` | Valid for 24 hours from when it ran |
| `{"type": "head_commit"}` | Valid until repo HEAD changes |

**Default recommendation**: use `ttl`, not `head_commit` (see §8.5 of
the architecture proposal for why).

---

## 9. Registration — Making Your System Discoverable

Once your system is built, register it with samgraha:

### 9.1 Via MCP tool

```json
{
  "tool": "register_standard",
  "args": {
    "path": "/path/to/samgraha/system/{system_name}",
    "system": "{system_name}"
  }
}
```

### 9.2 Via CLI

```bash
python schema/knowledge-hub/knowledge-hub-loader.py \
  --system {system_name} \
  --knowledge-hub /path/to/samgraha/system/{system_name}
```

### 9.3 Store your init plan

After registration, store your init plan so samgraha can gate
dependencies:

```json
{
  "tool": "store_system_plan",
  "args": {
    "system_name": "{system_name}",
    "plan_json": "{output of your init script}"
  }
}
```

---

## 10. Complete Checklist

Use this checklist to verify your system is complete:

### System structure
- [ ] `system.yaml` exists with correct `name` and `description`
- [ ] `scripts/` directory exists
- [ ] `templates/` directory exists
- [ ] `documentation-standards/` directory exists
- [ ] `00-domain-relationships.md` exists with domain graph

### Scripts (all required)
- [ ] `scripts/init.py` — returns §8.4 plan JSON
- [ ] `scripts/validate.py` — validates documents/sections
- [ ] `scripts/calculate.py` — computes scores from audit results
- [ ] `scripts/report.py` — generates reports from audit data
- [ ] `scripts/scaffold.py` — creates file/section skeletons
- [ ] `scripts/plan_generation.py` — determines what needs generating

### Script compliance
- [ ] All scripts accept `--repo-root`, `--in`, `--out` args
- [ ] All scripts write valid JSON to `--out`
- [ ] All scripts use the standard envelope (`status`, `message`, `written`)
- [ ] `scaffold.py` includes `skipped` field for idempotency
- [ ] `calculate.py` returns `final_score`, `band`, `breakdown` in output
- [ ] `init.py` returns a valid §8.4 plan with `system`, `use_cases`, `phases`
- [ ] All phases in init plan have `depends_on` properly declared
- [ ] Script phases have `expiry` rules (or `null` for never-expires)

### Templates
- [ ] At least one report template exists
- [ ] Templates use Jinja2 syntax
- [ ] Templates are loadable by your scripts

### Documentation standards
- [ ] One `{domain}.md` file per domain your system supports
- [ ] Each domain file lists required/optional sections
- [ ] Each domain file lists validation rules
- [ ] `00-domain-relationships.md` defines the domain dependency graph

### Inheritance (if applicable)
- [ ] `extends` points to a valid parent system
- [ ] `drops` lists only files that exist in the parent
- [ ] Overridden files are in the correct relative path
- [ ] No circular inheritance chains

### Registration
- [ ] System registered via `register_standard` or loader
- [ ] Init plan stored via `store_system_plan`
- [ ] `cargo check -p mcp` passes (samgraha-side verification)

---

## 11. Worked Example: `rust_dev`

### 11.1 Directory layout

```
samgraha/system/rust_dev/
├── system.yaml
├── scripts/
│   ├── init.py
│   ├── validate.py
│   ├── calculate.py
│   ├── report.py
│   ├── scaffold.py
│   └── plan_generation.py
├── templates/
│   ├── plan/
│   │   └── PLAN.md.j2
│   └── reports/
│       └── domain_report.md.j2
└── documentation-standards/
    ├── 00-domain-relationships.md
    ├── 01-vision-standards.md
    ├── 02-architecture-standards.md
    ├── 03-implementation-standards.md
    ├── 04-readme-standards.md
    └── ...
```

### 11.2 `system.yaml`

```yaml
name: rust_dev
description: "Documentation standards for Rust projects"
extends: base_dev
drops:
  - "06-design-standards.md"
  - "09-feature-design-standards.md"
  - "11-prototype-standards.md"
```

### 11.3 `scripts/calculate.py` (minimal example)

```python
#!/usr/bin/env python3
"""Calculate documentation score for a Rust project."""
import argparse
import json
import sys
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--in", required=True, dest="input_file")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    # Read input (audit results, scoring rules, etc.)
    with open(args.input_file) as f:
        input_data = json.load(f)

    # Your scoring logic here
    score = calculate_score(input_data)

    # Write output envelope
    output = {
        "status": "ok",
        "message": None,
        "written": [],
        "final_score": score["total"],
        "band": score["band"],
        "breakdown": score["breakdown"],
    }
    with open(args.out, "w") as f:
        json.dump(output, f, indent=2)

def calculate_score(input_data):
    # Implement your scoring formula
    # This is YOUR domain logic, not samgraha's
    return {
        "total": 82,
        "band": "Good",
        "breakdown": {"deterministic": 90, "semantic": 74}
    }

if __name__ == "__main__":
    main()
```

---

## 12. FAQ

**Q: Can I use a language not listed in §5.3?**
A: Only if you can make it executable. `.py`/`.sh`/`.ps1`/`.js` are
dispatched automatically. Anything else needs a shebang or must be a
compiled binary.

**Q: Do I need all 6 scripts?**
A: Yes. samgraha expects all capabilities to be available. If a
capability doesn't apply to your system, implement it as a no-op that
returns `{"status": "ok", "message": "not applicable", "written": []}`.

**Q: Can I have multiple scripts per capability?**
A: No. samgraha discovers one script per capability name. If you need
multiple validation passes, combine them into one `validate.py` script.

**Q: How do I handle errors in my scripts?**
A: Return `{"status": "error", "message": "what went wrong", "written": []}`.
samgraha surfaces the message to the caller. Non-zero exit codes also
produce an error, but with a less helpful message.

**Q: Can my scripts access the database?**
A: Yes — open `.samgraha/knowledge.db` directly with `sqlite3`. Your
scripts can read `script_runs`, `system_plans`, `plan_generation_inputs`,
and any other table. Don't modify tables you don't own.

**Q: How do I test my system before registering it?**
A: Run each script manually:
```bash
python scripts/calculate.py \
  --repo-root /path/to/repo \
  --in /tmp/input.json \
  --out /tmp/output.json
cat /tmp/output.json
```

**Q: What if my system needs external dependencies (pip packages, etc.)?**
A: Your scripts manage their own dependencies. Use `requirements.txt`,
`pyproject.toml`, or whatever your language's convention is. samgraha
doesn't manage your dependencies — it just runs your scripts.

---

## Appendix A: Script Template (Python)

Starter template for any capability script:

```python
#!/usr/bin/env python3
"""{Capability} script for {system_name}."""
import argparse
import json
import sys
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--in", required=True, dest="input_file")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    try:
        with open(args.input_file) as f:
            input_data = json.load(f)

        # --- YOUR LOGIC HERE ---

        output = {
            "status": "ok",
            "message": None,
            "written": [],
        }
    except Exception as e:
        output = {
            "status": "error",
            "message": str(e),
            "written": [],
        }

    with open(args.out, "w") as f:
        json.dump(output, f, indent=2)

if __name__ == "__main__":
    main()
```

## Appendix B: Init Plan Template

Starter template for your `init.py` script:

```python
#!/usr/bin/env python3
"""Init script — returns the system's phase-wise plan (§8.4)."""
import argparse
import json

PLAN = {
    "system": "{system_name}",
    "use_cases": [
        {
            "id": "new-repo",
            "label": "New repo, no code, no docs",
            "phases": [
                {
                    "id": "vision-plan-input",
                    "kind": "semantic",
                    "description": "Determine what Vision needs",
                    "depends_on": [],
                    "pre_script": None,
                    "post_script": None,
                },
                {
                    "id": "vision-plan-render",
                    "kind": "script",
                    "script": "scripts/plan_generation.py",
                    "depends_on": ["vision-plan-input"],
                    "pre_script": None,
                    "post_script": None,
                    "expiry": None,
                },
            ],
        },
    ],
}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--in", required=True, dest="input_file")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    output = {
        "status": "ok",
        "message": None,
        "written": [],
        **PLAN,
    }
    with open(args.out, "w") as f:
        json.dump(output, f, indent=2)

if __name__ == "__main__":
    main()
```
