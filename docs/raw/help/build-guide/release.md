# Release Checklist

## Purpose

Checklist and process for creating a Samgraha release.

## Content

### Pre-Release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] All lint checks pass (`cargo clippy`)
- [ ] Documentation is up to date
- [ ] Built-in knowledge compiles without errors
- [ ] Changelog is updated
- [ ] Version is bumped in `Cargo.toml`

### Release Process

1. Update version in `Cargo.toml`.
2. Update changelog with release notes.
3. Run the release script:
   ```bash
   ./scripts/build-release.ps1
   ```
4. Verify the release package:
   ```bash
   ./release/samgraha/bin/cli.exe --version
   ./release/samgraha/bin/cli.exe info
   ```
5. Tag the release:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
6. Create a GitHub Release with the package archive.

### Post-Release

- Verify the MCP server starts via the launcher script (`./release/samgraha/run-mcp.cmd` on Windows, `run-mcp.sh` on Unix) — this is the recommended way to invoke it, not calling `bin/mcp.exe` directly (though that also works).
- Run a search query against built-in help to verify it's loaded (`cli.exe search "compile" --domain help`).
- Test MCP integration with Claude Code, OpenCode, or Codex CLI.

## Related

- [Build Overview](overview.md)
- [Distribution](distribution.md)
- [Version Command](../commands/version.md)
