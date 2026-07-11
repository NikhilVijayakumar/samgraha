# Domain Relationships

## Purpose

Cross-domain dependency map for the documentation standards in this directory — the agreed target model, not a description of what `crates/standards/src/builtin.rs` currently enforces. Code hasn't caught up to this yet; when it does, this file becomes the source of truth to implement against. Use it to see what a domain feeds into, or what feeds into it, without opening every standard document.

## Sections

### All Declared Relationships

A domain can have more than one parent — e.g. Feature derives from both Vision and Philosophy, not just Vision. Each parent gets its own row.

| From | Relationship | To |
|------|--------------|-----|
| vision | inspires | philosophy |
| vision | derives | feature |
| philosophy | derives | feature |
| vision | derives | security |
| philosophy | derives | security |
| philosophy | guides | architecture |
| philosophy | guides | design |
| philosophy | guides | engineering |
| security | guides | architecture |
| security | guides | engineering |
| architecture | soft-aligns-with (mutual, non-mandatory) | engineering |
| external-context | informs | engineering |
| feature | derives | feature-design |
| design | derives | feature-design |
| external-context | informs | feature-design |
| feature | derives | feature-technical |
| engineering | derives | feature-technical |
| architecture | derives | feature-technical |
| external-context | informs | feature-technical |
| prototype | validates | feature-design |
| prototype | validates | feature-technical |
| feature-technical | derives | implementation |
| engineering | derives | implementation |
| prototype | informs | implementation |
| implementation | derives | build |
| readme | references | vision |
| readme | requires | build |

`product-guide` declares no relationships — flat content describing the finished product's usage, same as `help` always has been. It isn't part of the derivation graph; it's written after everything else, referencing all of it informally rather than through a machine-readable edge.

### Traceability Chain

```text
Tier 1                  Tier 2                                              Tier 3

Vision ── inspires ──> Philosophy
   │                       │
   ├───────────────────────┼──> Security ── guides ──> Architecture
   │                       │                └─ guides ──> Engineering
   │                       │
   ├───────────────────────┴──> Feature
   │                       ├──> Architecture ──┐ soft-aligns, non-mandatory
   │                       ├──> Design         │ (most frameworks expect an
   │                       └──> Engineering ◄──┘  architecture, none require one)
   │
External Context (independent) ── informs ──> Engineering
                                └─ informs ──> Feature Design, Feature Technical

Feature ─┬─ (+ Design, + External Context if any) ──────> Feature Design ──┐
         └─ (+ Architecture, + Engineering, + External Context if any) ──> Feature Technical ─┤
                                                                                                ↓
                                                                          Tier 4 ── Prototype (validates both)
                                                                                                ↓
                                                              Tier 5 ── Implementation (+ Engineering, informed by Prototype)
                                                                                                ↓
                                                                                    Tier 6 ── Build
                                                                                                ↓
                                                          Tier 7 ── Readme (requires Build, for install/run instructions)
                                                                                                ↓
                                                                              Product Guide (requires everything, incl. Readme)
```

### Document Authoring Order — Tiered Model

Filenames in this directory carry a `NN-` prefix showing the order docs get written in, not audit priority. Docs in the same tier have no dependency on each other (with one noted exception) and can be authored in parallel.

**Tier 1 — Foundational.** No dependencies on anything else.

| # | File | Derived From |
|---|------|--------------|
| 00 | `00-domain-relationships.md` | meta — not an authored artifact |
| 01 | `01-vision-standards.md` | initial idea (no standard — see below) |
| 02 | `02-philosophy-standards.md` | vision |

**Tier 2 — Independent.** Each derives only from Tier 1, not from each other, with two exceptions: External Context informs Engineering directly, and Architecture/Engineering have a soft, non-mandatory best-practice relationship (most frameworks expect an architecture to follow, but nothing requires one to exist first).

