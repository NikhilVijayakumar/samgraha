# Philosophy — Lattice

## Purpose

This document establishes Lattice's decision-making philosophy — the principles, values, and trade-offs that guide every downstream choice.

- **Vision says:** Why Lattice exists and what problem it solves for teams.
- **Philosophy says:** How the team building Lattice chooses to think, prioritize, and decide.

**Scope boundaries:**
- **In scope:** Guiding principles, named values with priority rankings, explicit trade-offs
- **Out of scope:** Feature specifications, architectural decisions, implementation choices

## Principles

### Simplicity First

When two designs solve the same problem, choose the simpler one. Complexity is a cost, not a feature.

If adding a framework means the team must learn a new paradigm, it must clearly reduce complexity in the rest of the system to justify itself.

### Link Before You Describe

When adding a new piece of work, connect it to existing context first — related discussions, prior decisions, or parent goals — before writing a description. Context accumulation beats context creation.

If a team member starts a new task and immediately begins writing a description, the better first step is to link it to the epic, discussion, or decision that spawned it.

### Make the Implicit Explicit

If a team is likely to wonder "why did we decide this?" or "what changed?", capture that proactively rather than waiting for someone to ask.

When a design review ends without a written decision, the next step is to capture the decision and its reasoning before moving on — not to rely on memory.

### Preserve Flow, Don't Force Structure

Structuring work should happen within the flow of collaboration, not as a separate step that breaks momentum.

When a team is deep in discussion and needs to create a work item, the creation should happen inline — without leaving the conversation or switching tools.

## Values

### Context Integrity

We prioritize preserving the full context of every decision over making any individual action faster. Teams lose more time to lost context than to slow tools.

This sometimes means adding an extra step to capture context that could otherwise be skipped.

### Team Autonomy

We optimize for each team's ability to work the way that suits them, even when that means supporting multiple approaches to the same problem.

This sometimes means maintaining flexibility that adds complexity to the system.

### Progressive Disclosure

We surface information at the level of detail the user needs right now, hiding complexity until it is relevant.

This sometimes means building two interfaces to the same data — one simple, one detailed — instead of one interface that tries to be both.

## Trade-offs

### Completeness vs. Speed

**Chosen:** Capturing full context for every decision and work item, even when it takes extra time.
**Sacrificed:** The speed of getting started with a new tool — there is more to set up initially.
**Reason:** Our value of Context Integrity demands we invest upfront; teams recover that time many times over when they never have to re-explain context.

### Flexibility vs. Simplicity

**Chosen:** Allowing teams to configure workflows, naming, and structure to match how they already work.
**Sacrificed:** A single, opinionated default experience that requires zero configuration.
**Reason:** Our value of Team Autonomy means we adapt to the team, not the other way around — even though a rigid default would be simpler to build.
