# Requirements Checklist

- [ ] CHK001 Every helper maps only to its existing typed operation and cited KMIP 2.1 section.
- [ ] CHK002 Inputs remain typed; no arbitrary message or transport injection is available.
- [ ] CHK003 Timeout options remain separate from Request Header fields; no retry or Poll path is added.
- [ ] CHK004 Each C symbol is compiled and called from a C consumer, including invalid-input cases.
- [ ] CHK005 Fixture tests decode each request and return an encoded typed response; failures preserve delivery state and redaction.
- [ ] CHK006 Public API manifest, generated output, coverage, traceability, format, Clippy, and applicable CI checks pass on the final SHA.

