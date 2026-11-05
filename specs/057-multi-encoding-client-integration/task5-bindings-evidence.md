# KMIPKIT-0057 Task 5 evidence — four-language selection API

Task 5 is complete. The public API manifest is the source for the C header,
Rust FFI metadata, Java and Python exports, and the checked-in parity fixtures.
The public C header now has a complete construction and execution path rather
than encoding-selection declarations detached from a usable client.

## Red, Green, Refactor commits

The failing API and binding tests were added in separate signed commits:

- `d81b3a9d1` — specify the four-language encoding API.
- `86f82c9a6` — specify default and override behavior in the bindings.
- `221c0e4eb` — require generated C client records.
- `13044fb36` — require dependency-safe generated C record order.
- `4fe7c161d` — exercise the Java encoding override over TLS.
- `df6fae119` — require a usable public C header client path.
- `b6a35a17d` — require 0057 Rust audit paths to be accounted for.

Green implementation is in signed commit `120a4ab76`. The separate Refactor
commit records final verification and marks only Task 5 complete.

## API and compatibility

- Rust exposes `WireEncoding::{Ttlv, Json, Xml}` and immutable client
  configuration copies. `RequestOptions` supplies an optional per-exchange
  override; the configured default remains TTLV. Selection is override, then
  configuration, then TTLV.
- C exposes `kmipkit_wire_encoding_t` and the TTLV/JSON/XML constants. New
  versioned transport records let C construct a transport and client, create a
  batch, execute it, and read ordered result metadata through the generated
  public header. `kmipkit_client_execute_with_options` accepts `-1` for the
  configured default and `0`, `1`, or `2` for TTLV, JSON, or XML.
- Java exposes `org.kmipkit.extensions.WireEncoding`, immutable
  `ClientConfiguration.withWireEncoding(...)`, the configuration getter, and
  `Client.executeWithOptions(..., @Nullable WireEncoding)`.
- Python exports `kmipkit.extensions.WireEncoding`, immutable
  `ClientConfiguration.with_wire_encoding(...)`, and
  `Client.execute(..., wire_encoding=None)`.
- Unknown C/JNI encoding values and invalid Python values map to the established
  invalid-input error. The C `-1` sentinel is accepted only for the optional
  exchange override. Existing execute entry points continue to use the
  configuration default; existing ABI version and symbols are unchanged.

## Verification

| Check | Result |
|---|---|
| `python -B tools/api_manifest/generate.py --check` | Passed; all 12 generated outputs current. |
| Four-file manifest/API pytest selection (`test_manifest_v2.py`, `test_t030_package_docs.py`, `test_generate.py`, `test_wire_encoding_public_api.py`) | 93 passed, 2 skipped. |
| Exact checked-in Rust inventory test | 1 passed. |
| CI extension fixture command: `python -B -m unittest discover --failfast -s tools/api_manifest/tests -p 'test_extension_fixtures.py' -v` | 17 passed, 1 Windows symlink-permission skip. |
| `mvn -q -f bindings/java/pom.xml test` | Passed, including JNI native build and Java TLS encoding override test. |
| `python -m pytest -q bindings/python/tests` | 162 passed. |
| `cargo test -p kmipkit-client --all-features wire_encoding_tests` | 7 passed. |
| `cargo test -p kmipkit-ffi --all-features` | 54 Rust unit tests and 3 C ABI consumer tests passed. The public C consumer constructs a client/batch and calls the encoding override entry point. |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy -p kmipkit -p kmipkit-ffi -p kmipkit-client -p kmipkit-transport --all-targets --all-features -- -D warnings` | Passed. |
| `git diff --check` | Passed. |

The compiler emitted non-fatal AWS-LC environment warnings about NASM and an
optional C feature probe; the Rust, C, JNI, and Java build/test commands above
completed successfully.

## Rust public-export audit comparison

The same pinned Rust 1.94.0 audit was run against the 0054 base worktree and the
0057 worktree:

| Snapshot | Observed | Manifest / matched | Approved exclusions | Missing from manifest | Other mismatches/issues |
|---|---:|---:|---:|---:|---:|
| 0054 base | 10,432 | 9,503 | 22 | 907 | 0 |
| 0057 after reconciliation | 10,490 | 9,521 | 62 | 907 | 0 |

Both live `audit_rust_exports.py --check` runs exit 1 because of the same 907
unrelated paths. The feature introduced exactly 42 fresh-rustdoc paths: 40
standard/blanket trait implementation members for the new `WireEncoding`
aliases and two `TransportCauseCategory::EncodingMismatch` variants. The
manifest records the 40 language-level trait paths as approved exclusions;
the two diagnostic variants are blocked like the existing transport cause
category and cite FR-004. Thus 0057 adds no residual audit delta. The checked-in
static inventory test passes.

The remaining 907 paths are already present in the 0054 audit. Their package
distribution is 522 in `kmipkit-protocol`, 206 in `kmipkit`, and 179 in
`kmipkit-client`. The largest root groups are
`kmipkit_protocol::profiles_generated` (140),
`kmipkit_protocol::ProfileReadinessBlocker` (56),
`ProfileValidationErrorCode` (39 per package), and
`ProfileValidationError` (35 per package). Representative paths are derived
trait methods for `ProfileObjectState` and generated profile metadata. They
are preserved as a known broader inventory gap; this evidence does not claim
the full fresh-rustdoc audit is green.

Task 6 remains open. This evidence does not claim that coverage, whole-workspace
gates, interoperability against independent KMIP implementations, or release
readiness have been completed.
