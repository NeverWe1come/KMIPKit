# Implementation Plan: Local Cosmian KMS Integration Tests

**Branch**: `feature/KMIPKIT-0015-cosmian-operations` | **Date**: 2026-10-08 (exhaustive amendment 2026-10-16; multipart amendment 2026-10-17) | **Spec**: [spec.md](spec.md)

**Input**: [Approved matrix design](../../docs/superpowers/specs/2026-10-16-cosmian-exhaustive-matrix-design.md), [specification](spec.md), [execution plan](../../docs/superpowers/plans/2026-10-16-cosmian-exhaustive-matrix.md).

## Summary

Expand the opt-in local Cosmian 5.28.0 suite from its historical twenty-case baseline to every source-derived valid intersection for the 35 advertised operation paths (34 published rows, with Set/Modify separate). Cover applicable objects, default/explicit formats, fixture/vendor encodings, algorithms and parameters through the existing typed Rust client. Preserve Query's known response defect and separate Ping, Adjust Attribute and Re-certify discrepancy probes. Every measured result remains individual; a passing finding assertion never becomes interoperability success.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; PowerShell 7-compatible local runner; POSIX shell/OpenSSL fixture generator.

**Primary Dependencies**: Existing `kmipkit-client`, `kmipkit-protocol`, `kmipkit-transport`; Docker Compose; `alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`; `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`. Use the existing workspace-pinned `serde_json` as a dev dependency only if needed.

**Source identity**: Cosmian annotated tag object `cec59a003b9cfc4f253ca239fde42925988700fa`, dereferenced commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`. OASIS immutable local sources define normative validity; exact source ledger and disagreements are in `research.md`.

**Storage**: Disposable container SQLite, cleared at startup and removed at teardown; ignored PKI/test fixtures under `.local/cosmian-kms/`.

**Testing**: Retain `cosmian_kms` integration target; split ignored network cases by family. Offline schema, source-inventory completeness and case/test parity checks must not contact Docker or any server.

**Target Platform**: Local Docker Desktop/Engine and Rust 1.94+ host.

**Constraints**: Existing raw TTLV/TLS 1.3/mTLS with hostname/chain verification; loopback-only 5696; no automatic retry, external service, sensitive output or committed credential. No production API/protocol changes except the maintainer-authorized KMIP 2.1 multipart input/output correction recorded as FR-017. Imports use only test-generated material. Destroy targets only a unique object that case created. Stop/remove all Compose services after any live run, including failure.

**Scale/Scope**: Exact 35-path inventory and all applicable source-derived dimensions from `spec.md`; each case owns setup/linked objects and reports a stable ID. The inventory is a valid intersection, not all mathematical combinations. Documented-but-source-gap rows remain visible; unrepresentable requests become client blockers rather than alternate wire implementations.

## Constitution Check

- **Specification and traceability**: The amendment defines FR-011 through FR-016 and SC-005 through SC-009, keeps previous identifiers stable, and adds exact OASIS operation/data citations. Concrete matrix rows must refine sources before acceptance.
- **Test first and evidence**: Keep historical Red/Green/Refactor evidence and measured baseline distinct. The multipart amendment adds Red/Green/Refactor evidence for FR-017; the new protocol behavior is covered by unit contract tests before the live Cosmian rerun.
- **Core and protocol boundaries**: All 35 paths and probes already have `ClientRequest` variants. No public operation/API expansion is needed. The explicitly authorized FR-017 exception is limited to typed multipart input/output handling; unrelated production defects remain out of scope.
- **Security**: Test-generated fixtures and independent state only; no persistent data mount; cleanup/teardown even after failure; no request retry; no secret/raw-body logging. Existing suite is mutating, not read-only.
- **Human-governed workflow**: Reuse the authorized dedicated worktree/feature branch. Existing authorization permits the expanded local work; only humans approve/merge/release. Retain DCO sign-off and the separate draft-PR review process.

## Design and sequencing

1. Amend specification, research, plan, tasks, traceability and `docs/development/kmip-interoperability.md` before implementation. Run the actual CI traceability contracts listed below; `docs/development/testing.md` does not document these commands.
2. Add an offline failing manifest-completeness validator and initially empty JSON array. Assert the exact 35 supported paths, Query's defect classification and separation of probes. Capture failure before adding inventory rows.
3. Build `tests/integration/cosmian/capability-matrix.json` from pinned vendor sources and exact OASIS clauses. Each row has a stable ID, FR links, operation/object, algorithm/parameters, format/encoding, setup, normative/vendor citations, expected outcome, test/family and observed result. Only inapplicable dimensions use null. An exclusion/disagreement ledger must prevent silent omission.
4. Cover every valid operation/object intersection, including lifecycle, registration, certificate/CSR and random operations. Inspect each operation's restrictions rather than treating managed-object support as primitive support. Set/Modify remain separate; Query remains defective until successful required-field-complete parsing is measured. Probe Re-certify separately because its dispatcher and support table disagree.
5. Add Import/Export defaults, explicit formats, ASN.1 PEM/DER paths and source-only conversions, including PrivateKey/PFX. Preserve `formats.md` omissions, source-only legacy/PKCS7 paths and encoding mismatches as evidence. Separate fixture conversion from actual wire bytes; standard enumerations/DER requirements remain authoritative. Generate fixture material locally and track all PFX-linked objects.
6. Enumerate every applicable documented or source-implemented generation, Certify, encryption, hash, HMAC, signature, derivation and wrapping variant. Inspect delegated helpers, including HMAC/derivation gaps. Include Hash/SHA-224 as a documentation/source discrepancy case. Use matching explicit keys/parameters for round trips and verification. Certify's actual issuer/digest behavior constrains its cases. IV Length is in bits, Tag Length in bytes (§4.16); do not invent a unit defect.
7. Refactor the large target into family modules sharing only typed-client, independent fixture and sanitized reporting helpers. Validate one-to-one case/test coverage and ordinary-run ignore behavior, without excluding offline validators.
8. Execute the pinned local suite serially and record every actual case result and blocker with source/test ID and client/server metadata. Always run Compose down and status after the run, including failure. Preserve original failure if cleanup also fails; never count rejection or malformed data as a successful exchange.
9. Update English/Spanish entry points, run guides and traceability with measured evidence. Keep historical selected cases distinct from exhaustive results. Synchronize with release, complete required checks and leave final integration acceptance to human review.

## Planned file layout

```text
crates/kmipkit-client/tests/cosmian_kms.rs
crates/kmipkit-client/tests/cosmian_kms/
  common.rs
  lifecycle.rs
  transfer.rs
  key_management.rs
  random.rs
  cryptographic.rs
