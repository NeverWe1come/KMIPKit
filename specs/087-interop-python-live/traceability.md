# Traceability: KMIPKIT-0087

| Requirement | Implementation evidence | Verification |
|---|---|---|
| 0087-FR-001 | Python catalog planner and offline preflight | `test_plan.py`, `test_preflight.py` |
| 0087-FR-002 | Pinned wheel installer and provenance manifest | artifact installer/verification tests and CI |
| 0087-FR-003 | Catalog dispatcher and explicit `not_run` results | catalog-to-dispatch completeness test |
| 0087-FR-004 | Connection validation and mutation gate | configuration and CLI tests; controlled TLS tests |
| 0087-FR-005 | Schema-2.0 report writer | shared JSON Schema validation |
| 0087-FR-006 | Public API model and equivalence checks | request/response TTLV tests |
| 0087-FR-007 | Python 3.12 installed-wheel workflow | GitHub Actions `Interop Python Validator` |

Normative source: OASIS KMIP 2.0/2.1 Specifications §§2–12 and the matching
OASIS Profiles transport/encoding clauses. Exact operation clauses are carried
by each shared catalog row.
