# KMIPKIT-0103 Delivery Tasks

- [x] C001 Confirm the KMIP 2.1 request shapes and Rust KMIPKIT-0102 builders
  for all seven cryptographic operations.
- [x] C002 Record the approved C ABI scope, ownership model, result handling,
  exclusions, and acceptance criteria.
- [x] R001 Add failing compiled C-consumer tests for all seven constructors,
  semantic fields, build/execute behavior, and invalid inputs; capture Red.
- [x] G001 Implement opaque operation builders and fixed-width exported C ABI
  constructors, setters, build, execute, options, and release functions.
- [x] G002 Prove all seven operation routes over a controlled TLS fixture,
  including typed outcomes, version checks, and single-exchange behavior.
- [x] D001 Update public API manifest, generated header/registries, bilingual C guide,
  traceability, and implementation evidence.
- [ ] V001 Run C ABI and consumer suites, full relevant Rust suites, formatting,
  Clippy, coverage, API export audit, manifest generation, and traceability.
- [ ] P001 Commit Red, Green, and Refactor separately and open a `[WIP]` draft
  PR stacked on KMIPKIT-0102; retain WIP until exact-head CI is green.
