# KMIPKit C binding

C headers and consumers for the Rust KMIPKit library through its C ABI.

## Documentation

- [Installation and build](docs/installation.md)
- [Native library](docs/native-library.md)
- [Testing](docs/testing.md)
- [Examples](examples/README.md)

## First local example

After following the installation guide, run from the repository root:

```sh
cargo build --locked -p kmipkit-ffi
cmake -S bindings/c -B build/c-consumer
cmake --build build/c-consumer --config Release
ctest --test-dir build/c-consumer -C Release \
  -R '^kmipkit_vendor_extension_example$' --output-on-failure
```

See the example guide for platform requirements and what this command exercises.

## Shared behavior

The [user guides](../../docs/README.md) describe KMIP operations and transports.
The [binding architecture](../../docs/architecture/ffi-and-bindings.md) explains
shared ownership and API boundaries. The [bindings index](../README.md) links
the other languages. Local example success does not establish full language
parity or KMIP conformance.
