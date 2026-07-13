#!/usr/bin/env bash
# Thin wrapper — delegates to the samgraha-documentation system's own script
# source (docs/knowledge-hub/script/, data, not implementation). This repo
# overrides via scripts/ (the repo-local Tier 2 convention resolve_check()
# already probes by name), not by hardcoding the path in Rust — a different
# repo puts its own check_name.sh here pointing wherever its scripts live.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/../../docs/knowledge-hub/script/ubuntu/08-external-context/dependency-reachable.sh" "$@"
