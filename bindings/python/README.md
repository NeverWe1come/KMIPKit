# KMIPKit Python binding

Python 3.12 API and CFFI bridge to the Rust KMIPKit implementation.

## Documentation

- [Installation and build](docs/installation.md)
- [Native library](docs/native-library.md)
- [Testing](docs/testing.md)
- [Examples](examples/README.md)

## First local example

After following the installation guide, run from the repository root:

```sh
python bindings/python/examples/vendor_extension_registry.py
```

See the example guide for platform requirements and what this command exercises.

## Shared behavior

The [user guides](../../docs/README.md) describe KMIP operations and transports.
The [binding architecture](../../docs/architecture/ffi-and-bindings.md) explains
shared ownership and API boundaries. The [bindings index](../README.md) links
the other languages. Local example success does not establish full language
parity or KMIP conformance.
