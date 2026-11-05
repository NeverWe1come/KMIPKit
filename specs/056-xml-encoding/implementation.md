# KMIPKIT-0056 XML codec implementation

## Boundary and representation

The private `kmipkit-client::xml_codec` module implements the accepted
[0054 common contract](../054-multi-encoding-governance/contracts/common-wire-codec.md)
over the ordered `kmipkit_ttlv::Item` tree. Client selection, transport I/O,
public declarations, generated artifacts and API dispatch remain owned by
KMIPKIT-0057. [ADR-0018](../../docs/adr/0018-multi-encoding-1-0-scope.md)
governs the transport boundary and contextual HTTPS interpretation.

Known tags and assigned Enumeration/Bit Mask values use the checked-in
`WIRE_NAMES` metadata, normalized by Profiles v2.1 §5.4.1.1. Generic `TTLV`
elements with three-byte hexadecimal tags preserve allocation-valid unknown
and vendor tags. Unknown Enumeration values use eight-digit hexadecimal
strings. Mask output uses assigned names separated by spaces followed by an
eight-digit hexadecimal remainder. Reserved and extension placeholders are
never assigned symbolic names. Zero masks use `0x00000000`.

All eleven model types preserve their represented values, child order and
repeated tags. Big Integers preserve exact sign-extension octets and require
nonempty multiples of eight bytes. XML Byte Strings and Big Integers use
`xsd:hexBinary` without a prefix. DateTime uses an explicitly zoned Gregorian
`xsd:dateTime`; output normalizes to UTC without limiting the model's `i64`
seconds range. Fractional dates with a nonzero fraction cannot enter the
seconds-only model and are rejected rather than rounded. DateTimeExtended
remains an `xsd:long` value. Unknown symbolic names cannot be represented
without a raw identity and are rejected; their hexadecimal equivalents work.

## Security and ownership

The pinned `quick-xml =0.42.0` dependency has default features disabled. Its
borrowed UTF-8 reader supplies events; KMIPKit performs namespace, attribute,
type and scalar checks. No DOM, DTD processing, external entity resolver,
schema retrieval, filesystem access or network access exists in the codec.
The parser's end-name stack is disabled; the codec compares its own bounded,
zeroizing open-name owners. Comments are checked for malformed `--` content.

Wire-byte limits apply before UTF-8 validation and parsing. Item count and
Structure depth are checked before payload decoding/descent, including the
root. Namespace and scalar scratch are bounded by the already checked input
size, use fallible reservations and zeroizing strings, and are released on
both success and error. Decoded values move into existing zeroizing model
owners. The model's boxed payload and Structure insertion remain the existing
infallible allocation boundaries, as in the TTLV codec.

The encoder checks every append before growing output past the byte cap and
checks tree depth/count while traversing. Returned bytes are
`Zeroizing<Vec<u8>>`. Allocation growth has ADR-0012's existing limitations:
zeroizing the live allocation cannot establish clearing of earlier allocator
copies. Borrowed input belongs to the caller, whose lifecycle remains its
responsibility. Errors retain only a safe common category and no input,
parser error, decoded payload, source error, or XML context.

## Dependency review

The exact `quick-xml` 0.42.0 crate manifest and source were inspected locally:
MIT license, Edition 2024, MSRV 1.86 (below project MSRV 1.94), no build script,
and only the already present `memchr` dependency with default features
disabled. Optional encoding, asynchronous, serde and HTML entity features
are not enabled. The borrowed reader and raw attributes avoid the dependency's
allocating entity expansion helpers; predefined and numeric references are
decoded into explicit KMIPKit zeroizing scratch. DTD events fail immediately.
Dependency-policy scan results are recorded in the implementation report.

## Requirement traceability

| ID | OASIS source | Implementation / verification |
|---|---|---|
| KMIPKIT-0056-R001 | Spec v2.1 §10.2.3; Profiles v2.1 §§5.4.1.1–5.4.1.6.2 | Catalog normalization, general/specific XML forms, namespace scopes, default/explicit Structure and ordered repeated children. |
| KMIPKIT-0056-R002 | Profiles v2.1 §§5.4.1.2, 5.4.1.6.3, 5.4.1.6.5–5.4.1.6.13 | Independent scalar literal cases, malformed/range tests, full-range DateTime property tests and raw scalar/tree properties. |
| KMIPKIT-0056-R003 | Profiles v2.1 §5.4.1.6.4 | All three assigned M/B families, all 33 flags, space separator, numeric/hex forms, zero/reserved/unknown bit roundtrips. |
| KMIPKIT-0056-R004 | 0054 common contract; constitution IV | Exact byte/depth/count boundaries for encode/decode, deeply nested adversarial input and depth preflight before payload attribute decoding. |
| KMIPKIT-0056-R005 | 0054 common contract; constitution IV | Malformed XML, DTD, external entities, namespace/prolog/reference validation, malformed scalars and payload-free errors. |
| KMIPKIT-0056-R006 | Profiles v2.1 §§5.4.2, 5.4.4.1 | All four pinned `MSGENC-XML-M-1-21` message documents, deterministic `$NOW`, two response-size requests and failure/success statuses, semantic replay. |

All verification tests are in
`crates/kmipkit-client/tests/unit/xml_codec_tests.rs`; official fixture origin
and exact SHA-256 are in `crates/kmipkit-client/tests/fixtures/xml/README.md`.
This work proves codec behavior and fixture replay. It does not claim the
complete XML Client profile, real-server interoperability, or certification.
