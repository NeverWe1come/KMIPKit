# KMIPKIT-0102 Delivery Tasks

- [x] C001 Confirm exact KMIP 2.1 sections, typed request models, current
  validators, and client routes for the seven in-scope operations.
- [x] C002 Record the approved scope, explicit cryptographic choices, secret
  ownership, exclusions, and acceptance criteria.
- [x] R001 Add failing tests for all seven constructors and operation-specific
  request fields; record the exact Red command and result.
- [x] G001 Implement explicit Encrypt, Decrypt, Sign, Signature Verify, MAC,
  MAC Verify, and Hash builders using the typed request models.
- [x] G002 Verify local rejection, version gates, secret redaction, batch
  conversion, one-exchange execution, and all three outcomes.
- [x] D001 Update manifest, Rust documentation, traceability, examples, and
  generated artifacts for exactly the new public exports.
- [ ] V001 Run format, Clippy, focused and full client suites, coverage,
  manifest generation/audit, traceability, and documentation checks.
- [x] P001 Commit Red, Green, and Refactor separately and open a `[WIP]` draft
  against PR #184; keep WIP until exact-head CI is green.
