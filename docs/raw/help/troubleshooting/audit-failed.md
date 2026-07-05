# Audit Failed

## Purpose

Common audit failures and how to resolve them.

## Content

### Problem: "Required section missing"

**Cause**: A required section (as defined by the standard) is not present in the document.

**Solution**: Add the missing section with the correct heading.

```
feat-001 (error): Missing Purpose section
→ Add ## Purpose to the document
```

### Problem: "Gate failure — score below threshold"

**Cause**: The overall quality score is below the gate threshold.

**Solution**:
1. Run `samgraha audit --report` to see all findings.
2. Fix findings starting with errors (highest severity).
3. Recompile: `samgraha compile`
4. Re-audit: `samgraha audit --gate 80`

### Problem: Unexpected findings from the semantic provider

**Cause**: The `semantic` provider (`--provider semantic`) is currently a heuristic checker (short-document, vague-language/placeholder-text, and scope checks) — it does not call out to an AI model and has no API key requirement. Findings like "Document is very short" or "contains placeholder text" come from these heuristics, not an LLM judgment.

**Solution**: If the heuristic findings aren't useful for a given document, run deterministic checks only (the CLI default):
```bash
samgraha audit --provider deterministic
```

## Related

- [Command: audit](../commands/audit.md)
- [Audit Guide: Fixing Findings](../audit-guide/fixing-findings.md)
- [Command: compile](../commands/compile.md)
