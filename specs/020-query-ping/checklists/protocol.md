# Protocol Review Checklist: KMIPKIT-0020

**Purpose**: Independent review of normative completeness and protocol behavior

**Created**: 2026-10-09

**Feature**: [spec.md](../spec.md)

**Review Ownership**: Independent protocol reviewer. Mark an item [x] only after checking it against the pinned OASIS source.

**Marker Semantics**: [x] means this review criterion was satisfied; it is not implementation-completion evidence.

- [ ] CHK001 Ping request and response payload are empty per §6.1.36, Tables 271–272.
- [ ] CHK002 Query request contains at least one Query Function and supports repetition per §6.1.40, Table 282.
- [ ] CHK003 Optional Object Groups from Table 282 are represented as the §7.23 Table 375 structure with zero or more repeated Object Group attributes, encoded as Text String values per §4.35 Tables 99–100.
- [ ] CHK004 All 14 standard Query Function values and extension range from §11.44, Table 476 are accounted for.
- [ ] CHK005 Every response member, conditional requirement, optionality, and repetition in Table 283 is accounted for.
- [ ] CHK006 Query Extension List/Map precedence is treated as server behavior.
- [ ] CHK007 The §6.1.40 empty-response prose and Table 283 required Protection Storage Masks field conflict is recorded as `KMIPKIT-DISC-047`; the client contract accepts both forms without asserting server conformance.
- [ ] CHK008 Query-specific Operation Failed reasons match §6.1.40.1, Table 284.
- [ ] CHK009 Unknown enumeration values and extension Items remain lossless.
- [ ] CHK010 Common batch, result, transport-delivery, limits, no-asynchronous-operation, and no-retry behavior is preserved.
- [ ] CHK011 OASIS cases are cited accurately with fixture/mapping limitations and without profile claims.
- [ ] CHK012 Source hierarchy conflicts are recorded, client behavior remains deterministic, and no server-conformance claim is made for an unresolved case.

## Notes

- Record any source disagreement with exact documents, sections, clauses, and downstream effect.
- Do not infer a conformance pass from an unavailable fixture.