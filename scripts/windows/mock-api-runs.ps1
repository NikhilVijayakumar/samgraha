# Thin wrapper — delegates to the samgraha-documentation system's own script
# source (docs/knowledge-hub/script/, data, not implementation). This repo
# overrides via scripts/ (the repo-local Tier 2 convention resolve_check()
# already probes by name), not by hardcoding the path in Rust — a different
# repo puts its own check_name.ps1 here pointing wherever its scripts live.
& "$PSScriptRoot/../../docs/knowledge-hub/script/windows/11-prototype/mock-api-runs.ps1" @args
exit $LASTEXITCODE
