#!/usr/bin/env python3
"""Knowledge Hub Loader — ingests docs/knowledge-hub/ files into schema rows.

The schema (this script's own directory, schema/knowledge-hub/) is a
sibling of docs/knowledge-hub/, not nested inside it — kept separate so
docs/knowledge-hub/ has no samgraha-specific dependency, only document
system content.

Usage:
    python schema/knowledge-hub/knowledge-hub-loader.py [--db PATH] [--system NAME]
        [--knowledge-hub PATH] [--schema PATH] [--layout PATH] [--passes 1,3,5|all]
        [--dry-run] [--reset]

Runs all passes in order inside a single transaction. Idempotent — re-running
updates existing rows via upsert, never duplicates.
"""

import argparse
import json
import re
import shutil
import sqlite3
import sys
from pathlib import Path

import yaml

# Bumped whenever a table is added/removed/changed shape (e.g. the 09/10/13-15
# runtime-table removal). Stored in SQLite's own `PRAGMA user_version` — no
# extra table needed. `register`/`sync` on the Rust side reject a source DB
# whose version doesn't match, instead of a confusing downstream SQL error
# when a query expects a table/column an older or newer DB doesn't have.
SCHEMA_VERSION = 3


# ---------------------------------------------------------------------------
# Schema initialization
# ---------------------------------------------------------------------------

