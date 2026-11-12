# KMIPKIT-0108 Traceability

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | KMIP v2.1 §6.1.20, Tables 223–224; KMIPKIT-0016-FR-006 | `fluent_attribute_read_tests`: ordered references, duplicate rejection, present-empty Attributes, and Pending correlation |
| FR-002 | KMIP v2.1 §6.1.21, Tables 226–227 | `fluent_attribute_read_tests`: optional identifier omission in the typed batch request and ordered response reference |
| FR-003 | KMIPKIT-0093-FR-004–005; ADR-0012 | `fluent_attribute_read_tests`: local validation, typed request variants, zero transport calls on invalid input |
| FR-004 | KMIP v2.1 §§8–9; KMIPKIT-0093-FR-005, FR-008; KMIPKIT-0007 | `fluent_attribute_read_tests`: one-exchange fake transport, typed success/Pending outcomes, correlation and delivery state |
| FR-005 | KMIPKIT-0093-FR-011; repository public API manifest policy | Rust docs, bilingual guides, generated manifest/parity checks |

## Red/Green/Refactor Evidence

- Red commit `aa3402158`: added tests and confirmed they failed because the
  fluent constructors were missing.
- Green: `CARGO_TARGET_DIR=target/kmipkit-0108 cargo test -p kmipkit-client
  fluent_attribute_read_tests --lib` passed four tests after the builder
  implementation.
- Refactor and remaining gates are recorded before the PR is marked ready.
