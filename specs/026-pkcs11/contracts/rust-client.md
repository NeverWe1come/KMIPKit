# Rust Client Contract: PKCS#11

Additive contract following existing request/response and explicit-client patterns. OASIS source: §6.1.37 Tables 273–275. Cross-language mapping is governed by KMIPKIT-0043.

## Types and method

- Types: Pkcs11Request, Pkcs11Response, Pkcs11Error.
- Request requires PKCS#11 Function; builders accept optional Interface, Correlation Value, and Input Parameters.
- Preserve Function's Enumeration-or-Integer choice. Keep interface, parameters, and return-code values lossless generic TTLV structures.
- Success exposes required Function/Return Code and optional Interface/Correlation Value/Output Parameters. Failure retains the shared result and fabricates no success fields.
- Admit ClientRequest::Pkcs11 and ClientOperation::Pkcs11; expose Client::pkcs11(request, limits), with an options-bearing method following existing convention.
- One explicit call uses one exchange. It does not retry, Poll, infer remote state, or parse the PKCS#11 Profile.
- Keep generic response TTLV accessible. Redact parameter/correlation contents in Debug, Display, and errors.

Use these Rust names as API-manifest inputs only after the approved 0043 mapping is integrated. Never hand-edit generated bindings.
