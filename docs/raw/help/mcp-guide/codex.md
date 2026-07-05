# Connecting Codex CLI

## Purpose

How to connect Codex CLI to the Samgraha MCP server.

## Content

### Configuration

Codex CLI supports MCP tool integration. Configure the Samgraha MCP server in your Codex CLI settings, pointing `command` at the `mcp.exe` binary directly (there is no `samgraha mcp` subcommand — `cli.exe`/`samgraha` and `mcp.exe` are separate binaries):

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "mcp.exe"
    }
  }
}
```

### What Codex Can Do

Once connected, Codex CLI can:

- Access project documentation as context during code generation
- Search for relevant feature specs when implementing new code
- Check architecture constraints before suggesting solutions
- Look up engineering standards for code style compliance

## Related

- [MCP Overview](overview.md)
- [Claude Integration](claude-code.md)
- [OpenCode Integration](opencode.md)
- [Tools Reference](tools.md)
