# ADR: trace-gate — CI Enforcement for the PM-Spine Traceability Pipeline

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** BLOCK A architecture council (AgilePlus, Tracera, phenotype-pm-core)

---

## Context

The phenotype PM spine enforces an **intent → spec/ADR → plan/WBS → execution → evidence** pipeline (see `traceability-core` `Layer` enum and `ProgressionGate`). The pipeline is only as strong as its weakest enforcement point: without CI enforcement, developers can ship code that satisfies no tracked functional requirement, breaking requirements traceability silently.

Current state:
- `traceability-core` defines `AcceptanceContract`, `CoverageMatrix`, `CoverageState`, and `ProgressionGate` — the data model is complete.
- `Tracera` ingests and stores trace links via its REST API.
- **Missing:** a lightweight, zero-infrastructure CI gate that any BLOCK A repo can adopt in one line, without requiring a running Tracera instance.

---

## Decision

We introduce **`trace-gate`** as the CI enforcement mechanism, backed by **`traceability-decorators`** as its scanner library.

### Architecture

```
Consumer repo CI
    │
    ▼
trace-gate --manifest trace-gate.toml --src src/
    │
    ├── traceability-decorators::scan_dir()
    │       │  regex walk over *.rs, *.ts, *.go, *.py, …
    │       │  emits ScanTraceLink { fr_id, spec_id, file, line, symbol }
    │       ▼
    ├── CoverageSummary::build()  (uses traceability-core::CoverageState)
    │       │  FR manifest ↔ scan hits → Covered | Missing
    │       ▼
    ├── exit 0   (all FRs covered)
    │   exit 1   (any FR missing → CI fails)
    │   exit 2   (usage / manifest error)
    │
    └── --push <url>  (best-effort Tracera ingest; never affects exit code)
```

### Supported annotation patterns

| Language | Pattern | Example |
|---|---|---|
| Rust | `#[trace_fr(spec="SPEC-001", fr="FR-001")]` | Attribute macro form |
| Any | `// FR: FR-001` | Generic line comment |
| TS/Go | `// @trace_fr spec="SPEC-001" fr="FR-001"` | Block-comment attribute form |
| Python | `# FR: FR-001` | Hash comment form |

### Manifest format (`trace-gate.toml`)

```toml
[[requirement]]
fr_id = "FR-001"
description = "User can log in"
spec_id = "SPEC-001"

[[requirement]]
fr_id = "FR-002"
description = "User can register"
```

### One-line adoption snippet

```yaml
# In .github/workflows/ci.yml of any BLOCK A consumer repo:
jobs:
  trace-gate:
    uses: KooshaPari/phenotype-pm-core/.github/workflows/trace-gate.yml@trace-gate-v1
```

---

## Versioning & Adoption

The trace-gate pipeline merged to `master` at SHA `6064d30185696459c583a64c851320f99be2b00a`. To prevent silent staleness for consumers, we adopt **tag-pinned distribution** rather than branch-tracking.

### Branch & release strategy

- **`master` is the single long-lived branch.** All trace-gate work merges to `master`. There is no parallel `main` snapshot branch — it was a one-time snapshot of the merge SHA and has been deleted to avoid a silent-staleness trap (consumers pinning `@main` would silently freeze at the snapshot SHA forever).
- **Consumers pin a TAG, never `@main` or `@master`.** Pinning `@master` would silently pull unreviewed changes; pinning `@main` would silently freeze. A tag gives consumers an explicit, immutable, reviewable version they upgrade on purpose.
- **Each merged change that affects the reusable workflow gets a new `trace-gate-vN` tag + GitHub release.** The release notes describe what changed so consumers can decide whether to bump their `@trace-gate-vN` pin. Tags are immutable; we never move a published tag.

| Version | SHA | Notes |
|---|---|---|
| `trace-gate-v1` | `6064d301` | Decorator scanner + CI coverage gate + reusable workflow (initial adoption pipeline) |

### One-line adoption snippet (tag-pinned)

```yaml
# In .github/workflows/ci.yml of any BLOCK A consumer repo:
jobs:
  trace-gate:
    uses: KooshaPari/phenotype-pm-core/.github/workflows/trace-gate.yml@trace-gate-v1
```

To upgrade later, bump the tag suffix (e.g. `@trace-gate-v2`) after reviewing that release's notes.

---

## Consequences

### Positive
- Zero runtime infrastructure: `trace-gate` is a single statically-compiled binary; no Tracera connection required for the gate to run.
- Additive: existing repos add a `trace-gate.toml` + one workflow `uses:` line; nothing is deleted or renamed.
- Unified vocabulary: `CoverageState::Missing` / `Covered` come directly from `traceability-core`, keeping the data model coherent with Tracera's matrix.
- `--push` stub enables future Tracera ingest without a flag change in consumer CI.

### Negative / Trade-offs
- Annotation discipline: developers must add `#[trace_fr(...)]` or `// FR: ...` comments. Enforcement of this is cultural + PR-review until a linter is added.
- Regex-only: proc-macros or LSP-based extraction would be richer, but are overkill for the initial adoption phase.
- `--push` is a stub until Tracera #754 (ingest endpoint stabilisation) merges.

### AgilePlus Rollout

AgilePlus (`C:/Users/koosh/Dev/AgilePlus`) is the first consumer target. The rollout is gated on Tracera #754 for the `--push` wire-up; the gate-only (exit-code) mode can be adopted immediately by adding `trace-gate.toml` to the repo root and the workflow `uses:` line to CI.

```
AgilePlus adoption checklist:
[ ] Add trace-gate.toml listing all Epic/Story-derived FR IDs
[ ] Annotate implementation functions with // FR: <id> or #[trace_fr(...)]
[ ] Add workflow uses: KooshaPari/phenotype-pm-core/.github/workflows/trace-gate.yml@trace-gate-v1
[ ] (post #754) Add --push https://tracera.prod/api/v1/coverage/ingest to workflow inputs
```

---

## Alternatives Considered

| Alternative | Rejected because |
|---|---|
| Proc-macro only | Rust-only; BLOCK A has TS/Go/Python services |
| Tracera-first (require live API) | Breaks offline / PR CI; circular dependency on Tracera uptime |
| SonarCloud custom rules | Can't query FR manifest; needs org-scope token (known issue) |
| Inline in AgilePlus only | PM-spine intent is cross-repo; needs shared home in `phenotype-pm-core` |

---

*Co-authored with phenotype-pm-core PM spine team. Superset-merge decision: unify AgilePlus + Tracera traceability models into one gate (not pick one, drop the other).*
