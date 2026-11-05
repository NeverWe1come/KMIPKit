# KMIPKIT-0049 Normative Acceptance Checklist

This reviewer checklist follows the pinned KMIP 2.1 catalog and must be
closed with code and test evidence. It is separate from the requirement
quality checklist.

## Managed object structures

- [ ] Certificate §2.1/Table 2 fields, encodings, and order are implemented and tested.
- [ ] Certificate Request §2.2/Table 3 fields, encodings, and order are implemented and tested.
- [ ] Opaque Object §2.3/Table 4 fields and context separation are implemented and tested.
- [ ] PGP Key §2.4/Table 5 retains version and opaque public-only/public-private Key Blocks.
- [ ] Private Key §2.5/Table 6 and Public Key §2.6/Table 7 wrap typed Key Blocks.
- [ ] Secret Data §2.7/Table 8 handles wrapped and plaintext Key Values.
- [ ] Split Key §2.8/Table 9 handles identifier bounds and conditional Prime Field Size.
- [ ] Symmetric Key §2.9/Table 10 wraps typed Key Block.

## Key structures and secrets

- [ ] Key Block §3.1/Table 12 optionality, pairing, and wrapping rules pass positive and negative tests.
- [ ] Key Value §3.2/Table 13 supports required material and optional Attributes.
- [ ] Key Wrapping Data and key-information structures §§3.3/Tables 14–17 preserve required and optional fields.
- [ ] All nine transparent material formats §§3.4–3.12/Tables 18–26 have source-backed typed forms.
- [ ] Unknown/future enum values and vendor Items round-trip without loss.
- [ ] Debug and errors redact material; owned payloads zeroize; borrowed views obey owner lifetime.
- [ ] Register/Import typed helpers preserve existing generic APIs and operation behavior.
- [ ] Every operative traceability row links to actual implementation and test names.