tests/integration/cosmian/
  capability-matrix.json
  compose.yaml
  cosmian-kms.toml
  generate-pki.sh
  README.md
scripts/integration/cosmian-kms.ps1
specs/015-cosmian-kms-integration/{spec,research,plan,tasks,traceability,quickstart}.md
docs/development/kmip-interoperability.md
docs/user-guide/en/production-transports.md
docs/user-guide/es/transportes-produccion.md
```

`crates/kmipkit-client/Cargo.toml` changes only if a manifest-validation dev dependency is needed. The focused operation-family modules, versioned manifest, offline completeness/parity tests and ignored live runners are implemented. Generated fixtures and full-run logs remain ignored and sanitized under `.local/cosmian-kms/`.

## Verification Plan

For this documentation amendment, run the two existing CI contract commands from `.github/workflows/ci.yml`:

```text
python3 -B -m unittest scripts.tests.test_requirement_traceability -v
python3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v
```

Review every amended FR/SC against the approved design, typed request variants and exact local OASIS clauses; self-review the full diff and run `git diff --check`. These existing suites cover selected repository traceability contracts; they do not prove the new matrix inventory or live coverage before those checks/cases are implemented.

For subsequent implementation, record distinct Red/Green/Refactor evidence. Run offline manifest completeness/parity and existing focused client/transport tests, syntax/Compose checks, ordinary Cargo ignore behavior, and `cargo fmt --all --check`. Run the live target through `scripts/integration/cosmian-kms.ps1 -Action test` (serial ignored tests), followed by `-Action down` and empty `-Action status`, including on failure. Retain exact per-case results and image digest. Complete workspace Clippy, tests, coverage, traceability, immutable-source, generated-output, dependency/security and compatibility gates required by the repository before requesting review; no historical pass substitutes for a current final-head result.

## Multipart scope amendment — authorized 2026-10-17

FR-017 permits caller-provided multipart input data where the operation supports it and accepts
optional output data for multipart Hash, MAC, and Sign responses. Preserve required output on
single-part responses and all TTLV structural checks. OASIS KMIP v2.1 §6.1 defines multipart request
sequencing, §7.8 requires reuse of the first response's Correlation Value, and Hash §6.1.24, Table
235 marks Data as not required for multipart Hash. Section 6.1 also says Data is always required
except when Init or Final Indicator is true. The project records this source interpretation as
unresolved and makes no conformance claim for an intermediate Hash request that omits Data. The
Cosmian Hash test uses a, b, and c as explicit input blocks, reuses the initial Correlation Value,
and validates a returned final digest against the independent abc known-answer vector; it does not
exercise the disputed omission case.

## Evidence and risks

The historical 2026-10-15 baseline had twenty passing test cases, eighteen successful typed exchanges and two finding probes. The exhaustive matrix now contains 6,429 rows, including 6,417 executable cases with individual live classifications/blockers and 12 normative exclusions; 3,728 executable rows are successful typed interoperability outcomes, including 24 source-discrepancy Create Key Pair probes and 54 source/documentation transfer probes whose operation postconditions passed; their discrepancy classifications remain and they are not KMIP 2.1 conformance claims. The targeted follow-up measured the 15 former cipher fixture blockers as 12 typed finding responses and three target rejections. A subsequent review corrected three AES-ECB fixtures for both Encrypt and Decrypt and the CoverCrypt framing assertion; Encrypt now records 287/300 interoperable cases, Decrypt 222/234, Get 797/1,279, and Export 807/1,291. The 50 legacy-PKCS#12 Get/Export validation failures were caused by the test omitting a vendor format selector; a targeted rerun resolved them. The 446-case Register follow-up later recorded 154 target successes, 286 setup rejections and six target rejections; Register is now 737/1,029 (full attribution in `task-6-report.md`). Remaining server rejections and support-clarification findings remain in the ledger. The latest complete serial suite with rustls 0.23.46 ended with 101 passed and 9 failed functions after 6,383.81 seconds, and Compose teardown was verified. The failures remain in the matrix and final review gates remain open. Inherited Response Message/Create/Split Key fixes and KMIPKIT-0021 shared-tag corrections are historical production work; do not duplicate or expand them here. The earlier hosted dependency-policy failure on `rustls 0.23.45` was addressed by the approved KMIPKIT-0045 release update; the final hosted CI run 39007889527 passed on commit `056f45a5`.

Vendor docs, source and runtime can disagree. Preserve both sources and actual outcome, including source-only formats and documented-but-source-gap algorithms. Missing fixtures, local API representation gaps or service prerequisites are explicit blockers. They do not justify weakening typed parsing, fabricating standard values, discarding valid advertised cases or making profile/certification claims.