| # | File | Derived From |
|---|------|--------------|
| 03 | `03-security-standards.md` | vision + philosophy (**draft**, not yet registered) — guides Architecture(05) and Engineering(07) downstream as a constraint, not a peer |
| 04 | `04-feature-standards.md` | vision + philosophy |
| 05 | `05-architecture-standards.md` | philosophy; soft/non-mandatory relationship with Engineering(07) |
| 06 | `06-design-standards.md` | philosophy |
| 07 | `07-engineering-standards.md` | philosophy; soft/non-mandatory relationship with Architecture(05); informed by External Context(08) |
| 08 | `08-external-context-standards.md` | independent — informs Engineering(07) directly, and informs Feature Design(09)/Feature Technical(10) downstream |

**Tier 3 — Derived.** Each combines multiple Tier 2 outputs.

| # | File | Derived From |
|---|------|--------------|
| 09 | `09-feature-design-standards.md` | feature(04) + design(06) + external-context(08) if any |
| 10 | `10-feature-technical-standards.md` | feature(04) + engineering(07) + architecture(05) + external-context(08) if any |

**Tier 4 — Validation.**

| # | File | Derived From |
|---|------|--------------|
| 11 | `11-prototype-standards.md` | feature-design(09) + feature-technical(10) — validates, doesn't derive |

**Tier 5 — Realization (draft, not yet registered).**

| # | File | Derived From |
|---|------|--------------|
| 12 | `12-implementation-standards.md` | feature-technical(10) + engineering(07) + prototype(11) |

**Tier 6 — Packaging (draft, not yet registered).**

| # | File | Derived From |
|---|------|--------------|
| 13 | `13-build-standards.md` | implementation(12) |

**Tier 7 — Final overview.** Written last, after everything above has settled.

| # | File | Derived From |
|---|------|--------------|
| 14 | `14-readme-standards.md` | vision (final refactor pass — see below); needs Build(13) to exist for install/run instructions |
| 15 | `15-product-guide-standards.md` | the finished product itself — needs everything else, including README, to be accurate |

**Initial idea has no standard.** It's not a kept artifact — a rough idea gets folded straight into Vision (01) and, downstream, into Feature (04) as one of its derivation inputs. Nothing audits the idea stage itself.

**README plays both ends.** Early on it can hold the initial idea (or a separate idea doc can be used instead) before Vision exists. Once every other standard doc in the chain is complete, README gets refactored into the structure `14-readme-standards.md` defines — including install/run instructions, which is why README sits after Build(13) rather than before Implementation(12): a README that must stay implementation-detail-free could be written earlier, but a conventional README needs build/run instructions, which don't exist until Build(13) does.

**Security, Implementation, and Build are drafts.** None has a `StandardDefinition` in `crates/standards/src/builtin.rs`, so none is enforced or audited yet.

* **Security** is fully specified: project-wide threat model, data classification, and security principles, once — Architecture's, Engineering's, and Feature Technical's own Security Considerations/Standards sections derive from it rather than duplicate it. See `03-security-standards.md`'s "Relationship to Per-Domain Security Sections" for the exact ownership split.
* **Implementation** is fully specified: the as-built, one-to-one counterpart to Feature Technical, distinct from Engineering's repo-wide Code Standards by scope (per-feature vs repo-wide). See `12-implementation-standards.md`'s "Relationship to Engineering's Code Standards."
* **Build** is fully specified: project-wide versioning/packaging/distribution/provenance policy, distinct from Engineering's Build Standards section, which stays scoped to CI/CD mechanics. See `13-build-standards.md`'s "Relationship to Engineering's Build Standards."

## Usage

Check this before adding a new relationship to a domain — if the relationship isn't in the table above, add it here too, otherwise this document drifts out of sync with itself the same way it drifted out of sync with code before this rewrite. When `security`, `implementation`, and `build` get registered as real `StandardDefinition`s in `crates/standards/src/builtin.rs`, that code should be written to match this file, not the other way around.
