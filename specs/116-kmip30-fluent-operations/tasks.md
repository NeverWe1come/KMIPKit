# KMIPKIT-0116 Delivery Tasks

- [x] C001 Confirm the seven approved KMIP 3.0 assignments, pinned clauses,
  typed models, current execution routes, version gate, and four-language
  API gap.
- [x] C002 Record the parent-approved scope, field mapping, security limits,
  release-line placement, acceptance criteria, and explicit version behavior.
- [x] R001 Add failing consumer/API-shape tests for all seven operation
  builders in Rust, C, Java, and Python; record exact Red evidence.
- [x] G001 Implement Rust builders, typed conversion, and selected-version
  execution using existing request models.
- [ ] G002 Implement the opaque C ABI, direct common-task calls, builder
  lifecycle, results, and ownership tests.
- [ ] G003 Implement Java/JNI and Python/CFFI builders, typed response access,
  PEP 561 stubs, and prior-API compatibility tests.
- [ ] D001 Update manifest, generated exports, operation capability matrix,
  traceability, and bilingual guides/examples.
- [ ] V001 Run formatting, lint, focused and full suites, sanitizers, coverage,
  manifest generation/audit, T102, traceability, and exact-head CI.
- [ ] P001 Commit Red, Green, and Refactor separately and open a `[WIP]`
  stacked draft PR. Remove `[WIP]` only after all required exact-head CI is
  green; do not merge.
