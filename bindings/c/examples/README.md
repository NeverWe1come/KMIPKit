# C vendor extension example

`vendor_extension.c` registers two data-only definitions and freezes them into
a client configuration. It inspects a valid inbound value as recognized and
reads the original generic TTLV tags and values. It then inspects the same
discriminator with an out-of-range value: the extension remains unrecognized,
and its original generic TTLV tags and value remain available. The example
also validates two outbound values, chooses each Criticality Indicator
explicitly, and attaches both extensions to one `Discover Versions` batch
item in caller order. It verifies the attached identities and criticality
values.

Unknown response extensions still follow KMIPKIT-0007: an unrecognized
critical extension is rejected and an unrecognized non-critical extension
remains available as generic TTLV. This example inspects local values and does
not exercise response transport handling.

The C API does not currently expose a send or message-encoding operation, so
this example does not contact a KMIP server.

The payload uses the same discriminator and value Tags as the shared
`known.alpha` extension fixture (`0x420001` and `0x420004`), so the example
does not add a new Tag allocation.

The sample values are non-secret. Treat real vendor payloads as sensitive:
do not print or log them. Any bytes or scalars copied out while inspecting a
generic TTLV view are caller-owned, outside KMIPKit's zeroization guarantee,
and must be cleared by the caller when needed. Release every opaque C handle
with its matching `kmipkit_*_release` function.

From the repository root, build the FFI library, configure the C consumer, and
run the example's CTest smoke test:

```sh
cargo build -p kmipkit-ffi
cmake -S bindings/c -B build/c-consumer
cmake --build build/c-consumer --target kmipkit_vendor_extension_example
ctest --test-dir build/c-consumer -R '^kmipkit_vendor_extension_example$' --output-on-failure
```
