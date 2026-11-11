# KMIPKIT-0101 Delivery Tasks

- [x] C001 Verify each operation's pinned source, direction, typed request and
  response model, version gate, and current client route.
- [x] R001 Add a failing compile/runtime test for all eight fluent constructors
  and record the exact expected failure.
- [ ] G001 Implement operation-specific typed builders and semantic setters;
  preserve existing validation and one-exchange execution.
- [ ] G002 Test typed batch conversion, Success/Pending/Failure, correlation,
  delivery state, local rejection, version gates, and redaction.
- [ ] D001 Update public API manifest, traceability, examples or API guide,
  generated artifacts, and implementation evidence.
- [ ] V001 Run format, Clippy, focused and full client suites, coverage,
  manifest generation, export audit, and traceability checks.
- [ ] P001 Commit Red, Green, and Refactor separately, open a `[WIP]` draft
  based on #183, and keep WIP until exact-head CI is green.
