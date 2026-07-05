# MCP Server Installation

## Purpose

How to set up the Samgraha MCP server.

## Content

### Prerequisites

- The `mcp.exe` binary, built from `crates/mcp` — a separate binary from `cli.exe`/`samgraha` (there is no `mcp` subcommand on the CLI binary)
- A compiled knowledge database (run `samgraha compile` first, from the repo `mcp.exe` will be pointed at)

### Starting the MCP Server

The server is not started directly by a user — an MCP client (Claude Code, OpenCode, Codex CLI, MCP Inspector) launches it as a subprocess and talks to it over stdio using line-delimited JSON-RPC 2.0. Point your client's config at the binary directly, or at the packaged `run-mcp.cmd` (Windows) / `run-mcp.sh` (Unix) launcher script if one ships alongside it:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "mcp.exe"
    }
  }
}
```

There is no port, host, or HTTP endpoint to configure — it is not a network server.

### Built-in Stores

`mcp.exe` automatically loads `help.db` and `standards.db` from next to the binary at startup, merging them into every search/get_sections call alongside the repository's own knowledge. A missing store file is skipped silently (not fatal) — `mcp.exe` still starts, and `samgraha info` will show that store as missing.

### Verification

There's no HTTP health check. Confirm the server works by sending an `initialize` request (or letting your MCP client do it) and checking `tools/list` returns the expected tool set — see [Inspector Debugging](inspector.md).

## Related

- [MCP Overview](overview.md)
- [Tools Reference](tools.md)
- [Claude Integration](claude-code.md)
