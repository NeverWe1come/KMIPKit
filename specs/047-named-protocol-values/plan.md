# KMIPKIT-0047 Implementation Plan

## Summary

Generate stable named constants for the checked-in KMIP 2.1 Enumeration and
Bit Mask catalog entries, with matching lookup helpers in all four supported
languages. Preserve the existing open raw numeric representations.

## Decisions

- The checked-in normative catalog is the only generation input.
- Names use the catalog value name normalized by the pinned generator's
  deterministic language-specific identifier rules. Raw numeric values are
  always exposed alongside names.
- Unknown Enumeration values have no known name and remain valid raw values.
- Bit-mask helpers return only recognized assigned bits; the original raw
  mask remains available unchanged so callers can inspect unknown bits.
- Rust constants live under a generated `kmipkit::protocol::values` namespace;
  C exports generated prefixed macros and lookup functions in the public
  header; Java and Python expose generated protocol-value namespaces.
- The generator validates exact catalog counts, duplicate identifiers,
  duplicate values within each value family, and numeric range before writing.
- Generated files are never edited by hand; one pinned generator updates all
  four language surfaces and manifest input artifacts deterministically.

## Normative References

Every value is traced to its exact catalog element and the OASIS KMIP 2.1
section recorded on that element. Enumeration and Bit Mask definitions are in
the pinned normative specification §§11–12. Project-specific unknown-value
preservation follows the approved catalog policy and Constitution §4.

## Validation

The generator contract checks counts and identities; language consumers check
representative values and all generated names. Negative cases cover unknown
Enumeration values, masks containing both known and unknown bits, invalid
catalog duplicates, and malformed raw values. `--check` must detect stale or
missing generated outputs without modifying them.
