# Tasks: KMIPKIT-0125 Pre-Release Hardening

- [x] T001: Author specification, plan, tasks, and traceability.
- [ ] T002: Add negative requirement coverage contract test (Red) and verify KMIPKIT-REQ-PROF-5.12.3-003 with negative tests & decision record (Green).
- [ ] T003: Configure Miri strict-provenance verification on `kmipkit-ttlv` and `kmipkit-protocol` with bounded limits annotations.
- [ ] T004: Implement extended hostile negative tests for XML, JSON, and wire framing.
- [ ] T005: Add opt-in `fuzzing` feature with `#[doc(hidden)]` hooks in `kmipkit-client` and `kmipkit-transport`.
- [ ] T006: Add fuzz targets (`json_codec`, `wire_and_xml_preflight`, `s2c_and_multi_version`) with seed corpora and update fuzz configuration.
- [ ] T007: Expand CI workflow (`ffi-sanitizer-c`, `ffi-sanitizer-jni`, `fuzz-smoke`, and `miri`).
- [ ] T008: Add hardening campaign script and evidence documentation.
