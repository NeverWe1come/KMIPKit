# KMIPKIT-0055 tasks

- [x] T001 Verify the approved specification, ADR-0018, common Item/limits/error
  contract and immutable OASIS sources.
- [x] T002 Pin the exact official MSGENC-JSON-M-1-21 description and provenance.
- [x] T003 Record failing profile, mask, limits and official-replay tests in a
  signed RED development commit.
- [x] T004 Implement bounded private parsing/emission without public dispatch,
  public API manifest or generated API changes.
- [x] T005 Preserve exact ordered values and unknown raw tag/enum/mask bits;
  validate scalar forms and malformed input; use only generated symbolic names.
- [x] T006 Apply common wire-byte/count/depth limits before payload construction,
  redact errors, and zeroize temporary/output payload buffers.
- [x] T007 Replay four official semantic messages and compare the emitted JSON
  objects to the independent descriptor projection; compare the three literal
  normative JSON wire samples under profile-permitted representations; verify 100 generated
  semantic roundtrips.
- [x] T008 Refactor metadata indexing and writer responsibilities without changing
  wire semantics, and preserve the existing source audit unchanged.
- [x] T009 Run relevant client tests, formatting, Clippy and focused coverage;
  record exact evidence, traceability and integration boundaries.

Client/transport publication and four-language exposure are KMIPKIT-0057 work.
A complete profile claim and live server interoperability are separate gates.
