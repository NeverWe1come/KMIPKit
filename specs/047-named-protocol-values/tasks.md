# KMIPKIT-0047 Tasks

- [x] T001 Add catalog contract tests that account for all 64 Enumeration
  types, 723 Enumeration value rows, 3 Bit Masks, and 44 Bit Mask value rows,
  while asserting named constants only for the 644 assigned Enumeration
  values and 33 assigned bits (Red commit).
- [x] T002 Add cross-language API contract tests proving named values map to
  exact raw values, unknown Enumeration values remain representable, and mask
  lookups preserve unknown bits (Red commit).
- [x] T003 Implement the deterministic generator and Rust, C, Java, and Python
  named-value surfaces; regenerate checked-in outputs (Green commit).
- [x] T004 Reconcile every new public declaration with the approved API
  manifest and add generated-output and exact-count checks.
- [x] T005 Document usage and clarify that named values do not assert server
  support, profile conformance, or cryptographic policy.
- [x] T006 Refactor generator and language surfaces without changing their
  public behavior; rerun focused checks and generator `--check` (Refactor
  commit).
- [ ] T007 Run applicable format, lint, unit, binding, manifest, coverage, and
  security checks; record exact commands and outcomes in
  `implementation-evidence.md`. The recorded local Windows format, workspace
  tests/Clippy, Rust coverage, four-language consumers, manifest
  generators/audits, and scoped security scan pass. Keep this task open until
  the required cross-platform aggregate coverage and hosted platform controls
  are available and their results are recorded.
- [x] T009 Reconcile the approved count discrepancy: the catalog rows include
  reserved and wildcard extension entries, which do not have assigned
  numeric values. Confirm that generated usable constants cover only entries
  with `allocation=assigned` and concrete 32-bit values.
- [x] T008 Push this feature branch and open a draft PR against
  `feature/KMIPKIT-0043-t076-shared-client-mapping`; later change its base to
  `release/1.0.0` as described by the approved Phase 5 plan.
