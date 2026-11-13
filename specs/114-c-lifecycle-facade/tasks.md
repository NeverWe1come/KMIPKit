# KMIPKIT-0114 Delivery Tasks

- [x] T001 [RED COMMIT] Add a compiled C consumer that calls all five
  operations and their timeout variants and checks invalid-input behavior.
- [x] T002 Record the expected missing-symbol or behavior failures and commit
  Red evidence before production changes. `cargo test -p kmipkit-ffi
  --features coverage-c-consumer --test c_api_coverage
  public_c_client_surface_probe -- --exact` failed while compiling the C
  consumer because the ten lifecycle helpers are not declared in the public
  header yet. This is the intended Red result; no production implementation
  was present in the working tree.
- [x] T003 [GREEN COMMIT] Implement the ten C ABI entry points by constructing
  the approved typed requests and using the existing client execution path.
- [x] T004 [REFACTOR COMMIT] Consolidate shared pointer, options, response,
  and error handling; add manifest declarations and regenerate all outputs.
- [x] T005 Add controlled TLS fixture tests for decoded request semantics and
  encoded typed responses for all five operations, including a no-send case.
- [ ] T006 Run lifecycle C ABI, compiled consumer, sanitizer, traceability,
  manifest/export, coverage, format, Clippy, and relevant workspace checks;
  resolve every exact-head CI failure.

