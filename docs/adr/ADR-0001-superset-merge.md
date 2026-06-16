# ADR-0001: Superset merge of tracera-core + agileplus-domain

**Status:** Accepted  
**Date:** 2026-06-15  
**Context:** `phenotype-pm-core` is the shared PM/traceability spine consumed by **AgilePlus** (authoring) and **Tracera** (live traceability service). Both codebases had overlapping but non-identical domain models.

## Decision summary

| Area | Winner | Notes |
|------|--------|-------|
| Requirement IDs | **Tracera** (`ids.rs`) | `RequirementId` / `NfrId` newtypes; AgilePlus `fr_id: String` at boundary via `to_fr_id_string()` |
| Artifact graph nodes | **Tracera** (`artifact.rs`) | `Artifact` + `ArtifactKind` (7 variants) + `ArtifactRef` |
| Requirements | **Tracera** (`requirement.rs`) | ISO 29148 status + DO-178C `VerificationMethod`; free-form `acceptance_criteria` kept |
| Trace links | **Tracera** (`tracelink.rs`) | 7 link types + `confidence: f32`; split from monolithic `lib.rs` |
| Coverage matrix | **Tracera** (`matrix.rs`) | `CoverageMatrix` / `CoverageState` / `build_matrix` |
| Impact analysis | **Tracera** (`impact.rs`) | BFS blast-radius + weighted scoring |
| Intent ontology | **AgilePlus** (`intent_graph.rs`) | `NodeType`, `DagStage`, `RelationshipType`, graph validation |
| Feature lifecycle | **AgilePlus** (`lifecycle.rs`) | 8-stage `FeatureState` linear machine |
| Governance | **AgilePlus** (`governance.rs`) | `GovernanceContract`, `PolicyRule`, `EvidenceType`, `BuiltinPolicy` |
| Acceptance & gates | **NEW** (`contract.rs`) | `AcceptanceContract` + `ProgressionGate` hybrid |

## §1 Artifact kinds vs intent node types (hybrid)

- **Tracera wins** for graph persistence: `ArtifactKind` (Requirement/Design/Code/Test/Evidence/Risk/Rationale).  
  Source: `Tracera/crates/tracera-core/src/lib.rs:86-97`
- **AgilePlus wins** for authoring ontology: `NodeType` (Intent/Plan/Feature/Story/Task/Spec/Commit/Test/PR/Bug/Artifact).  
  Source: `AgilePlus/crates/agileplus-domain/src/intent_graph.rs:17-29`
- **Boundary:** map at integration via `ArtifactRef::kind_str()` and external_id mirrors; no lossy merge into one enum.

## §2 Requirement identifiers

- **Tracera wins** internally: `ids.rs` newtypes with `FR-` / `NFR-` prefixes.  
  Source: `Tracera/crates/tracera-core/src/ids.rs`
- **Hybrid at boundary:** `from_fr_id_string()` / `to_fr_id_string()` for AgilePlus string fields.  
  Source: `crates/traceability-core/src/ids.rs`

## §3 TraceLink vocabulary

- **Tracera wins** verbatim: Implements, Verifies, Duplicates, Satisfies, DerivesFrom, ConflictsWith, Refines + confidence.  
  Source: `Tracera/crates/tracera-core/src/lib.rs:46-57, 184-239`
- AgilePlus `CanonicalLinkType` / `RelationshipType` remain in `intent_graph` for ontology edges; Tracera link types are the **persistence** vocabulary.

## §4 Requirement status vs feature lifecycle

- **Tracera wins** for requirements: `RequirementStatus` (Draft→Verified).  
  Source: `Tracera/crates/tracera-core/src/lib.rs:110-121`
- **AgilePlus wins** for features: `FeatureState` 8-stage machine (Created→Retrospected).  
  Source: `AgilePlus/crates/agileplus-domain/src/domain/state_machine.rs:13-22`
- These are **orthogonal** lifecycles; no merge into one enum.

## §5 VerificationMethod vs EvidenceType (hybrid)

- **Tracera wins** for *how* verification is performed: Test/Analysis/Inspection/Demonstration/Review.  
  Source: `Tracera/crates/tracera-core/src/lib.rs:123-132`
- **AgilePlus wins** for *what* evidence artifact exists: TestResult/CiOutput/ReviewApproval/SecurityScan/LintResult/ManualAttestation.  
  Source: `AgilePlus/crates/agileplus-domain/src/domain/governance.rs:74-97`
- Mapping left to consumers; `contract` gates use matrix coverage + evidence presence.

## §6 AcceptanceContract & ProgressionGate (new)

- **New in this crate:** `AcceptanceContract { artifact_ref, criteria, verification, bdd }`.  
  Satisfied **only** when every `Criterion` maps to `CoverageState::Covered` in the matrix.
- **New:** `ProgressionGate` over layers Intent → IntentDoc → SpecAdr → PlanWbs → Execution → Evidence with predicates: `not_approved`, `missing_acceptance`, `missing_evidence`, `missing_implementation`, `missing_test` (AgilePlus governance vocabulary).
- Tracera free-form `acceptance_criteria: Vec<String>` preserved; promotable to `Criterion` lazily.

## §7 Coverage matrix

- **Tracera wins** entirely.  
  Source: `Tracera/crates/tracera-core/src/matrix.rs`, `lib.rs:291-316`
- States: Covered / Partial / Missing / Stale / Conflict.

## §8 Impact analysis

- **Tracera wins** (included because `lib.rs` public API already exports it for Tracera consumers).  
  Source: `Tracera/crates/tracera-core/src/impact.rs`

## §9 Lifecycle errors

- **Hybrid:** AgilePlus `DomainError::InvalidTransition` replaced with local `LifecycleError` so the spine crate has no AgilePlus error dependency.

## Consequences

- AgilePlus imports: `lifecycle`, `governance`, `intent_graph`, `contract`, `requirement`, `artifact`, `ids`.
- Tracera imports: `artifact`, `requirement`, `tracelink`, `matrix`, `impact`, `ids`; reads `contract`/`governance` for gate evaluation.
- Single crate `traceability-core` under workspace `phenotype-pm-core`.
