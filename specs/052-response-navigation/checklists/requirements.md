# KMIPKIT-0052 Requirements Checklist

- [ ] All 56 supported client-initiated operation results have an exact pinned OASIS response mapping or a reviewed explicit exception.
- [ ] The four asynchronous control/result response models are reachable through typed `ClientOperationOutcome` readers.
- [ ] Every high-level response item has one consistent raw Response Payload accessor that returns absence when no payload exists.
- [ ] Response Header raw access is directly available as an ordered borrowed TTLV Structure.
- [ ] Rust response views borrow from the response owner and cannot escape its lifetime.
- [ ] Nested structures and arbitrary-depth `0x54xxxx` tags are reachable without flattening, sorting, or dropping repeats.
- [ ] Raw inspection preserves unknown allocated tags, future values, type, order, and repeated items.
- [ ] Debug/error/toString/repr output omits response payload values, credentials, and key material.
- [ ] Rust-owned response-tree storage and FFI copies zeroize when their final owner is released.
- [ ] C raw-tree handles use fixed-width ABI, stable symbols, explicit release, and panic containment.
- [ ] Java and Python wrappers expose explicit close semantics and are covered by real consumers.
- [ ] English and Spanish docs explain absence semantics, lifetimes, no byte-exact replay, and external-copy limitations.
- [ ] All normative requirements have stable IDs and trace to code and executable tests.
- [ ] Every new public route is reconciled in the API manifest using the official generator; no generated file is edited by hand.
- [ ] Red, Green, and Refactor evidence is recorded in distinct commits for each behavior slice.
- [ ] Required local tests, formatting, lint, and manifest checks pass; coverage and remote CI status are reported honestly.