def init_schema(conn: sqlite3.Connection, schema_dir: Path, *, reset: bool = False) -> None:
    """Initialize schema from SQL files in schema/.

    If reset=True, drops all tables first (00-reset.sql) then recreates.
    If reset=False (default), skips entirely when tables already exist.
    """
    has_schema = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='systems'"
    ).fetchone()

    if has_schema and not reset:
        return  # tables exist, upserts in passes handle data

    sql_files = sorted(schema_dir.glob("*.sql"))
    if not sql_files:
        raise FileNotFoundError(f"No .sql files found in {schema_dir}")
    for sql_file in sql_files:
        conn.executescript(sql_file.read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# Pass 0 — systems, standards
# ---------------------------------------------------------------------------

def pass_0(conn: sqlite3.Connection, system_name: str) -> tuple[int, int]:
    """Upsert the systems and standards rows. Returns (system_id, standard_id).

    The first system registered becomes the default (is_default=1). Subsequent
    systems default to is_default=0 — the partial unique index
    ux_systems_one_default enforces exactly one default, so inserting is_default=1
    when another row already holds it crashes with IntegrityError.
    """
    has_systems = conn.execute("SELECT 1 FROM systems LIMIT 1").fetchone()
    default_val = 0 if has_systems else 1
    conn.execute(
        """INSERT INTO systems (name, description, is_default)
           VALUES (?, ?, ?)
           ON CONFLICT (name) DO UPDATE SET description = excluded.description,
                                            is_default = excluded.is_default
           RETURNING id""",
        (system_name, f"Documentation standards for {system_name}", default_val),
    )
    system_id = conn.execute("SELECT id FROM systems WHERE name = ?", (system_name,)).fetchone()[0]

    conn.execute(
        """INSERT INTO standards (system_id, name, version, description)
           VALUES (?, ?, ?, ?)
           ON CONFLICT (system_id, name, version) DO UPDATE
              SET description = excluded.description
           RETURNING id""",
        (system_id, "documentation-standards", "v1",
         "Documentation standards for the samgraha-documentation system"),
    )
    standard_id = conn.execute(
        "SELECT id FROM standards WHERE system_id = ? AND name = ? AND version = ?",
        (system_id, "documentation-standards", "v1"),
    ).fetchone()[0]

    return system_id, standard_id


# ---------------------------------------------------------------------------
# Layout — where each pass looks for its input, relative to the knowledge-hub
# root. Every pass used to hardcode its own subpath (e.g. `kh_dir / "audit"`);
# a system that reorganizes directory names (not file *format* — the
# YAML/Markdown shape each pass parses is still the samgraha-documentation
# convention, see docs/proposal.md) can now override just the paths via
# --layout <json file> or a layout.json sitting in the knowledge-hub dir,
# without touching this script. ponytail: directory names are pluggable,
# parser format is not — a system needing a genuinely different YAML/Markdown
# shape still needs its own loader (see Improvement 1 / Open Question 2 in
# docs/proposal.md), this only removes the "my directories are named
# differently" blocker.
# ---------------------------------------------------------------------------

DEFAULT_LAYOUT: dict[str, str] = {
    "domain_relationships": "00-domain-relationships.md",
    "plan_loop": "plan/core/loop.yaml",
    "templates_generation_document": "templates/generation/document",
    "audit_root": "audit",
    "documentation_standards": "documentation-standards",
    "script_schema": "script/schema",
    "audit_deterministic": "audit/deterministic",
    "audit_semantic": "audit/semantic",
    "templates_root": "templates",
    "calculation_root": "calculation",
    "plan_usecase": "plan/usecase",
}


def resolve_layout(kh_dir: Path, overrides: dict[str, str] | None) -> dict[str, Path]:
    """Merge `overrides` (partial — only the keys a system wants to rename)
    over `DEFAULT_LAYOUT`, then resolve every value to an absolute Path
    under `kh_dir`. Unknown override keys are ignored with a warning rather
    than silently accepted — a typo'd key should not turn into "pass reads
    default and warns file-not-found" without explanation."""
    merged = dict(DEFAULT_LAYOUT)
    for key, value in (overrides or {}).items():
        if key not in DEFAULT_LAYOUT:
            print(f"  Warning: layout override '{key}' is not a known layout key, ignored", file=sys.stderr)
            continue
        merged[key] = value
    return {key: kh_dir / rel for key, rel in merged.items()}


# ---------------------------------------------------------------------------
# Pass registry — populated by later phases
# ---------------------------------------------------------------------------

PASSES: list = []


def register_pass(fn):
    """Decorator to register a pass function. Each receives (conn, standard_id, layout)."""
    PASSES.append(fn)
    return fn


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def extract_yaml_block(md_path: Path) -> dict:
    """Extract the first fenced ```yaml ... ``` block from a Markdown file."""
    text = md_path.read_text(encoding="utf-8")
    match = re.search(r"```yaml\s*\n(.*?)```", text, re.DOTALL)
    if not match:
        raise ValueError(f"No YAML block found in {md_path}")
    return yaml.safe_load(match.group(1))


def extract_relationship_types_with_gating(raw_text: str) -> list[dict]:
    """Parse relationship_types entries, extracting tier-gating from inline
    YAML comments (which the YAML parser strips)."""
    # Find the relationship_types: block
    match = re.search(
        r"relationship_types:.*?\n((?:\s+-\s+.*\n)+)", raw_text, re.DOTALL
    )
    if not match:
        return []
    results = []
    for line in match.group(1).splitlines():
        line = line.strip()
        if not line.startswith("-"):
            continue
        # Extract the type name (everything before #)
        name_part = line[2:].split("#")[0].strip()
        if not name_part:
            continue
        # Extract tier-gating from comment
        gating_match = re.search(r"tier-gating:\s*(\w+)", line)
        tier_gating = gating_match.group(1) if gating_match else "none"
        results.append({"name": name_part, "tier_gating": tier_gating})
    return results


# ---------------------------------------------------------------------------
# Pass 1 — domains, relationship_types, domain_relationships
# ---------------------------------------------------------------------------

@register_pass
def pass_1(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    md_path = layout["domain_relationships"]
    raw_text = md_path.read_text(encoding="utf-8")
    data = extract_yaml_block(md_path)

    # --- domains ---
    domain_key_to_id: dict[str, int] = {}
    sort_counter: dict[int, int] = {}  # tier -> next sort_order
    for tier_entry in data.get("tiers", []):
        tier_num = tier_entry["tier"]
        sort_counter.setdefault(tier_num, 0)
        for domain_key in tier_entry.get("domains", []):
            sort_order = sort_counter[tier_num]
            sort_counter[tier_num] += 1
            name = domain_key.replace("-", " ").title()
            conn.execute(
                """INSERT INTO domains (standard_id, key, name, tier, sort_order, description)
                   VALUES (?, ?, ?, ?, ?, NULL)
                   ON CONFLICT (standard_id, key) DO UPDATE
                      SET name = excluded.name, tier = excluded.tier,
                          sort_order = excluded.sort_order
                   RETURNING id""",
                (standard_id, domain_key, name, tier_num, sort_order),
            )
            row = conn.execute(
                "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
                (standard_id, domain_key),
            ).fetchone()
            domain_key_to_id[domain_key] = row[0]

    print(f"  domains: {len(domain_key_to_id)}")

    # --- relationship_types ---
    rt_data = extract_relationship_types_with_gating(raw_text)
    rt_name_to_id: dict[str, int] = {}
    for rt in rt_data:
        conn.execute(
            """INSERT INTO relationship_types (standard_id, name, tier_gating, description)
               VALUES (?, ?, ?, NULL)
               ON CONFLICT (standard_id, name) DO UPDATE
                  SET tier_gating = excluded.tier_gating
               RETURNING id""",
            (standard_id, rt["name"], rt["tier_gating"]),
        )
        row = conn.execute(
            "SELECT id FROM relationship_types WHERE standard_id = ? AND name = ?",
            (standard_id, rt["name"]),
        ).fetchone()
        rt_name_to_id[rt["name"]] = row[0]

    print(f"  relationship_types: {len(rt_name_to_id)}")

    # --- relationships ---
    rel_count = 0
    for rel in data.get("relationships", []):
        from_key = rel["from"]
        to_key = rel["to"]
        type_name = rel["type"]
        mutual = 1 if rel.get("mutual", False) else 0

        from_id = domain_key_to_id.get(from_key)
        to_id = domain_key_to_id.get(to_key)
        type_id = rt_name_to_id.get(type_name)

        if from_id is None:
            raise ValueError(f"Unknown from_domain: {from_key}")
        if to_id is None:
            raise ValueError(f"Unknown to_domain: {to_key}")
        if type_id is None:
            raise ValueError(f"Unknown relationship_type: {type_name}")

        conn.execute(
            """INSERT INTO domain_relationships
                  (standard_id, from_domain_id, to_domain_id, relationship_type_id,
                   mutual, enforce_order, note)
               VALUES (?, ?, ?, ?, ?, 0, NULL)
               ON CONFLICT (standard_id, from_domain_id, to_domain_id, relationship_type_id)
               DO UPDATE SET mutual = excluded.mutual
               """,
            (standard_id, from_id, to_id, type_id, mutual),
        )
        rel_count += 1

    print(f"  domain_relationships: {rel_count}")

    # --- enforce_order (second pass over loop.yaml) ---
    loop_path = layout["plan_loop"]
    loop_data = yaml.safe_load(loop_path.read_text(encoding="utf-8"))
    enforce_count = 0
    for ordering in loop_data.get("within_tier_ordering", []):
        from_key = ordering.get("from")
        to_key = ordering.get("to")
        if not from_key or not to_key:
            continue
        from_id = domain_key_to_id.get(from_key)
        to_id = domain_key_to_id.get(to_key)
        if from_id is None or to_id is None:
            raise ValueError(
                f"enforce_order references unknown domain: {from_key} -> {to_key}"
            )
        # Find the matching domain_relationships row
        row = conn.execute(
            """SELECT id FROM domain_relationships
               WHERE standard_id = ? AND from_domain_id = ? AND to_domain_id = ?""",
            (standard_id, from_id, to_id),
        ).fetchone()
        if row is None:
            raise ValueError(
                f"enforce_order: no domain_relationships edge for "
                f"{from_key} -> {to_key} (ordering constraint references "
                f"undeclared relationship)"
            )
        conn.execute(
            "UPDATE domain_relationships SET enforce_order = 1 WHERE id = ?",
            (row[0],),
        )
        enforce_count += 1

    print(f"  enforce_order set: {enforce_count}")


# ---------------------------------------------------------------------------
# Pass 2 — section_catalog
# ---------------------------------------------------------------------------

def _parse_markdown_table(text: str, heading: str) -> list[dict]:
    """Find a ## heading and parse the first Markdown table beneath it.
    Returns list of dicts keyed by cleaned column names."""
    # Find the heading
    pattern = rf"##\s+{re.escape(heading)}\s*\n(.*?)(?=\n##\s|\Z)"
    match = re.search(pattern, text, re.DOTALL)
    if not match:
        return []
    block = match.group(1)
    lines = [l.strip() for l in block.splitlines() if l.strip().startswith("|")]
    if len(lines) < 3:  # header + separator + at least one row
        return []
    # Parse header
    headers = [h.strip().strip("`").lower() for h in lines[0].split("|")[1:-1]]
    # Skip separator line (lines[1])
    rows = []
    for line in lines[2:]:
        cells = [c.strip().strip("`") for c in line.split("|")[1:-1]]
        if len(cells) != len(headers):
            continue
        rows.append(dict(zip(headers, cells)))
    return rows


def _resolve_section_type_collisions(section_dir: Path, glob_pat: str) -> dict[tuple[str, str], str]:
    """Scan a domain-keyed section directory tree for filename collisions —
    two or more files in the same domain stripping to the identical
    section_type (e.g. product-guide's 04-product_context.md "early/
    optional" variant vs 07-product_context.md "required", same section
    name, different numeric prefix). The report template for these already
    disambiguates them via distinct Jinja keys (product_context_early vs
    product_context), so: the last (highest-numbered) file keeps the bare
    name, earlier ones get an "_early" suffix (or "_v{n}" if there are more
    than two, though that hasn't occurred in this tree). Returns
    {(domain_key, filename): resolved_section_type} for every file in the
    tree, not just the colliding ones, so callers can look up any file's
    resolved name uniformly."""
    resolved: dict[tuple[str, str], str] = {}
    if not section_dir.is_dir():
        return resolved
    for domain_dir in sorted(section_dir.iterdir()):
        if not domain_dir.is_dir():
            continue
        domain_key = re.sub(r"^\d+-", "", domain_dir.name)
        groups: dict[str, list[Path]] = {}
        for f in sorted(domain_dir.glob(glob_pat)):
            if f.stem == "generic":
                continue
            base_type = re.sub(r"^\d+-", "", f.stem)
            groups.setdefault(base_type, []).append(f)
        for base_type, files in groups.items():
            for i, f in enumerate(files):
                if i == len(files) - 1:
                    resolved[(domain_key, f.name)] = base_type
                else:
                    suffix = "_early" if len(files) == 2 else f"_v{i + 1}"
                    resolved[(domain_key, f.name)] = f"{base_type}{suffix}"
    return resolved


@register_pass
def pass_2(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    gen_dir = layout["templates_generation_document"]
    count = 0
    for md_file in sorted(gen_dir.glob("*.md")):
        # Extract domain key from filename: "01-vision.md" -> "vision"
        name = md_file.stem  # e.g. "01-vision"
        domain_key = name.split("-", 1)[1] if "-" in name else name

        # Resolve domain_id
        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {md_file.name}")
            continue
        domain_id = row[0]

        text = md_file.read_text(encoding="utf-8")
        table_rows = _parse_markdown_table(text, "Required Sections")
        for tr in table_rows:
            sort_order = int(tr.get("#", "0"))
            name = tr.get("section", "")
            semantic_type = tr.get("semantic_type", "")
            required = tr.get("required", "")
            mandatory = 1 if required and ("✓" in required or "check" in required.lower()) else 0

            if not semantic_type:
                raise ValueError(f"Missing semantic_type in {md_file.name} row: {tr}")

            conn.execute(
                """INSERT INTO section_catalog
                      (domain_id, semantic_type, name, sort_order, mandatory)
                   VALUES (?, ?, ?, ?, ?)
                   ON CONFLICT (domain_id, semantic_type) DO UPDATE
                      SET name = excluded.name, sort_order = excluded.sort_order,
                          mandatory = excluded.mandatory""",
                (domain_id, semantic_type, name, sort_order, mandatory),
            )
            count += 1

    # Extend the catalog with sections that exist in the audit trees but
    # were never declared in any generation template's Required Sections
    # table (e.g. observability, stakeholders, success_criteria — genuine
    # audit-only extras, not typos). Registered non-mandatory since they're
    # outside the Required Sections contract. Without this, pass_5/pass_6
    # silently collide instead of erroring when they can't resolve these —
    # found by tracing 55 lost rules + 30 lost templates back to exactly
    # this gap.
    extra_count = 0
    collision_count = 0
    audit_dir = layout["audit_root"]
    for tree, glob_pat in (
        (audit_dir / "semantic" / "section", "*.md"),
        (audit_dir / "deterministic" / "section", "*.yaml"),
    ):
        if not tree.is_dir():
            continue
        collision_map = _resolve_section_type_collisions(tree, glob_pat)
        for (dk, fname), resolved_type in sorted(collision_map.items()):
            if resolved_type != re.sub(r"^\d+-", "", Path(fname).stem):
                print(f"  NOTE: {fname} in domain '{dk}' shares its section "
                      f"name with a later file — registered as '{resolved_type}'")
                collision_count += 1
        for domain_dir in sorted(tree.iterdir()):
            if not domain_dir.is_dir():
                continue
            domain_key = re.sub(r"^\d+-", "", domain_dir.name)
            drow = conn.execute(
                "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
                (standard_id, domain_key),
            ).fetchone()
            if drow is None:
                continue
            domain_id = drow[0]
            for f in sorted(domain_dir.glob(glob_pat)):
                if f.stem == "generic":
                    continue
                section_type = collision_map.get(
                    (domain_key, f.name), re.sub(r"^\d+-", "", f.stem)
                )
                exists = conn.execute(
                    "SELECT 1 FROM section_catalog WHERE domain_id = ? AND semantic_type = ?",
                    (domain_id, section_type),
                ).fetchone()
                if exists:
                    continue
                max_order = conn.execute(
                    "SELECT COALESCE(MAX(sort_order), 0) FROM section_catalog WHERE domain_id = ?",
                    (domain_id,),
                ).fetchone()[0]
                display_name = section_type.replace("_", " ").title()
                conn.execute(
                    """INSERT INTO section_catalog
                          (domain_id, semantic_type, name, sort_order, mandatory)
                       VALUES (?, ?, ?, ?, 0)
                       ON CONFLICT (domain_id, semantic_type) DO NOTHING""",
                    (domain_id, section_type, display_name, max_order + 1),
                )
                extra_count += 1

    print(f"  section_catalog: {count} (+{extra_count} audit-only extras, "
          f"{collision_count} filename collisions resolved)")


# ---------------------------------------------------------------------------
# Pass 3 — standard_docs
# ---------------------------------------------------------------------------

@register_pass
def pass_3(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    std_dir = layout["documentation_standards"]
    count = 0
    for md_file in sorted(std_dir.glob("*.md")):
        name = md_file.stem  # e.g. "01-vision-standards"
        # Extract domain key: "01-vision-standards" -> "vision"
        parts = name.split("-")
        # Remove leading NN prefix and trailing "standards"
        domain_parts = [p for p in parts[1:] if p != "standards"]
        domain_key = "-".join(domain_parts)

        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {md_file.name}")
            continue
        domain_id = row[0]

        text = md_file.read_text(encoding="utf-8")
        # Extract H1 title
        h1_match = re.search(r"^#\s+(.+)$", text, re.MULTILINE)
        title = h1_match.group(1).strip() if h1_match else name

        conn.execute(
            """INSERT INTO standard_docs (domain_id, title, content, source_file)
               VALUES (?, ?, ?, ?)
               ON CONFLICT (domain_id) DO UPDATE
                  SET title = excluded.title, content = excluded.content,
                      source_file = excluded.source_file""",
            (domain_id, title, text, md_file.name),
        )
        count += 1

    print(f"  standard_docs: {count}")


# ---------------------------------------------------------------------------
# Pass 4 — script_checks, script_check_dependencies
# ---------------------------------------------------------------------------

@register_pass
def pass_4(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    schema_dir = layout["script_schema"]
    check_name_to_id: dict[str, int] = {}

    for manifest_file in sorted(schema_dir.rglob("*.manifest.yaml")):
        domain_folder = manifest_file.parent.name
        domain_key = None
        if domain_folder != "_generic":
            # Strip NN- prefix: "03-security" -> "security"
            domain_key = re.sub(r"^\d+-", "", domain_folder)

        # Resolve domain_id
        domain_id = None
        if domain_key:
            row = conn.execute(
                "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
                (standard_id, domain_key),
            ).fetchone()
            if row is None:
                print(f"  WARNING: domain '{domain_key}' not found, skipping {manifest_file.name}")
                continue
            domain_id = row[0]

        # Read manifest
        manifest = yaml.safe_load(manifest_file.read_text(encoding="utf-8"))
        check_name = manifest["check"]

        # Read matching schema.json
        schema_json_path = manifest_file.with_suffix("").with_suffix(".schema.json")
        if not schema_json_path.exists():
            raise FileNotFoundError(f"Missing schema JSON: {schema_json_path}")
        schema_body = schema_json_path.read_text(encoding="utf-8")
        schema_data = json.loads(schema_body)

        # Extract category from properties.category.const
        category = (
            schema_data.get("properties", {})
            .get("category", {})
            .get("const")
        )

        timeout = manifest.get("timeout_seconds", 60)
        requires_network = 1 if manifest.get("requires_network", False) else 0

        conn.execute(
            """INSERT INTO script_checks
                  (standard_id, domain_id, check_name, category,
                   timeout_seconds, requires_network, result_schema, description)
               VALUES (?, ?, ?, ?, ?, ?, ?, NULL)
               ON CONFLICT (standard_id, check_name) DO UPDATE
                  SET domain_id = excluded.domain_id, category = excluded.category,
                      timeout_seconds = excluded.timeout_seconds,
                      requires_network = excluded.requires_network,
                      result_schema = excluded.result_schema""",
            (standard_id, domain_id, check_name, category,
             timeout, requires_network, schema_body),
        )
        row = conn.execute(
            "SELECT id FROM script_checks WHERE standard_id = ? AND check_name = ?",
            (standard_id, check_name),
        ).fetchone()
        check_name_to_id[check_name] = row[0]

    # script_check_dependencies was dropped (schema-redesign-proposal.md §4.2
    # — zero Rust consumers, confirmed by usage audit) — a manifest's own
    # `depends_on` list stays in script/schema/*.manifest.yaml on disk;
    # nothing here needs to duplicate it into rows anymore.
    print(f"  script_checks: {len(check_name_to_id)}")


# ---------------------------------------------------------------------------
# Pass 5 — rules, rule_evidence_params
# ---------------------------------------------------------------------------

def _insert_rule_and_params(
    conn: sqlite3.Connection, standard_id: int, rule_key: str, kind: str,
    scope: str, domain_id: int, section_catalog_id: int | None,
    description: str, condition: str, message: str, severity: str,
    weight: float, mandatory: int, evidence_type: str, is_fallback: int,
    evidence_params: list[dict],
) -> None:
    """Insert a rules row and its rule_evidence_params children."""
    existing = conn.execute(
        """SELECT id, description FROM rules
           WHERE standard_id = ? AND domain_id = ?
             AND COALESCE(section_catalog_id, 0) = COALESCE(?, 0)
             AND scope = ? AND kind = ? AND rule_key = ?""",
        (standard_id, domain_id, section_catalog_id, scope, kind, rule_key),
    ).fetchone()
    if existing is not None and existing[1] != description:
        # Two different source items resolved to the identical natural key
        # and would silently overwrite each other via ON CONFLICT — this is
        # not a section-resolution failure (that's caught above), it's two
        # genuinely different rules colliding. Loud, not silent: surfaces
        # cases like product-guide's "early"/"required" section variants,
        # which the report template already disambiguates via distinct
        # Jinja keys (product_context_early vs product_context) but whose
        # source filenames both strip to the same section_type.
        print(f"  WARNING: rule_key '{rule_key}' in domain_id={domain_id} "
              f"section_catalog_id={section_catalog_id} scope={scope} kind={kind} "
              f"collides with an existing rule of different content — "
              f"second insert will overwrite the first (existing description: "
              f"{existing[1]!r}, new description: {description!r})")
    conn.execute(
        """INSERT INTO rules
              (standard_id, domain_id, section_catalog_id, rule_key, kind,
               scope, description, condition, message, severity, weight,
               mandatory, evidence_type, is_fallback)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT (standard_id, domain_id, section_catalog_key, scope, kind, rule_key)
           DO UPDATE SET description = excluded.description,
                         condition = excluded.condition,
                         message = excluded.message,
                         severity = excluded.severity,
                         weight = excluded.weight,
                         mandatory = excluded.mandatory,
                         evidence_type = excluded.evidence_type,
                         is_fallback = excluded.is_fallback
           RETURNING id""",
        (standard_id, domain_id, section_catalog_id, rule_key, kind,
         scope, description, condition, message, severity, weight,
         mandatory, evidence_type, is_fallback),
    )
    rule_id = conn.execute(
        """SELECT id FROM rules
           WHERE standard_id = ? AND domain_id = ? AND rule_key = ?
           AND scope = ? AND kind = ?""",
        (standard_id, domain_id, rule_key, scope, kind),
    ).fetchone()[0]

    # Delete-then-reinsert evidence params (inside transaction — no gap)
    conn.execute("DELETE FROM rule_evidence_params WHERE rule_id = ?", (rule_id,))
    for idx, param in enumerate(evidence_params):
        conn.execute(
            """INSERT INTO rule_evidence_params (rule_id, param_key, param_value, sort_order)
               VALUES (?, ?, ?, ?)""",
            (rule_id, param["key"], param["value"], idx),
        )


def _parse_semantic_rubric(text: str) -> tuple[list[dict], str | None]:
    """Parse a semantic rubric .md file. Returns (criteria_rows, first_objective_bullet)."""
    # Extract first bullet under "## Audit Objectives"
    obj_match = re.search(
        r"##\s+Audit Objectives\s*\n((?:- .+\n)+)", text, re.DOTALL
    )
    first_objective = None
    if obj_match:
        for line in obj_match.group(1).splitlines():
            line = line.strip()
            if line.startswith("- "):
                first_objective = line[2:].strip()
                break

    # Extract Scoring Criteria table
    rows = _parse_markdown_table(text, "Scoring Criteria")
    return rows, first_objective


@register_pass
def pass_5(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    rule_count = 0
    param_count = 0

    # --- 5a. Deterministic YAML rules ---
    det_dir = layout["audit_deterministic"]
    for yaml_file in sorted(det_dir.rglob("*.yaml")):
        if yaml_file.name.endswith("-relationships.yaml"):
            continue  # handled in 5c

        data = yaml.safe_load(yaml_file.read_text(encoding="utf-8"))
        if not data or "rules" not in data:
            continue

        domain_key = data.get("domain")
        scope = data.get("scope", "document")
        section_type = data.get("section_type")

        if not domain_key:
            raise ValueError(f"Missing domain in {yaml_file}")

        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {yaml_file.name}")
            continue
        domain_id = row[0]

        # Resolve section_catalog_id
        section_catalog_id = None
        if scope == "section" and section_type:
            sc_row = conn.execute(
                """SELECT sc.id FROM section_catalog sc
                   JOIN domains d ON sc.domain_id = d.id
                   WHERE d.standard_id = ? AND d.key = ? AND sc.semantic_type = ?""",
                (standard_id, domain_key, section_type),
            ).fetchone()
            if sc_row:
                section_catalog_id = sc_row[0]
            else:
                print(f"  WARNING: section '{section_type}' not found in domain "
                      f"'{domain_key}' catalog, skipping {yaml_file.name}")
                continue

        for rule in data["rules"]:
            rule_key = rule["id"]
            description = rule.get("description", "")
            condition = rule.get("condition", "")
            message = rule.get("message", "")
            severity = rule.get("severity", "warning")
            weight = float(rule.get("weight", 1.0))
            mandatory = 1 if rule.get("mandatory", False) else 0
            # A rule declares its evidence either as `evidence: {type: ..., ...}`
            # or, for glob-shaped checks, as `check: {file_globs: [...]}` with
            # no `type` key at all (both forms are real — the same standard's
            # own rule files mix them, e.g. inf-001/002 use `evidence:`,
            # inf-003 uses `check:`). Falling back to "unknown" with zero
            # params for the second form would silently make that rule a
            # permanent no-op, so both are normalized into the same
            # (evidence_type, params) shape here rather than assuming every
            # rule uses one form.
            evidence = rule.get("evidence")
            check = rule.get("check")
            if evidence:
                evidence_type = evidence.get("type", "unknown")
                evidence_items = {k: v for k, v in evidence.items() if k != "type"}
            elif check and "file_globs" in check:
                evidence_type = "glob_match"
                evidence_items = {"pattern": check["file_globs"]}
            elif check:
                evidence_type = "unknown"
                evidence_items = check
            else:
                evidence_type = "unknown"
                evidence_items = {}

            evidence_params = []
            for k, v in evidence_items.items():
                if isinstance(v, list):
                    for item in v:
                        evidence_params.append({"key": k, "value": str(item)})
                else:
                    evidence_params.append({"key": k, "value": str(v)})

            _insert_rule_and_params(
                conn, standard_id, rule_key, "deterministic", scope,
                domain_id, section_catalog_id, description, condition,
                message, severity, weight, mandatory, evidence_type, 0,
                evidence_params,
            )
            rule_count += 1
            param_count += len(evidence_params)

    # --- 5b. Semantic Markdown rubrics ---
    sem_dir = layout["audit_semantic"]
    # Same collision resolution as pass_2's audit-only-extras scan (see
    # _resolve_section_type_collisions) — must match exactly, or
    # section_catalog will have the suffixed entry but this pass will
    # still look up the bare name.
    sem_collision_map = _resolve_section_type_collisions(sem_dir / "section", "*.md")
    for md_file in sorted(sem_dir.rglob("*.md")):
        text = md_file.read_text(encoding="utf-8")
        criteria, first_objective = _parse_semantic_rubric(text)
        if not criteria:
            continue

        # Determine domain and section from path
        rel = md_file.relative_to(sem_dir)
        parts = rel.parts  # e.g. ("document", "01-vision.md") or ("section", "01-vision", "01-purpose.md")
        scope = parts[0]  # "document" or "section"

        # Extract domain key from NN-domain pattern
        if scope == "document":
            fname = parts[1]  # "01-vision.md"
        else:
            fname = parts[1]  # "01-vision" (folder)

        domain_key = re.sub(r"^\d+-", "", fname.replace(".md", ""))

        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {md_file.name}")
            continue
        domain_id = row[0]

        # Resolve section_catalog_id for section-scoped
        section_catalog_id = None
        is_generic = md_file.stem == "generic"
        if scope == "section" and not is_generic:
            # Extract section type from filename: "01-purpose.md" -> "purpose"
            # (or the collision-resolved name, e.g. "product_context_early",
            # when another file in this domain shares the bare name)
            section_type = sem_collision_map.get(
                (domain_key, md_file.name), re.sub(r"^\d+-", "", md_file.stem)
            )
            sc_row = conn.execute(
                """SELECT sc.id FROM section_catalog sc
                   JOIN domains d ON sc.domain_id = d.id
                   WHERE d.standard_id = ? AND d.key = ? AND sc.semantic_type = ?""",
                (standard_id, domain_key, section_type),
            ).fetchone()
            if sc_row:
                section_catalog_id = sc_row[0]
            else:
                print(f"  WARNING: section '{section_type}' not found in domain "
                      f"'{domain_key}' catalog, skipping {md_file.name}")
                continue

        for cr in criteria:
            rule_key = cr.get("id", "")
            if not rule_key:
                continue
            description = cr.get("description", "")
            weight_str = cr.get("weight", "recommended")
            score_str = cr.get("score", "")

            # Parse score: "0 or 40" -> extract numeric
            score_match = re.search(r"(\d+(?:\.\d+)?)", score_str)
            weight_val = float(score_match.group(1)) if score_match else 1.0

            is_mandatory = 1 if weight_str.strip().lower() == "mandatory" else 0
            severity = "error" if is_mandatory else "warning"

            condition = first_objective if first_objective else description
            message = description  # message = description (final decision)

            _insert_rule_and_params(
                conn, standard_id, rule_key, "semantic", scope,
                domain_id, section_catalog_id, description, condition,
                message, severity, weight_val, is_mandatory, "llm_judgment",
                1 if is_generic else 0, [],
            )
            rule_count += 1

    # --- 5b2. Semantic YAML (document-scope, one prompt per domain) ---
    # A different real shape from 5b's markdown criteria tables — no
    # per-criterion "Scoring Criteria" table, one prompt_template evaluating
    # the whole domain's documentation at once (python_hackathon's actual
    # audit/semantic/document/*.yaml — 5b's rglob("*.md") never sees these
    # at all, so without this they were silently never ingested). One rule
    # per file, rule_key fixed at "llm-review" (there's exactly one prompt
    # per domain per file, no natural per-criterion id the way the markdown
    # table format has one).
    for yaml_file in sorted(sem_dir.rglob("*.yaml")):
        data = yaml.safe_load(yaml_file.read_text(encoding="utf-8"))
        if not data or "prompt_template" not in data:
            continue  # not this shape — e.g. a file 5a already handled

        domain_key = data.get("domain")
        scope = data.get("scope", "document")
        if not domain_key:
            print(f"  WARNING: missing domain in {yaml_file}, skipping")
            continue

        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {yaml_file.name}")
            continue
        domain_id = row[0]

        description = (data.get("description") or f"Semantic review of {domain_key}").strip()
        prompt = data.get("prompt_template", "").strip()

        params = [{"key": "prompt_template", "value": prompt}]
        for model in (data.get("ensemble") or {}).get("required_models", []):
            params.append({"key": "ensemble_models", "value": model})
        for field in data.get("metadata_fields", []):
            params.append({"key": "metadata_fields", "value": field})

        _insert_rule_and_params(
            conn, standard_id, "llm-review", "semantic", scope,
            domain_id, None, description, description,
            description, "warning", 1.0, 0, "llm_judgment",
            0, params,
        )
        rule_count += 1
        param_count += len(params)

    # --- 5c. Relationship YAML rules ---
    for yaml_file in sorted(det_dir.glob("*-relationships.yaml")):
        data = yaml.safe_load(yaml_file.read_text(encoding="utf-8"))
        if not data or "relationships" not in data:
            continue

        domain_key = data.get("domain")
        if not domain_key:
            raise ValueError(f"Missing domain in {yaml_file}")

        row = conn.execute(
            "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
            (standard_id, domain_key),
        ).fetchone()
        if row is None:
            print(f"  WARNING: domain '{domain_key}' not found, skipping {yaml_file.name}")
            continue
        domain_id = row[0]

        for entry in data["relationships"]:
            owner = entry.get("owner")
            entry_id = entry.get("id", "")

            if owner == "document":
                # Document-owned: kind=semantic, scope=document
                description = entry.get("description", "")
                entry_type = entry.get("type", "unknown")
                _insert_rule_and_params(
                    conn, standard_id, entry_id, "semantic", "document",
                    domain_id, None, description, description, description,
                    "warning", 1.0, 0, entry_type, 0, [],
                )
                rule_count += 1

            elif owner == "section":
                # Section-owned: kind=deterministic, scope=section
                from_section = entry.get("from_section", "")
                target_domain = entry.get("target_domain")
                target_section = entry.get("target_section")
                entry_type = entry.get("type", "")

                # Skip Tier-1 "no upstream" entries
                if target_domain is None or target_section is None:
                    continue

                # Resolve section_catalog_id from from_section
                sc_row = conn.execute(
                    """SELECT sc.id FROM section_catalog sc
                       JOIN domains d ON sc.domain_id = d.id
                       WHERE d.standard_id = ? AND d.key = ? AND sc.semantic_type = ?""",
                    (standard_id, domain_key, from_section),
                ).fetchone()
                if sc_row is None:
                    print(f"  WARNING: from_section '{from_section}' not found in domain "
                          f"'{domain_key}' catalog, skipping relationship '{entry_id}'")
                    continue
                sc_id = sc_row[0]

                desc = f"Traceability: '{from_section}' {entry_type} {target_domain}.{target_section}"
                cond = f"section is cross-referenced to {target_domain}.{target_section}"
                msg = f"Missing expected {entry_type} cross-reference from '{from_section}' to {target_domain}.{target_section}"

                evidence_params = [
                    {"key": "target_domain", "value": target_domain},
                    {"key": "target_section", "value": target_section},
                    {"key": "type", "value": entry_type},
                ]

                _insert_rule_and_params(
                    conn, standard_id, entry_id, "deterministic", "section",
                    domain_id, sc_id, desc, cond, msg, "warning", 0.5, 0,
                    "cross_reference", 0, evidence_params,
                )
                rule_count += 1
                param_count += len(evidence_params)

    print(f"  rules: {rule_count}")
    print(f"  rule_evidence_params: {param_count}")


# ---------------------------------------------------------------------------
# Pass 6 — templates
# ---------------------------------------------------------------------------

@register_pass
def pass_6(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    templates_dir = layout["templates_root"]
    count = 0

    def _strip_prefix(name: str) -> str:
        """Strip NN- numeric prefix from folder/file names."""
        return re.sub(r"^\d+-", "", name)

    def _process_template(md_file: Path, kind: str, audit_bucket: str | None,
                          scope: str, domain_key: str | None,
                          section_type: str | None) -> None:
        nonlocal count

        # Resolve domain_id
        domain_id = None
        if domain_key:
            row = conn.execute(
                "SELECT id FROM domains WHERE standard_id = ? AND key = ?",
                (standard_id, domain_key),
            ).fetchone()
            if row is None:
                print(f"  WARNING: domain '{domain_key}' not found, skipping {md_file.name}")
                return
            domain_id = row[0]

        # Resolve section_catalog_id
        section_catalog_id = None
        if scope == "section" and section_type and domain_id:
            sc_row = conn.execute(
                """SELECT sc.id FROM section_catalog sc
                   WHERE sc.domain_id = ? AND sc.semantic_type = ?""",
                (domain_id, section_type),
            ).fetchone()
            if sc_row:
                section_catalog_id = sc_row[0]
            else:
                print(f"  WARNING: section '{section_type}' not found for domain "
                      f"'{domain_key}', skipping {md_file.name}")
                return

        text = md_file.read_text(encoding="utf-8")
        h1_match = re.search(r"^#\s+(.+)$", text, re.MULTILINE)
        name = h1_match.group(1).strip() if h1_match else md_file.stem

        # sort_order: section-scoped generation templates use section_catalog order
        sort_order = 0
        if kind == "generation" and scope == "section" and section_catalog_id:
            so_row = conn.execute(
                "SELECT sort_order FROM section_catalog WHERE id = ?",
                (section_catalog_id,),
            ).fetchone()
            if so_row:
                sort_order = so_row[0]

        source_file = str(md_file.relative_to(templates_dir))

        conn.execute(
            """INSERT INTO templates
                  (standard_id, domain_id, section_catalog_id, kind,
                   audit_bucket, scope, name, content, sort_order, source_file)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT (standard_id, domain_id, section_catalog_key, kind,
                            audit_bucket_key, scope)
               DO UPDATE SET name = excluded.name, content = excluded.content,
                             sort_order = excluded.sort_order,
                             source_file = excluded.source_file""",
            (standard_id, domain_id, section_catalog_id, kind,
             audit_bucket, scope, name, text, sort_order, source_file),
        )
        count += 1

    # --- generation/document/{NN-domain}.md ---
    gen_doc_dir = templates_dir / "generation" / "document"
    for md_file in sorted(gen_doc_dir.glob("*.md")):
        domain_key = _strip_prefix(md_file.stem)
        _process_template(md_file, "generation", None, "document", domain_key, None)

    # --- generation/section/{NN-domain}/{NN-section-type}.md ---
    gen_sec_dir = templates_dir / "generation" / "section"
    if gen_sec_dir.is_dir():
        for domain_dir in sorted(gen_sec_dir.iterdir()):
            if not domain_dir.is_dir():
                continue
            domain_key = _strip_prefix(domain_dir.name)
            for md_file in sorted(domain_dir.glob("*.md")):
                # generation/section filenames use hyphens for multi-word
                # names ("participating-components.md") but section_catalog
                # (sourced from Required Sections tables, matching the
                # audit tree's convention) uses underscores — normalize so
                # the lookup in _process_template actually matches instead
                # of silently colliding on NULL.
                section_type = _strip_prefix(md_file.stem).replace("-", "_")
                _process_template(md_file, "generation", None, "section",
                                  domain_key, section_type)

    # --- audit/{deterministic,semantic}/{document,section}/{NN}-*-report.md ---
    # Flat structure: files sit directly in document/ and section/, named per domain.
    # Each file is a report template for a domain.
    audit_dir = templates_dir / "audit"
    for bucket_dir in sorted(audit_dir.iterdir()):
        if not bucket_dir.is_dir() or bucket_dir.name in ("summary", "README.md"):
            continue
        audit_bucket = bucket_dir.name

        for scope in ("document", "section"):
            scope_dir = bucket_dir / scope
            if not scope_dir.is_dir():
                continue
            for md_file in sorted(scope_dir.glob("*.md")):
                if md_file.name == "README.md":
                    continue
                # 01-vision-report.md → strip NN- prefix, then strip -report
                domain_key = _strip_prefix(md_file.stem).removesuffix("-report")
                _process_template(md_file, "audit_report", audit_bucket,
                                  scope, domain_key, None)

    # --- audit/summary/{NN}-*-report.md (per-domain summary reports) ---
    summary_dir = audit_dir / "summary"
    if summary_dir.is_dir():
        for md_file in sorted(summary_dir.glob("*.md")):
            if md_file.name == "README.md":
                continue
            domain_key = _strip_prefix(md_file.stem).removesuffix("-report")
            _process_template(md_file, "audit_report", "summary",
                              "document", domain_key, None)

    print(f"  templates: {count}")


# ---------------------------------------------------------------------------
# Pass 7 — calculation_rules, calculation_inputs, score_bands
# ---------------------------------------------------------------------------

def _bucket_name(yaml_file: Path, calc_dir: Path) -> str:
    """Derive a bucket key from a calculation file's own location — e.g.
    deterministic/document.yaml -> deterministic_document,
    aggregation/domain/01-infrastructure.yaml -> aggregation_domain_01_infrastructure.
    No standard's specific bucket names (deterministic_document, final_score, ...)
    are hardcoded anywhere — whatever files a standard's calculation/ directory
    contains is what becomes rows here, named after their own path."""
    rel = yaml_file.relative_to(calc_dir).with_suffix("")
    return re.sub(r"[^a-zA-Z0-9]+", "_", str(rel)).strip("_")


@register_pass
def pass_7(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    # No fixed list of expected files/buckets (no "every standard has exactly
    # a deterministic/document.yaml, a summary/final_score.yaml, ..."
    # assumption) — every *.yaml under calculation/ is inspected by its own
    # content and classified generically:
    #   "bands" present            -> a rating-band file -> score_bands
    #   "calculation" + "formula"  -> a scoring-bucket file -> calculation_rules
    #   neither                    -> not a calculation-rule file (e.g. a
    #                                 standard's weights.yaml or
    #                                 validation/scoring_validation.yaml,
    #                                 which has its own "checks" shape) -> skipped
    # This means a standard that defines its own bucket set — 3 buckets, 12
    # buckets, no "final_score"/"trend" at all — is fully supported without
    # touching this loader.
    calc_dir = layout["calculation_root"]
    rule_count = 0
    input_count = 0
    band_count = 0

    def _upsert_rule(bucket, method, scope, formula, rollup=None,
                      tol_method=None, tol_k=None, tol_floor=None,
                      tol_scope=None, min_samples=None,
                      fallback_scope=None, fallback_min=None, note=None):
        nonlocal rule_count
        conn.execute(
            """INSERT INTO calculation_rules
                  (standard_id, bucket, calculation_method, scope, formula, rollup,
                   tolerance_method, tolerance_k, tolerance_floor, tolerance_scope,
                   min_samples, fallback_scope, fallback_min_samples, note)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT (standard_id, bucket) DO UPDATE SET
                   calculation_method = excluded.calculation_method,
                   scope = excluded.scope, formula = excluded.formula,
                   rollup = excluded.rollup,
                   tolerance_method = excluded.tolerance_method,
                   tolerance_k = excluded.tolerance_k,
                   tolerance_floor = excluded.tolerance_floor,
                   tolerance_scope = excluded.tolerance_scope,
                   min_samples = excluded.min_samples,
                   fallback_scope = excluded.fallback_scope,
                   fallback_min_samples = excluded.fallback_min_samples,
                   note = excluded.note
               RETURNING id""",
            (standard_id, bucket, method, scope, formula, rollup,
             tol_method, tol_k, tol_floor, tol_scope,
             min_samples, fallback_scope, fallback_min, note),
        )
        rule_count += 1
        return conn.execute(
            "SELECT id FROM calculation_rules WHERE standard_id = ? AND bucket = ?",
            (standard_id, bucket),
        ).fetchone()[0]

    if not calc_dir.is_dir():
        print("  calculation_rules: 0 (no calculation/ directory)")
        return

    for yaml_file in sorted(calc_dir.rglob("*.yaml")):
        data = yaml.safe_load(yaml_file.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            continue

        if "bands" in data:
            for i, band in enumerate(data["bands"]):
                conn.execute(
                    """INSERT INTO score_bands
                          (standard_id, rating, min_score, max_score, sort_order)
                       VALUES (?, ?, ?, ?, ?)
                       ON CONFLICT (standard_id, rating) DO UPDATE SET
                           min_score = excluded.min_score,
                           max_score = excluded.max_score,
                           sort_order = excluded.sort_order""",
                    (standard_id, band["rating"], band["min"], band["max"], i),
                )
                band_count += 1
            continue

        if "calculation" not in data or "formula" not in data:
            continue  # not a calculation-rule file — e.g. weights.yaml, validation/*.yaml

        bucket = _bucket_name(yaml_file, calc_dir)
        rollup = data.get("rollup")
        rollup_json = json.dumps(rollup) if rollup else None
        tol = data.get("tolerance") or {}
        note = (data.get("note") or "").strip() or None

        rule_id = _upsert_rule(
            bucket, data["calculation"], data.get("scope"), data["formula"].strip(),
            rollup=rollup_json,
            tol_method=tol.get("method"), tol_k=tol.get("k"), tol_floor=tol.get("floor"),
            tol_scope=tol.get("scope"), min_samples=tol.get("min_samples"),
            fallback_scope=tol.get("fallback_scope"), fallback_min=tol.get("fallback_min_samples"),
            note=note,
        )

        # A file's "inputs" only becomes calculation_inputs rows when it's a
        # weighted list of named buckets (final_score's shape) — deterministic/
        # semantic bucket files also have an "inputs" key, but it's a
        # {from, fields} source descriptor, not a weight list; trend's
        # "inputs" is a plain [current_score, previous_score] name list with
        # no weights. Distinguished by shape, not by which file this is.
        inputs = data.get("inputs")
        if isinstance(inputs, list) and inputs and all(
            isinstance(i, dict) and "name" in i and "weight" in i for i in inputs
        ):
            conn.execute("DELETE FROM calculation_inputs WHERE calculation_rule_id = ?", (rule_id,))
            for i, inp in enumerate(inputs):
                conn.execute(
                    """INSERT INTO calculation_inputs
                          (calculation_rule_id, name, weight, sort_order)
                       VALUES (?, ?, ?, ?)""",
                    (rule_id, inp["name"], inp["weight"], i),
                )
                input_count += 1

    print(f"  calculation_rules: {rule_count}")
    print(f"  calculation_inputs: {input_count}")
    print(f"  score_bands: {band_count}")


# ---------------------------------------------------------------------------
# Pass 8 — plan_settings, plan_scenarios
# ---------------------------------------------------------------------------

@register_pass
def pass_8(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    loop_path = layout["plan_loop"]
    loop_data = yaml.safe_load(loop_path.read_text(encoding="utf-8"))

    # plan_settings (one row)
    threshold = loop_data.get("threshold", {})
    conn.execute(
        """INSERT INTO plan_settings
              (standard_id, threshold_rating, max_iterations, fallback, note)
           VALUES (?, ?, ?, ?, ?)
           ON CONFLICT (standard_id) DO UPDATE SET
               threshold_rating = excluded.threshold_rating,
               max_iterations = excluded.max_iterations,
               fallback = excluded.fallback,
               note = excluded.note""",
        (standard_id, threshold.get("rating", "Acceptable"),
         loop_data.get("max_iterations", 5),
         loop_data.get("fallback", "human_review"),
         loop_data.get("note", "").strip() or None),
    )

    # plan_scenarios (walk the usecase tree)
    usecase_dir = layout["plan_usecase"]
    scenario_count = 0

    # repo_state dirs: repo_existing, repo_new
    for repo_dir in sorted(usecase_dir.iterdir()):
        if not repo_dir.is_dir():
            continue
        repo_state = re.sub(r"^repo_", "", repo_dir.name)  # existing / new

        # doc_state dirs: case_1_no_documentation, case_2_has_documention
        for case_dir in sorted(repo_dir.iterdir()):
            if not case_dir.is_dir():
                continue
            # Normalize typo: "documention" → "documentation"
            doc_state = re.sub(r"^case_\d+_", "", case_dir.name)
            doc_state = doc_state.replace("documention", "documentation")

            # tier dirs: tier_1 .. tier_8
            for tier_dir in sorted(case_dir.iterdir()):
                if not tier_dir.is_dir():
                    continue
                tier = int(re.sub(r"^tier_", "", tier_dir.name))

                # step files: 01-generation.md, 02-audit.md, 03-fix.md
                for step_file in sorted(tier_dir.glob("*.md")):
                    step_name = re.sub(r"^\d+-", "", step_file.stem)
                    content = step_file.read_text(encoding="utf-8")

                    conn.execute(
                        """INSERT INTO plan_scenarios
                              (standard_id, repo_state, doc_state, tier, step, content)
                           VALUES (?, ?, ?, ?, ?, ?)
                           ON CONFLICT (standard_id, repo_state, doc_state, tier, step)
                           DO UPDATE SET content = excluded.content""",
                        (standard_id, repo_state, doc_state, tier, step_name, content),
                    )
                    scenario_count += 1

    print(f"  plan_settings: 1")
    print(f"  plan_scenarios: {scenario_count}")


# ---------------------------------------------------------------------------
# Pass 9 — validation_rules
# ---------------------------------------------------------------------------

@register_pass
def pass_9(conn: sqlite3.Connection, standard_id: int, layout: dict[str, Path]) -> None:
    """calculation/validation/scoring_validation.yaml -> validation_rules.
    Optional — a standard with no validation/ subdirectory under its
    calculation/ root simply gets zero rows, not an error (same convention
    Pass 7 already applies to a missing calculation/ directory entirely)."""
    validation_path = layout["calculation_root"] / "validation" / "scoring_validation.yaml"
    if not validation_path.is_file():
        print("  validation_rules: 0 (no calculation/validation/scoring_validation.yaml)")
        return

    data = yaml.safe_load(validation_path.read_text(encoding="utf-8"))
    checks = data.get("checks", [])
    rule_count = 0
    for i, check in enumerate(checks):
        conn.execute(
            """INSERT INTO validation_rules
                  (standard_id, check_key, name, description, rule, severity, invalidate_audit, sort_order)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT (standard_id, check_key) DO UPDATE SET
                   name = excluded.name,
                   description = excluded.description,
                   rule = excluded.rule,
                   severity = excluded.severity,
                   invalidate_audit = excluded.invalidate_audit,
                   sort_order = excluded.sort_order""",
            (standard_id, check["id"], check.get("name", check["id"]),
             check.get("description"), check.get("rule", ""),
             check.get("severity"), 1 if check.get("invalidate_audit", False) else 0, i),
        )
        rule_count += 1

    print(f"  validation_rules: {rule_count}")


# ---------------------------------------------------------------------------
# Pass 10 — removed. workflow_stages was dropped (schema-redesign-proposal.md
# §4.1 — query ran every startup, result never consumed by any Rust caller;
# superseded by plan_scenarios and, going forward, workflow_phases). No
# replacement pass needed here — nothing reads a stage list from the DB.
# ---------------------------------------------------------------------------


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Ingest knowledge-hub files into schema rows."
    )
    parser.add_argument(
        "--db", default="knowledge-hub.db",
        help="SQLite database path (default: knowledge-hub.db)",
    )
    parser.add_argument(
        "--system", default="samgraha-documentation",
        help="System name to load (default: samgraha-documentation)",
    )
    parser.add_argument(
        "--knowledge-hub", default=None,
        help="Path to docs/knowledge-hub/ (default: auto-detect relative to repo root)",
    )
    parser.add_argument(
        "--schema", default=None,
        help="Path to the schema/*.sql directory (default: this script's own directory)",
    )
    parser.add_argument(
        "--layout", default=None,
        help="Path to a JSON file overriding directory-name keys in DEFAULT_LAYOUT "
             "(partial — only include the keys you want to rename). Lets a system "
             "with renamed directories register without editing this script; the "
             "file/YAML *format* each pass parses is unchanged. Default: none "
             "(uses samgraha-documentation's own layout), or auto-detects "
             "<knowledge-hub-dir>/layout.json if present.",
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Parse and validate without writing to DB (rolls back instead of committing)",
    )
    parser.add_argument(
        "--reset", action="store_true",
        help="Drop and recreate all tables before loading (destroys runtime data!)",
    )
    parser.add_argument(
        "--passes", default="all",
        help="Comma-separated pass numbers to run (e.g. '1,3,5'), or 'all' (default). "
             "Pass 0 (systems+standards) always runs regardless — every other pass "
             "depends on standard_id existing. A system that only defines domains and "
             "rules doesn't need to scan for templates/scripts/scoring/plan files it "
             "doesn't have.",
    )
    return parser.parse_args()


def find_knowledge_hub(script_path: Path) -> Path:
    """Walk up from the script location to find docs/knowledge-hub/."""
    current = script_path.resolve().parent
    for _ in range(10):  # safety limit
        candidate = current / "docs" / "knowledge-hub"
        if candidate.is_dir():
            return candidate
        current = current.parent
        if current == current.parent:
            break
    raise FileNotFoundError(
        "Could not find docs/knowledge-hub/ relative to script location. "
        "Use --knowledge-hub to specify the path."
    )


def main() -> int:
    args = parse_args()
    script_path = Path(__file__).resolve()
    kh_dir = Path(args.knowledge_hub) if args.knowledge_hub else find_knowledge_hub(script_path)
    # schema/ is a sibling of docs/knowledge-hub/, not nested inside it —
    # this script lives inside the schema directory itself, so that's the
    # default; --schema overrides for a different layout.
    schema_dir = Path(args.schema) if args.schema else script_path.parent

    if not schema_dir.is_dir():
        print(f"Error: schema directory not found at {schema_dir}", file=sys.stderr)
        return 1

    # --- Inheritance resolution (Phase 1) ---
    # If the target system has a system.yaml with an extends field, resolve
    # the inheritance chain (base + overlay) into a merged temp directory.
    # The loader then processes the merged tree exactly as it would a
    # standalone system — no pass logic changes needed.
    merged_dir = None
    from system_merger import (
        load_system_metadata, resolve_system, validate_merged_system,
        CircularInheritanceError, DanglingEdgeError,
    )
    meta = load_system_metadata(kh_dir)
    if meta and meta.extends:
        try:
            merged_dir = resolve_system(kh_dir)
            validate_merged_system(merged_dir, meta)
            kh_dir = merged_dir
            print(f"Inheritance resolved: {args.system or kh_dir.name} extends "
                  f"'{meta.extends}' -> merged tree at {merged_dir}")
        except CircularInheritanceError as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1
        except DanglingEdgeError as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1
    elif meta and meta.abstract:
        print(f"Warning: system '{kh_dir.name}' is marked abstract — "
              f"it should only be used as a base for inheritance, not "
              f"registered standalone.", file=sys.stderr)

    layout_overrides: dict[str, str] = {}
    layout_source = Path(args.layout) if args.layout else (kh_dir / "layout.json")
    if layout_source.is_file():
        layout_overrides = json.loads(layout_source.read_text(encoding="utf-8"))
        print(f"Layout overrides loaded from {layout_source} ({len(layout_overrides)} key(s))")
    layout = resolve_layout(kh_dir, layout_overrides)

    if args.passes == "all":
        selected_passes = PASSES
    else:
        wanted = {int(n) for n in args.passes.split(",") if n.strip()}
        selected_passes = [p for p in PASSES if int(p.__name__.rsplit("_", 1)[1]) in wanted]
        skipped = {int(p.__name__.rsplit("_", 1)[1]) for p in PASSES} - wanted
        if skipped:
            print(f"Skipping passes: {sorted(skipped)}")

    db_path = Path(args.db)
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA journal_mode = WAL")

    try:
        print(f"Initializing schema in {db_path} ...")
        init_schema(conn, schema_dir, reset=args.reset)
        conn.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")

        print(f"Pass 0: systems + standards ...")
        system_id, standard_id = pass_0(conn, args.system)
        print(f"  system_id={system_id}, standard_id={standard_id}")

        for pass_fn in selected_passes:
            name = pass_fn.__name__
            print(f"{name} ...")
            pass_fn(conn, standard_id, layout)

        if args.dry_run:
            conn.rollback()
            print("Dry run — nothing written, rolled back.")
        else:
            conn.commit()
            print("Done — all passes committed.")
    except Exception as e:
        conn.rollback()
        print(f"Error — rolled back: {e}", file=sys.stderr)
        raise
    finally:
        conn.close()
        # Clean up merged temp directory if one was created
        if merged_dir is not None:
            shutil.rmtree(merged_dir, ignore_errors=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
