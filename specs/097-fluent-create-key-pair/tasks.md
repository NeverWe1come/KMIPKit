# KMIPKIT-0097 Delivery Tasks

- [x] T001 Add failing builder tests for explicit algorithm/length, attribute
  group placement, invalid Table 191 pairs, and `NotSent` behavior.
- [x] T002 Add the minimal typed `CreateKeyPairBuilder` implementation and
  `RequestBuilder::create_key_pair()`; keep Red and Green commits separate.
- [x] T003 Add batch conversion, one-exchange execution, and outcome/correlation
  tests through the existing client transport path.
- [x] T004 Document and export the Rust surface in English, Spanish, and the
  generated API manifest; update traceability and repository audit inputs.
- [ ] T005 Run required local verification, open a `[WIP]` draft PR, repair all
  red CI checks, and mark ready only after the exact PR head is fully green.
