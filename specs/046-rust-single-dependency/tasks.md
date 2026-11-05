# KMIPKIT-0046 Tasks

- [ ] T001 Add an external single-dependency consumer fixture and prove it fails on missing facade namespaces and client configuration paths (Red commit).
- [ ] T002 Expose complete `protocol` and `ttlv` namespaces plus the minimal client and transport configuration surface; extend the fixture to cover the 56 operation request types, representative response and asynchronous types, and typed execution (Green commit).
- [ ] T003 Add deterministic manifest reconciliation for every new facade export path, regenerate generated outputs with the pinned generator, and test stable output and exact exclusion behavior.
- [ ] T004 Document the one-dependency Rust usage, configuration path, generic TTLV decoder boundary, and low-level transport exclusion; record the accepted facade decision in ADR-0017.
- [ ] T005 Refactor namespace/config exports and fixture helpers without changing behavior; rerun the fixture and export/manifest checks (Refactor commit).
- [ ] T006 Run local formatting, Clippy, targeted tests, manifest generation check, and applicable workspace tests; record evidence and limitations in implementation-evidence.md.
- [ ] T007 Push the feature branch and open/update the draft PR against `feature/KMIPKIT-0043-t076-shared-client-mapping`.
