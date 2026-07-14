# MCP Configuration

This document covers configuring AI assistants and IDEs to use the Saṃgraha MCP server.

## 1. Claude Code

**Development (source repo, any platform):**

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "cargo",
      "args": ["run", "--bin", "mcp"]
    }
  }
}
```

**Windows — release binary** (built with `scripts/build-release.ps1`):

Set `OUTPUT_DIR` in `.env`, then build:

```powershell
# .env
OUTPUT_DIR=E:\MCP\Samgraha\release

# Build
.\scripts\build-release.ps1
```

The script prints the output location on completion:

```
Location: E:\MCP\Samgraha\release\samgraha
```

The binary is always at `<Location>\bin\mcp.exe`. Point Claude Code at it:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "E:\\MCP\\Samgraha\\release\\samgraha\\bin\\mcp.exe"
    }
  }
}
```

Replace `E:\\MCP\\Samgraha\\release\\samgraha` with the actual `Location` path printed by the script (which is `OUTPUT_DIR\samgraha` from your `.env`). Use double backslashes in JSON.

**Linux / Ubuntu — release binary** (built with `scripts/build-release.sh`):

Set `OUTPUT_DIR` in `.env`, then build:

```bash
# .env
OUTPUT_DIR=/home/user/mcp/samgraha/release

# Build
bash scripts/build-release.sh
```

The script prints the output location on completion:

```
Location: /home/user/mcp/samgraha/release/samgraha
```

The binary is always at `<Location>/bin/mcp`. Point Claude Code at it:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "/home/user/mcp/samgraha/release/samgraha/bin/mcp"
    }
  }
}
```

Replace `/home/user/mcp/samgraha/release/samgraha` with the actual `Location` path printed by the script (which is `OUTPUT_DIR/samgraha` from your `.env`).

The binary discovers its repo root by walking up from the process working directory looking for `.samgraha` or `samgraha.toml`. **Project-scope** config works with no extra setup — Claude Code spawns the server with `cwd` already inside the repo.

Test prompts:

- "How does Knowledge Resolution work?"
- "Search for 'repository registry'"
- "What documents are available?"


## 2. OpenCode

**Development (source repo, any platform):**

Configure MCP server in `opencode.json` (project root or global config):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "samgraha": {
      "type": "local",
      "command": ["cargo", "run", "--bin", "mcp"]
    }
  }
}
```

The `cwd` defaults to the workspace root — ensure `.samgraha/knowledge.db` and a registered repo exist there, otherwise the server exits with "Failed to open knowledge registry".

Verify the server is registered:

```bash
opencode mcp list
```

Expected: `samgraha` listed with status.

**Windows — release binary** (built with `scripts/build-release.ps1`):

Set `OUTPUT_DIR` in `.env`, then build:

```powershell
# .env
OUTPUT_DIR=E:\MCP\Samgraha\release

# Build
.\scripts\build-release.ps1
```

The script prints the output location on completion:

```
Location: E:\MCP\Samgraha\release\samgraha
```

The binary is always at `<Location>\bin\mcp.exe`. Configure in `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "samgraha": {
      "type": "local",
      "command": ["E:\\MCP\\Samgraha\\release\\samgraha\\bin\\mcp.exe"]
    }
  }
}
```

Replace `E:\\MCP\\Samgraha\\release\\samgraha` with the actual `Location` path printed by the script. Use double backslashes in JSON.

**Linux / Ubuntu — release binary** (built with `scripts/build-release.sh`):

Set `OUTPUT_DIR` in `.env`, then build:

```bash
# .env
OUTPUT_DIR=/home/user/mcp/samgraha/release

# Build
bash scripts/build-release.sh
```

The script prints the output location on completion:

```
Location: /home/user/mcp/samgraha/release/samgraha
```

The binary is always at `<Location>/bin/mcp`. Configure in `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "samgraha": {
      "type": "local",
      "command": ["/home/user/mcp/samgraha/release/samgraha/bin/mcp"]
    }
  }
}
```

Replace `/home/user/mcp/samgraha/release/samgraha` with the actual `Location` path printed by the script.

The binary discovers its repo root by walking up from the process working directory looking for `.samgraha` or `samgraha.toml`. This works for project-local `opencode.json` (cwd defaults to workspace root).

Test prompts:

- "Use the samgraha tools — how does Knowledge Resolution work?"
- "Use samgraha — search for 'repository registry'"
- "Use samgraha — what documents are available?"


## 3. Antigravity IDE

**Development (source repo, any platform):**

Configure MCP server in `~/.gemini/config/mcp_config.json`:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "cargo",
      "args": ["run", "--bin", "mcp"]
    }
  }
}
```

**Windows — release binary**:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "E:\\MCP\\Samgraha\\release\\samgraha\\bin\\mcp.exe"
    }
  }
}
```

**Linux / Ubuntu — release binary**:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "/home/user/mcp/samgraha/release/samgraha/bin/mcp"
    }
  }
}
```

*(Note: Antigravity IDE sets the `cwd` appropriately, so specifying `env` with `SAMGRAHA_REPO` is not required for the server to anchor itself.)*

## 4. Codex CLI / future IDE integrations

Test compatibility.
