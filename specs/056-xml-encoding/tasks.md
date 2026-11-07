# KMIPKIT-0056 implementation evidence

- [x] Confirm approved scope, common Item contract, normative sources and
  ADR-0018 before implementation.
- [x] Record failing codec behavior tests and pin the official XML vector.
- [x] Implement private bounded XML parse/emit over all eleven Item types.
- [x] Resolve only assigned catalog tag, Enumeration and Bit Mask names;
  preserve allocation-valid unknown identities and raw bits through hex forms.
- [x] Support general/specific elements, namespace scopes, default Structure,
  explicit types, ordered children and repeated tags.
- [x] Enforce byte/count/depth bounds, including preflight before Structure
  payload attribute decoding, and return payload-free common errors.
- [x] Reject DTD/external entities, malformed XML/declarations/namespaces,
  invalid reference grammar, scalar range errors and lossy scalar conversions.
- [x] Replay all four official messages with deterministic timestamp binding;
  verify the two maximum sizes and failure/success response statuses.
- [x] Record separate signed Red, Green and Refactor commits.
- [x] Pass focused tests, complete client regression, fmt, Clippy, changed-code
  coverage and fresh root/fuzz dependency-policy checks.
- [x] Self-review scope and committed fixture bytes; preserve immutable OASIS
  upstream, public API manifest and dispatch boundaries.
- [x] Address independent review findings: complete allocation-free resource
  preflight before XML reader construction and XML 1.0 QName/NCName validation;
  record signed RED/GREEN/REFACTOR commits and fresh verification evidence.
- [x] Record requirement traceability and allocation/ownership limitations in
  [implementation.md](implementation.md) and the full execution evidence in
  [the worktree report](../../.superpowers/sdd/0056-xml/report.md).

KMIPKIT-0057 owns runtime selection and four-language exposure. Whole-release
cross-platform checks and actual two-server interoperability remain release
gates; this codec evidence does not claim complete XML Client conformance.
