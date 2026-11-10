# Tasks: KMIPKit Fluent API Foundation

**Input**: [spec.md](spec.md) and [plan.md](plan.md)

**Prerequisite**: Human approval of this specification.

## Phase 1: Contract and Rust core

- [ ] T001 Record spec approval and confirm normative references before code.
- [ ] T002 [US1] Add failing protocol tests for required algorithm, length,
  usage mask, default Name Type, catalogued attribute type validation,
  explicit unknown Item, and Vendor Attribute structure.
- [ ] T003 [US1] Implement semantic attribute accumulator and symmetric-key
  request builder on existing typed protocol models.
- [ ] T004 [US1] Add failing client tests for Create/Get/Destroy builders,
  typed outcomes, Pending correlation, batch conversion, options, version
  checks, and pre-send validation.
- [ ] T005 [US1] Implement `Client::request(&mut self)` builders using existing
  typed execution methods only.

## Phase 2: C ABI

- [ ] T006 [US1] Add failing compiled C consumer tests for direct operation
  functions, versioned input validation, opaque builder, ownership, error
  snapshots, and release behavior.
- [ ] T007 [US1] Implement the direct Create/Get/Destroy adapters and opaque
  symmetric-key builder with existing ABI status and lifetime conventions.
- [ ] T008 [US1] Regenerate public API declarations/inventories and audit C
  header and binary exports.

## Phase 3: Java and Python

- [ ] T009 [US1] Add failing Java/JNI consumer coverage for the three
  operations, options, typed outcomes, cleanup, and legacy imports.
- [ ] T010 [US1] Implement the Java client facade and fluent builders over the
  existing C ABI.
- [ ] T011 [US1] Add failing Python consumer coverage for fluent/named
  arguments, typed outcomes, options, cleanup, and legacy imports.
- [ ] T012 [US1] Implement Python fluent builders and named-argument methods
  over the existing C ABI.

## Phase 4: Parity, documentation, and release checks

- [ ] T013 [US1] Extend Rust, C, Java, and Python consumers to complete
  Create/Get/Destroy against the loopback TLS 1.3/mTLS fixture.
- [ ] T014 [US1] Verify shared negative cases, ownership/lifetime behavior,
  redaction, version gates, and no-poll/no-retry execution boundaries.
- [ ] T015 [US1] Update English/Spanish guides and requirement traceability;
  regenerate and audit the API manifest and binding inventories.
- [ ] T016 [US1] Run the repository-required format, lint, Rust/C/Java/Python,
  coverage, compatibility, traceability, manifest, and export checks; record
  exact commands/results in the PR.
- [ ] T017 [US1] Rebase on `release/1.0.0`, resolve conflicts, and obtain green
  remote CI before marking the PR ready for review.

## Commit sequence

- [ ] Red: focused failing tests, in a DCO-signed commit.
- [ ] Green: minimum implementation, in a separate DCO-signed commit.
- [ ] Refactor: cleanup, docs, generated output, and evidence, in a separate
  DCO-signed commit.
