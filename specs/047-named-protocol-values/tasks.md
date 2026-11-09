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
  security checks for the current revision; record exact commands and outcomes
  in `implementation-evidence.md`. The prior hosted closeout applies to the
  earlier head only. Focused checks for this correction pass, but the current
  manifest audit and remote workflow matrix still require closeout.
- [x] T009 Reconcile the approved count discrepancy: the catalog rows include
  reserved and wildcard extension entries, which do not have assigned
  numeric values. Confirm that generated usable constants cover only entries
  with `allocation=assigned` and concrete 32-bit values.
- [x] T010 Add a checksum-verified version-delta catalog for KMIP 2.1/3.0 and
  Red tests for exact operation, Object Class, and Object Type assignments
  from KMIP 2.1 §11.36 Table 470 and KMIP 3.0 §§11.34–11.35 Tables 581–582 and
  §11.37 Table 583.
- [x] T011 Generate versioned named values and lookups in Rust, C, Java, and
  Python; correct KMIP 3.0 `ObjectClass`, `ObjectType`, and explicit
  `VersionGate` operation spans without treating reserved codes as assigned.
- [x] T012 Reconcile the versioned public declarations, traceability,
  documentation, and corrected KMIPKIT-0071 references; run generator and API
  manifest checks.
- [x] T013 Synchronize shared T102 aggregate snapshot guards with the 143
  versioned-value declarations added by this specification; preserve the
  historical Rust audit snapshot and rerun dependent contract tests.
- [x] T008 Push this feature branch and open a draft PR against
  `feature/KMIPKIT-0043-t076-shared-client-mapping`; later change its base to
  `release/1.0.0` as described by the approved Phase 5 plan.
