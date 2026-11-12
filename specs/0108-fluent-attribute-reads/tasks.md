# KMIPKIT-0108 Delivery Tasks

- [x] T001 Add failing controlled tests for both builders, ordered references,
  omitted identifiers, duplicate-reference validation, and one-exchange
  execution. Red: commit `aa3402158`; `cargo test -p kmipkit-client
  fluent_attribute_read_tests --lib` failed because the two `RequestBuilder`
  constructors did not exist.
- [x] T002 Implement typed request, batch conversion, options, asynchronous
  indicator, and execution for Get Attributes and Get Attribute List. Green:
  four focused tests pass through the existing typed exchange path.
- [ ] T003 Document both APIs in English and Spanish; update traceability and
  generated API manifest/parity artifacts.
- [ ] T004 Run local gates, commit Red/Green/Refactor separately, open a `[WIP]`
  PR, and resolve every red CI check before marking ready.
