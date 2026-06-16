# phenotype-pm-core

Shared PM/traceability **spine** for [AgilePlus](https://example.invalid/AgilePlus) and [Tracera](https://example.invalid/Tracera).

This workspace is a **superset merge** of:

- **tracera-core** — artifact graph, requirements, trace links, coverage matrix, impact analysis
- **agileplus-domain** — intent graph ontology, feature lifecycle, governance contracts

Merge decisions are recorded in [`docs/adr/ADR-0001-superset-merge.md`](docs/adr/ADR-0001-superset-merge.md).

## Workspace layout

```
phenotype-pm-core/
├── Cargo.toml                 # workspace root
├── crates/
│   └── traceability-core/     # unified domain crate
└── docs/adr/
    └── ADR-0001-superset-merge.md
```

## `traceability-core` modules

| Module | Primary consumer | Responsibility |
|--------|------------------|----------------|
| `ids` | Both | `FR-` / `NFR-` requirement identifiers |
| `artifact` | Tracera | `Artifact`, `ArtifactKind`, `ArtifactRef` |
| `requirement` | Both | `Requirement`, ISO 29148 status, verification method |
| `tracelink` | Tracera | `TraceLink` + 7 link types + confidence |
| `matrix` | Tracera | `CoverageMatrix`, `build_matrix`, coverage states |
| `impact` | Tracera | Blast-radius / impact scoring |
| `intent_graph` | AgilePlus | Intent DAG ontology + validation |
| `lifecycle` | AgilePlus | `FeatureState` 8-stage machine |
| `governance` | AgilePlus | Policies, evidence, contracts |
| `contract` | Both | `AcceptanceContract`, `ProgressionGate` |

## Build & test

```bash
cargo build
cargo test
```

## Consumers

**AgilePlus** (authoring) typically imports:

`lifecycle`, `governance`, `intent_graph`, `contract`, `requirement`, `artifact`, `ids`

**Tracera** (live service) typically imports:

`artifact`, `requirement`, `tracelink`, `matrix`, `impact`, `ids`

…and reads `contract` / `governance` for progression gate evaluation.

## License

MIT OR Apache-2.0 (see workspace `Cargo.toml`).
