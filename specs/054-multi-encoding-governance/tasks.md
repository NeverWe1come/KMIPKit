# KMIPKIT-0054 Tasks

## RED

- [x] Add failing tests for private tag and Enumeration name rendering,
  catalog-derived tag-to-Enumeration family resolution, ambiguity rejection,
  stable ordering, and generator CLI write/check behavior.
- [x] Run the focused tests before implementation and capture the expected
  failure because the wire-name generator and Enumeration metadata are absent.

## GREEN

- [x] Implement the offline catalog generator and produce the private
  `wire_names_generated.rs` table.
- [x] Add the generator check and 95% line-coverage gate to CI; verify the
  checked-in output matches the pinned local catalog.
- [x] Run the focused catalog generator suite and compile the client crate.

## REFACTOR

- [x] Align the constitution, `AGENTS.md`, project definition, roadmap,
  architecture docs, ADR index, and supersession record with the approved
  three-encoding 1.0 scope.
- [x] Record the contextual HTTPS interpretation and distinguish local wire
  byte caps from OASIS `Maximum Response Size`.
- [x] Define the shared private `Item`/name-resolution codec contract and
  prepare draft 0055, 0056, and 0057 specifications with the correct OASIS
  section ownership and public API edit boundary.
- [x] Format the generated Rust table with `rustfmt` and gate that check in CI.
- [x] Review for stale TTLV-only 1.0 statements, public manifest changes,
  generated public output, and whitespace errors.
