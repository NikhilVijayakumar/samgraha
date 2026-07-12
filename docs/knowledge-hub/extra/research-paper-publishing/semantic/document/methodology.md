# Methodology Document Audit

This section details the Methodology Document Audit.

## Version
1.0.0

## Engineering Intent
Verifies a research paper's document-level properties — literature references, target audience, terminology, and claim coverage — hold across the whole paper, not just within the Methodology section itself. Section-level reproducibility and scope checks are owned by `audit/semantic/section/methodology/`; this file owns what only makes sense read against the full document.

## Audit Objectives
- Literature references are consistent and complete across the whole paper, not just cited once and forgotten
- Target audience is identifiable from the paper as a whole (who is expected to read and act on this)
- Terminology is used consistently across every section — a term introduced in one section isn't redefined or silently swapped for a synonym elsewhere
- Every claim made in the paper has coverage — supported by cited evidence, data, or the methodology itself

## Expected Quality
- Reference list is complete and every in-text citation resolves to an entry in it
- Audience is inferable from framing, prerequisite knowledge assumed, and venue conventions
- A term (e.g. "accuracy," "significant") means the same thing everywhere it's used
- Claims in the Abstract/Conclusion are traceable back to a result or citation in the body

## Red Flags
- Citations appear in text with no corresponding reference list entry, or vice versa
- Terminology shifts meaning across sections without acknowledgment (e.g. "significant" used both statistically and colloquially)
- Strong claims in Abstract/Conclusion unsupported by anything in Results or Methodology
- Paper mixes audience levels inconsistently (e.g. assumes expert statistics knowledge in one section, explains basics in another)

## Edge Cases
- Interdisciplinary papers with genuinely mixed audiences — acceptable if the paper explicitly scaffolds for both, not just inconsistent by accident
- Preprint/working paper with acknowledged incomplete claim coverage — acceptable if stated as a limitation, not silently unsupported

## Scoring Criteria

| ID | Weight | Score | Description |
|---|---|---|---|
| C1 | mandatory | 0 or 30 | All citations resolve to reference list entries and vice versa |
| C2 | mandatory | 0 or 30 | Terminology is consistent across all sections |
| C3 | mandatory | 0 or 20 | Claims in Abstract/Conclusion are traceable to supporting evidence in the body |
| C4 | recommended | 0 or 20 | Target audience is identifiable and consistently addressed |

Score = sum of passed criterion scores, capped at 100.
Mandatory criterion failure = ERROR. Recommended = WARNING.

## Output Schema
```json
{
  "criterion_id": "C3",
  "passed": false,
  "confidence": 0.84,
  "severity": "error",
  "evidence": { "section_id": 1, "paragraph_index": 0, "excerpt": "Our method achieves state-of-the-art results across all benchmarks." },
  "message": "Abstract claim of state-of-the-art results is not traceable to a specific supporting result in the body."
}
```
