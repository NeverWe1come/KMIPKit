# Fuzz targets

KMIPKit keeps bounded coverage-guided fuzz targets in this workspace.

## TTLV and typed-message parsing

`ttlv_decode` decodes at most 4 KiB of input with limits of 64 nested
Structures and 512 Items. Successful generic decoding is followed by attempts
to validate the same tree as a Request Message and a Response Message. Valid
Encrypt/Decrypt request items have their payload fields traversed; matching
response items are sent through the completed or Pending typed converters.
Malformed framing, message shapes, and typed response payloads are expected
rejection paths. Request payload schema rejection is covered by bounded
property tests at the protocol validation boundary. The target does not format
or log decoded payloads.

The checked-in `verification/security/fuzz/corpus/ttlv_decode/` seeds are synthetic bounded TTLV
messages, not OASIS conformance fixtures. They cover Encrypt and Decrypt
requests, successful responses, Pending responses, and unknown Result Reason
responses so the smoke campaign reaches both shared message validation and
typed operation conversion paths.

Run a bounded smoke campaign with the installed nightly toolchain and
`cargo-fuzz`:

```text
cargo +nightly fuzz run ttlv_decode -- -runs=1000 -max_len=4096 -timeout=5
```

Minimized crashes are written to `verification/security/fuzz/artifacts/ttlv_decode/`. Promote each
reproducible crash to a deterministic regression test before making a fix.

## Extension schema validation

`extension_schema` decodes at most 4 KiB of untrusted TTLV using limits of 64
nested Structures and 512 Items. If the root is a Structure, the target runs
it through a fixed nested data-only schema with discriminator, scalar range,
enumeration, repeated-field, and order constraints. Schema mismatches are
expected; panics and limit bypasses are not.

Run with the installed nightly toolchain and `cargo-fuzz`:

```text
cargo +nightly fuzz run extension_schema -- -max_len=4096 -timeout=5
```

Use `-runs=1000` for a short smoke campaign. Minimized crashes are written to
`verification/security/fuzz/artifacts/extension_schema/`. Promote each reproducible crash to a
deterministic protocol regression test before making a fix.

The executable OASIS-derived extension vectors cite KMIP Specification v2.1
§§7.13/Table 365, 8.3/Table 396, 9.13/Table 418, 11.44/Table 476, and §11.56
tag allocations in `bindings/shared/fixtures/extensions/oasis/` and
`crates/kmipkit-protocol/tests/extension_oasis_vectors.rs`.

## JSON codec and roundtrip

`json_codec` decodes at most 4 KiB of untrusted JSON using bounded limits.
Decoded items undergo roundtrip serialization and verification.

```text
cargo +nightly fuzz run json_codec -- -runs=1000 -max_len=4096 -timeout=5
```

## Transport wire framers and XML preflight

`wire_and_xml_preflight` feeds incremental byte sequences to the allocation-free
JSON and XML document boundary framers, and exercises XML preflight and decoding.

```text
cargo +nightly fuzz run wire_and_xml_preflight -- -runs=1000 -max_len=4096 -timeout=5
```

## Server-to-Client and multi-version negotiation

`s2c_and_multi_version` fuzzes incoming Server-to-Client batch ingestion across TTLV,
JSON, and XML encodings, as well as ProtocolVersion multi-version negotiation structures.

```text
cargo +nightly fuzz run s2c_and_multi_version -- -runs=1000 -max_len=4096 -timeout=5
```
