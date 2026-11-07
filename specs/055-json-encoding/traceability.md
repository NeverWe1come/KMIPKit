# KMIPKIT-0055 traceability

Sources are the pinned OASIS KMIP Specification v2.1 §10.2.2 and KMIP
Profiles v2.1 §5.5. Exact source copies remain immutable. The implementation
is private `kmipkit-client::json_encoding` over the common Item contract.

| Requirement | Normative source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0055-REQ-001 | Profiles §5.5.1.1 normalized names | `names::normalize`, generated `WIRE_NAMES` | symbolic tags/enums, three mask families, mandatory fixture |
| KMIPKIT-0055-REQ-002 | Profiles §§5.5.1.2–5.5.1.3, 5.5.1.6.1 hex and tags | fixed-width hex helpers, checked RawTag | malformed scalars, unknown extension raw tag, semantic property |
| KMIPKIT-0055-REQ-003 | Profiles §§5.5.1.4–5.5.1.6.2 default Structure and ordered JSON objects | `Scanner`, `read_item`, `Writer::item` | default/null/order/repeated children test |
| KMIPKIT-0055-REQ-004 | Profiles §5.5.1.6.3 Integer | `integer_value` | decimal and full-width signed hexadecimal cases |
| KMIPKIT-0055-REQ-005 | Profiles §5.5.1.6.4 masks | generated M/B families, `integer_value`, `Writer::mask` | symbolic/hex mixed components, `|`, all three families, unknown bits |
| KMIPKIT-0055-REQ-006 | Profiles §5.5.1.6.5 Long Integer | safe-number guard and exact 64-bit hex | precision boundaries and semantic property |
| KMIPKIT-0055-REQ-007 | Profiles §5.5.1.6.6 Big Integer | exact octet hex, multiple-of-eight validation | numeric zero, signed 8/16-byte values, invalid widths |
| KMIPKIT-0055-REQ-008 | Profiles §5.5.1.6.7 Enumeration | generated F/E resolution and hex fallback | assigned name, unknown 32-bit value, mandatory fixture |
| KMIPKIT-0055-REQ-009 | Profiles §5.5.1.6.8 Boolean | boolean/64-bit hex validation | both booleans, both hex forms, invalid hex/numeric values |
| KMIPKIT-0055-REQ-010 | Profiles §5.5.1.6.9 Text String | bounded string unescape and escaped output | Unicode, surrogate pairs, controls, exact property, malformed lexemes |
| KMIPKIT-0055-REQ-011 | Profiles §5.5.1.6.10 Byte String | unprefixed even-length hex | empty/random bytes, leading zeros, malformed hex |
| KMIPKIT-0055-REQ-012 | Profiles §5.5.1.6.11 Date-Time | zoned Gregorian/XSD parser, 64-bit hex | epoch/negative/BCE/leap/fraction/timezone/24:00 cases |
| KMIPKIT-0055-REQ-013 | Profiles §5.5.1.6.12 Interval | unsigned 32-bit decimal/hex | maximum, hex, negative/overflow rejection |
| KMIPKIT-0055-REQ-014 | Profiles §5.5.1.6.13 Date Time Extended | safe-number guard and exact 64-bit hex | precision threshold and semantic property |
| KMIPKIT-0055-REQ-015 | Profiles §5.5.4.1 MSGENC-JSON-M-1-21 | private codec; pinned fixture | three literal normative JSON wire examples; four descriptor messages with $NOW binding |
| KMIPKIT-0055-SEC-001 | Approved 0054 common contract | bounded scanner before tree construction; bounded recursive descent | wire-byte, count, Structure depth, malformed object nesting limits |
| KMIPKIT-0055-SEC-002 | Constitution IV; approved 0054 contract | `WireCodecError`, zeroizing temporary/output owners | redacted malformed cases; direct ownership review |

Profiles §5.5.2 also requires Baseline Client support and complete client
integration. Those obligations are outside this private codec and remain
tracked by the parent integration specification and programme. Passing the
codec replay does not establish a complete profile or certification claim.
Unknown names without a numeric tag/family identity cannot enter the Item
model; allocation-valid unknown tags and unknown raw enum/mask values use hex.
The optional JSON `name` is validated as a string hint and is not retained in
Item, which represents tag identity and semantic value only.
