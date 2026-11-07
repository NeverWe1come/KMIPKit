# KMIPKIT-0049 Normative Acceptance Checklist

This reviewer checklist follows the pinned KMIP 2.1 catalog and must be
closed with code and test evidence. It is separate from the requirement
quality checklist.

## Managed object structures

- [x] Certificate §2.1/Table 2 fields, encodings, and order are implemented and tested.
- [x] Certificate Request §2.2/Table 3 fields, encodings, and order are implemented and tested.
- [x] Opaque Object §2.3/Table 4 fields and context separation are implemented and tested.
- [x] PGP Key §2.4/Table 5 retains version and opaque public-only/public-private Key Blocks.
- [x] Private Key §2.5/Table 6 and Public Key §2.6/Table 7 wrap typed Key Blocks.
- [x] Secret Data §2.7/Table 8 handles wrapped and plaintext Key Values.
- [x] Split Key §2.8/Table 9 handles identifier bounds and conditional Prime Field Size.
- [x] Symmetric Key §2.9/Table 10 wraps typed Key Block.

## Key structures and secrets

- [x] Key Block §3.1/Table 12 optionality, pairing, and wrapping rules pass positive and negative tests.
- [x] Key Value §3.2/Table 13 supports required material and optional Attributes.
- [x] Key Wrapping Data and key-information structures §§3.3/Tables 14–17 preserve required and optional fields.
- [x] All nine transparent material formats §§3.4–3.12/Tables 18–26 have source-backed typed forms.
- [x] Unknown/future enum values and vendor Items round-trip without loss.
- [x] Debug and errors redact material; owned payloads zeroize; borrowed views obey owner lifetime.
- [x] Register/Import typed helpers preserve existing generic APIs and operation behavior.
- [x] Every operative traceability row links to actual implementation and test names.

Evidence: `traceability.md` maps all 53 operative requirements to implementation
and executable tests. Exact local commands, test results, coverage measurements,
security review, and remaining hosted gates are recorded in
`implementation-evidence.md`.
