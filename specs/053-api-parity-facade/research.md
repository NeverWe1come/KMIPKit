# KMIPKIT-0053 Research and Decisions

1. Rust remains the sole protocol/model/validation behavior source; adapters call its implementation.
2. Language APIs may use idiomatic naming only when exact manifest mappings and runnable tests prove equivalent data, wire, errors, delivery, redaction and lifetime.
3. Generic TTLV remains available for unknown tags, future values, repeated fields and vendor extensions.
4. C uses fixed-width values and opaque handles; Java uses AutoCloseable; Python uses close/context managers. KMIPKit-owned secret copies zeroize at last release.
5. Authentication remains one caller-owned value on one Request Message, never a client-global default.
6. Foreign validation exposes the shared Rust report; it does not duplicate rules or guess generic schemas.
7. Response semantics are inherited from KMIPKIT-0052, including absent-vs-empty and decoded-tree behavior.
8. Repetitive routes use deterministic reconciliation and the official generator; generated output is never edited by hand.
9. No protocol boundary or operation scope changes. The pinned OASIS Standard resolves normative questions.
