"""System inheritance merger — resolves base+overlay trees for knowledge-hub systems.

Runs as a preprocessing step before knowledge-hub-loader.py. Reads system.yaml
from the target system directory, resolves the inheritance chain (base → concrete,
possibly multi-level), and assembles a merged directory tree that the loader sees
as a standalone system.

The merge is generic and class-agnostic — code reads `extends:` and layers
directories. Adding a new class means a new base directory and systems pointing
`extends:` at it; nothing in this module changes.

Usage:
    from system_merger import resolve_system, load_system_metadata

    merged_dir = resolve_system(Path("samgraha/system/rust_dev"))
    # merged_dir is a temp directory with base+overlay, ready for the loader
"""

import re
import shutil
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

import yaml


# ---------------------------------------------------------------------------
# Metadata
# ---------------------------------------------------------------------------

@dataclass
class SystemMetadata:
    """Parsed contents of a system.yaml file."""
    class_name: str = ""
    subclass: str = ""
    extends: str = ""
    overrides: list[str] = field(default_factory=list)
    drops: list[str] = field(default_factory=list)
    abstract: bool = False
    note: str = ""


class CircularInheritanceError(Exception):
    """Raised when an inheritance chain contains a cycle."""


class DanglingEdgeError(Exception):
    """Raised when drops leave a dangling domain_relationships edge."""


def load_system_metadata(system_dir: Path) -> SystemMetadata | None:
    """Load system.yaml from a system directory. Returns None if absent."""
    meta_path = system_dir / "system.yaml"
    if not meta_path.is_file():
        return None
    data = yaml.safe_load(meta_path.read_text(encoding="utf-8"))
    if not data:
        return None
    return SystemMetadata(
        class_name=data.get("class", ""),
        subclass=data.get("subclass", ""),
        extends=data.get("extends", ""),
        overrides=data.get("overrides", []),
        drops=data.get("drops", []),
        abstract=data.get("abstract", False),
        note=data.get("note", ""),
    )


def find_system_dir(extends_name: str, current_system_dir: Path) -> Path:
    """Resolve a base system name to its directory.

    Looks for the base as a sibling of current_system_dir's parent.
    E.g. if current is samgraha/system/rust_dev and extends is base_dev,
    looks for samgraha/system/base_dev.
    """
    parent = current_system_dir.parent
    candidate = parent / extends_name
    if candidate.is_dir():
        return candidate
    raise FileNotFoundError(
        f"Base system '{extends_name}' not found at {candidate} "
        f"(sibling of {current_system_dir.name})"
    )


# ---------------------------------------------------------------------------
# Merge
# ---------------------------------------------------------------------------

def resolve_system(
    system_dir: Path,
    _chain: set[str] | None = None,
    _tempdirs: list[Path] | None = None,
) -> Path:
    """Resolve inheritance for a system, returning a merged directory tree.

    If the system has no system.yaml or no extends field, returns the
    original system_dir unchanged (backward compatible).

    The merged tree is materialized in a temp directory. Intermediate
    temp dirs from recursive (multi-level) calls are tracked in
    _tempdirs and cleaned up on error; the caller cleans the top-level
    returned path via shutil.rmtree in main()'s finally block.
    """
    meta = load_system_metadata(system_dir)
    if not meta or not meta.extends:
        return system_dir  # no inheritance, pass through

    # Circular dependency detection
    if _chain and meta.extends in _chain:
        raise CircularInheritanceError(
            f"Circular inheritance detected: {system_dir.name} extends "
            f"'{meta.extends}', but '{meta.extends}' is already in the "
            f"chain: {sorted(_chain)}"
        )
    chain = (_chain or set()) | {system_dir.name}

    # Mutable list tracks temp dirs across recursive calls.
    # Top-level caller passes None; each level initializes once.
    if _tempdirs is None:
        _tempdirs = []

    # Resolve base recursively (supports multi-level inheritance)
    base_dir = find_system_dir(meta.extends, system_dir)
    base_merged = resolve_system(base_dir, chain, _tempdirs)

    # Build merged tree: base first, then overlay concrete
    merged = Path(tempfile.mkdtemp(prefix=f"samgraha-merge-{system_dir.name}-"))
    _tempdirs.append(merged)

    try:
        # 1. Copy base tree into merged dir
        shutil.copytree(base_merged, merged, dirs_exist_ok=True)

        # 2. Overlay concrete system's files (override-wins)
        for item in sorted(system_dir.iterdir()):
            if item.name == "system.yaml":
                continue  # metadata not part of the content tree
            dest = merged / item.name
            if item.is_dir():
                shutil.copytree(item, dest, dirs_exist_ok=True)
            else:
                shutil.copy2(item, dest)

        # 3. Apply drops (domain-level removal)
        #
        # Each drop entry is a domain name like "06-design". The merger
        # removes all files related to that domain across the content tree:
        #   - documentation-standards/{NN}-domain-standards.md
        #   - audit/deterministic/document/{NN}-domain.yaml and relationships
        #   - audit/deterministic/section/{NN}-domain/ (entire directory)
        #   - audit/semantic/document/{NN}-domain.md (if present)
        #   - plan/{NN}-domain/ (entire directory, if present)
        #   - calculation/{NN}-domain/ (entire directory, if present)
        #   - templates/generation/section/{NN}-domain/ (if present)
        _apply_drops(merged, meta.drops)

    except Exception:
        # Clean up ALL temp dirs on error (current + all intermediate from recursion).
        # base_merged is either original (skip) or already in _tempdirs (clean it).
        for td in _tempdirs:
            shutil.rmtree(td, ignore_errors=True)
        raise

    # Success: clean intermediate temp dirs (deeper levels in the chain).
    # Our own `merged` dir is returned to the caller, who cleans it.
    # base_merged is either the original dir (skip) or in _tempdirs (already cleaned).
    for td in _tempdirs:
        if td != merged:
            shutil.rmtree(td, ignore_errors=True)

    return merged


