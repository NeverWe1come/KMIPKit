# Tasks: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

- [ ] **TASK-071-001**: Ingest OASIS KMIP 3.0 draft specifications and record spec documentation.
- [ ] **TASK-071-002**: Add Red unit tests for `LinkNormalizer`, System Objects, `ObjectClass`, and KEM operations in `kmipkit-protocol`.
- [ ] **TASK-071-003**: Implement Green `LinkNormalizer`, System Objects, `ObjectClass`, and KMIP 3.0 operations in `kmipkit-protocol`.
- [ ] **TASK-071-004**: Add Red tests for `kmipkit-test-server` KMIP 3.0 mode and operation dispatch.
- [ ] **TASK-071-005**: Implement Green KMIP 3.0 mode in `kmipkit-test-server`.
- [ ] **TASK-071-006**: Add end-to-end client tests for KMIP 3.0 version selection and operations.
- [ ] **TASK-071-007**: Refactor, run formatting, clippy, test suite, and catalog verification.
- [ ] **TASK-071-008**: Correct LinkNormalizer to use the exact 14 shared KMIP 2.1/3.0 semantic mappings and the 20 pinned KMIP 3.0 link tag assignments; reject lossy conversions. Implemented and verified in the KMIPKIT-0048 correction PR.
- [x] **TASK-071-009**: Catalog and generate all 60 exact KMIP 3.0 §11.58, Table 605 tag assignments, including credential and KEM tags; preserve version gating as the authority for edition-specific acceptance. Implemented by KMIPKIT-0048 Green commit `0fd8ea58e`.
