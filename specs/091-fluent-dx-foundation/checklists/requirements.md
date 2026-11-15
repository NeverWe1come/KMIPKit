# Requirements Checklist - KMIPKIT-0091

- [x] Delivery 1 is isolated from later Fluent DX operation families.
- [x] Common key properties and policy remain explicit.
- [x] Unknown attributes require explicit TTLV types; Vendor Attribute uses its own structure.
- [x] Result and pending behavior reuse existing typed response contracts.
- [x] C helpers and builder preserve stable ABI ownership/error conventions.
- [x] Existing Java, Python, C, and Rust APIs remain supported.
- [ ] Maintainer approves this specification before production implementation.
- [ ] Red/Green/Refactor evidence exists for each public behavior.
- [ ] All four package consumers pass the loopback TLS 1.3/mTLS journey.
- [ ] API manifest, generated outputs, export audit, documentation, and traceability agree.
- [ ] Required workspace, binding, lint, coverage, and security gates pass.