# Domain drop patterns — maps a domain name to glob patterns relative
# to the merged root.  Used by _apply_drops to remove all related files.
_DOMAIN_DROP_PATTERNS = [
    # Documentation standards
    lambda d: [f"documentation-standards/{d}-standards.md"],
    # Deterministic document audit
    lambda d: [
        f"audit/deterministic/document/{d}.yaml",
        f"audit/deterministic/document/{d}-relationships.yaml",
    ],
    # Deterministic section audit (directory)
    lambda d: [f"audit/deterministic/section/{d}"],
    # Semantic document audit
    lambda d: [
        f"audit/semantic/document/{d}.md",
    ],
    # Semantic section audit (directory)
    lambda d: [f"audit/semantic/section/{d}"],
    # Plan domain directory
    lambda d: [f"plan/{d}"],
    # Calculation domain directory
    lambda d: [f"calculation/{d}"],
    # Templates generation section
    lambda d: [f"templates/generation/section/{d}"],
]


def _apply_drops(merged: Path, drops: list[str]) -> None:
    """Remove all files related to dropped domains from the merged tree."""
    for domain in drops:
        for pattern_fn in _DOMAIN_DROP_PATTERNS:
            for rel_path_str in pattern_fn(domain):
                target = merged / rel_path_str
                if target.exists():
                    if target.is_dir():
                        shutil.rmtree(target)
                    else:
                        target.unlink()


# ---------------------------------------------------------------------------
# Post-merge validation
# ---------------------------------------------------------------------------

def _extract_yaml_block(md_path: Path) -> dict:
    """Extract the first fenced ```yaml ... ``` block from a Markdown file."""
    text = md_path.read_text(encoding="utf-8")
    match = re.search(r"```yaml\s*\n(.*?)```", text, re.DOTALL)
    if not match:
        return {}
    return yaml.safe_load(match.group(1)) or {}


def _extract_relationships_from_table(md_path: Path) -> list[dict]:
    """Extract relationships from a markdown table in 00-domain-relationships.md.

    Parses the markdown table format:
        | From | Relationship | To |
        |------|--------------|-----|
        | vision | inspires | philosophy |

    Returns a list of {"from": ..., "to": ...} dicts.
    """
    text = md_path.read_text(encoding="utf-8")
    relationships = []
    in_table = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("| From") or stripped.startswith("|---"):
            in_table = True
            continue
        if in_table and stripped.startswith("|"):
            parts = [p.strip() for p in stripped.split("|")]
            # parts[0] and parts[-1] are empty (before/after pipes)
            if len(parts) >= 4:
                from_key = parts[1].strip()
                to_key = parts[3].strip()
                if from_key and to_key:
                    relationships.append({"from": from_key, "to": to_key})
        elif in_table and not stripped.startswith("|"):
            in_table = False  # table ended
    return relationships


def _domain_key_from_drop(drop: str) -> str:
    """Convert a drop entry like '06-design' to the domain key 'design'."""
    # Strip leading number prefix: "06-design" -> "design"
    return re.sub(r"^\d+-", "", drop)


def validate_drops(merged_dir: Path, meta: SystemMetadata) -> None:
    """Cross-check drops against domain_relationships edges.

    After drops are applied, the inherited 00-domain-relationships.md may
    reference domain keys whose files were removed. This would cause Pass 1
    to fail with an opaque error. Instead, detect and report the mismatch
    here with a clear message.

    Raises DanglingEdgeError if dangling edges are found.
    """
    if not meta.drops or not meta.extends:
        return  # nothing to check

    # Convert drops to domain keys for comparison
    dropped_keys = {_domain_key_from_drop(d) for d in meta.drops}

    rel_path = merged_dir / "00-domain-relationships.md"
    if not rel_path.is_file():
        return  # no relationships file, nothing to check

    # Parse relationships from markdown table
    rels = _extract_relationships_from_table(rel_path)
    if not rels:
        return

    # Check edges for dangling references
    dangling: list[str] = []
    for rel in rels:
        from_key = rel.get("from", "")
        to_key = rel.get("to", "")
        if from_key and from_key in dropped_keys:
            dangling.append(f"'{from_key}' (from edge {from_key} -> {to_key})")
        if to_key and to_key in dropped_keys:
            dangling.append(f"'{to_key}' (from edge {from_key} -> {to_key})")

    # Also check enforce_order references
    loop_path = merged_dir / "plan" / "core" / "loop.yaml"
    if loop_path.is_file():
        loop_data = yaml.safe_load(loop_path.read_text(encoding="utf-8")) or {}
        for ordering in loop_data.get("within_tier_ordering", []):
            from_key = ordering.get("from", "")
            to_key = ordering.get("to", "")
            if from_key and from_key in dropped_keys:
                dangling.append(f"'{from_key}' (from enforce_order {from_key} -> {to_key})")
            if to_key and to_key in dropped_keys:
                dangling.append(f"'{to_key}' (from enforce_order {from_key} -> {to_key})")

    if dangling:
        unique = sorted(set(dangling))
        raise DanglingEdgeError(
            f"Drops removed domain(s) still referenced by domain_relationships "
            f"or enforce_order: {', '.join(unique)}. "
            f"Either remove the corresponding edges from 00-domain-relationships.md "
            f"in the concrete system's override, or remove the drops."
        )


def validate_merged_system(merged_dir: Path, meta: SystemMetadata) -> None:
    """Run all post-merge validations."""
    validate_drops(merged_dir, meta)
