# Vendor extensions

KMIP Message Extensions carry vendor-defined data in the standard fields from
KMIP 2.1 §9.13, Table 418. KMIPKit keeps each vendor definition in an immutable
registry owned by one client configuration. A registration describes data; it
does not load code or grant access to the transport.

The registry APIs cover extension registration, validation, request
attachment, response inspection, and generic TTLV preservation. The examples
build a typed Discover Versions batch item and do not open a network
connection.

## Define an extension

Each definition includes a non-empty Vendor Identification using ASCII
letters, digits, `_`, or `.`, an extension name and version, KMIP and KMIPKit
compatibility ranges, a data-only TTLV schema, and one exact discriminator
path and value. Optional Extension Information provides local metadata.

The discriminator identifies the vendor payload inside its registered vendor
namespace. Matching Vendor Identification alone never recognizes a payload.
KMIPKit validates the complete schema before returning a sealed registered
value. The typed request API does not accept a raw body, arbitrary TTLV Item,
or caller-defined conversion.

The executable Rust example builds a definition, validates two payloads,
selects a Criticality Indicator for each use, and attaches both in caller
order:

```text
cargo run -p kmipkit-client --example vendor_extension
```

Equivalent examples live with each language adapter:

| Adapter | Run the example |
|---|---|
| Rust | `cargo run -p kmipkit-client --example vendor_extension` |
| C | Build `kmipkit-ffi`, configure and build `bindings/c` with CMake, then run `ctest --test-dir target/c-examples --output-on-failure`. This runs `bindings/c/examples/vendor_extension.c`. |
| Java 17 | `mvn -f bindings/java/pom.xml test`; the JNI-backed example test runs `bindings/java/examples/VendorExtensionExample.java`. |
| Python 3.12 | In a pinned Python environment, run `maturin develop` in `bindings/python`, then `python examples/vendor_extension_registry.py`. The package suite also runs the example smoke test. |

For C, build the native library before CMake so the generated C ABI is
available:

```text
cargo build -p kmipkit-ffi
cmake -S bindings/c -B target/c-examples
cmake --build target/c-examples
ctest --test-dir target/c-examples --output-on-failure
```

## Attach an extension to a request

The caller chooses the Criticality Indicator every time it attaches a
registered value. KMIPKit has no default criticality. Repeated Message
Extensions preserve attachment order as allowed by §8.3, Table 396. A value
validated against another client's registry cannot be sent by a client
configured with a different registry; execution rejects it before encoding or
transport.

Client execution emits Vendor Identification, the explicit Criticality
Indicator, and the validated Vendor Extension Structure in Table 418 order.
The extension layer uses the existing private request writer and its
zeroizing encoded-buffer owner. It does not introduce a raw-message writer or
retry KMIP requests automatically.

## Inspect server extensions

On receive, KMIPKit checks the vendor and exact registered discriminator. Zero
matching definitions or more than one matching discriminator leaves the
payload unrecognized. Exactly one match still has to pass the complete
registered schema. Registration order does not affect the result.

Recognized values expose a typed view and retain their original generic TTLV
subtree. Unrecognized non-critical extensions remain available as generic
TTLV. Unknown critical extensions follow KMIPKIT-0007 and are rejected;
registry support does not weaken that rule. Generic values preserve unknown
tags, enumeration values, bitmask bits, repeated fields, and child order that
the KMIPKit TTLV allocation policy accepts.

The local Extension Information list/map corresponds to Query Extension List
and Query Extension Map metadata (§7.13, Table 365; §11.44, Table 476). It
describes this local registry only. It does not assert that a remote server
supports an extension, and this feature does not execute Query.

## Limits and secret handling

Default TTLV decoding limits are 16 MiB per message, depth 64, and 100,000
Items. Registry defaults and hard maxima are:

| Limit | Default | Hard maximum |
|---|---:|---:|
| Definitions per registry | 256 | 1,024 |
| Aggregate schema nodes | 16,384 | 100,000 |
| Child rules per Structure | 256 | 4,096 |
| UTF-8 bytes per identity/metadata field | 4,096 | 4,096 |
| Aggregate identity/metadata bytes | 1 MiB | 16 MiB |
| Bytes per discriminator scalar | 4,096 | 4,096 |
| Aggregate discriminator scalar bytes | 1 MiB | 16 MiB |
| Constraint members per rule | 256 | 4,096 |
| Aggregate constraint members | 16,384 | 100,000 |
| Payload index records | 200,000 | 200,000 |
| Discriminator lookup comparisons | 1,048,576 | 4,194,304 |
| Schema/discriminator depth | 64 | 64 |

Limits can be lowered or raised up to their hard maximum. Registry construction
checks aggregate counts before cloning or reserving. A runtime budget failure
returns a redacted resource-limit error without a partial typed result.
Java metadata strings must contain well-formed UTF-16; a rejected description
does not transfer or close the original `ExtensionInformation` value.

KMIPKit redacts extension payloads from errors and default formatting. Its
owned encoded request buffer is zeroized when sending finishes, whether the
write succeeds or fails. KMIPKit clears its temporary Java byte-array copies
after synchronous native calls. The JVM, callers, and Python runtime may still
create managed copies that KMIPKit cannot deterministically overwrite. Do not
put secrets in debug output or logs.

## Compatibility boundary

The registry is client-local and immutable after construction. Its schemas are
declarative and cannot execute scripts, callbacks, or dynamically loaded
plugins. This feature does not claim support for every server's vendor policy;
server acceptance depends on the peer's implementation and configuration.
