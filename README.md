# KMIPKit

KMIPKit is an open source KMIP client with a Rust core, a stable C ABI, and
Java and Python adapters. The review branch targets KMIP 2.1 by default, with
version-gated support for KMIP 2.0 and KMIP 3.0. It supports TTLV, JSON, and
XML; TLS 1.3 mutual authentication; raw TLS and HTTPS/HTTP 1.1; and Rust, C,
Java, and Python interfaces. See the verification materials for the exact
scope and evidence; this summary is not a conformance claim.

**Current implementation and release evidence:** see the dated
[project status](docs/project-status.md). Integrated code does not itself
establish profile conformance, external interoperability, or release readiness.

## Workspace

The Cargo workspace contains eight crates covering the TTLV model, KMIP
protocol, transport, client, Rust facade, C ABI, shared test support, and
standalone test server. The independent cloud KMS proxy is under `kmip-proxy/`
with its own workspace and dependency boundary. See the
[architecture overview](docs/architecture/overview.md) and
[Rust workspace guide](docs/development/rust-workspace.md).

Inspect and compile the Rust workspace with:

```sh
cargo metadata --format-version 1 --no-deps
cargo check --workspace --all-targets --all-features
```

## Documentation

Start with the [documentation index](docs/README.md). Contributors can also
read the repository's [contribution guide](CONTRIBUTING.md) and
[governance](GOVERNANCE.md).

## Español

KMIPKit es una biblioteca cliente KMIP de código abierto, con núcleo Rust,
ABI C estable y adaptadores para Java y Python. La rama de revisión usa
KMIP 2.1 por defecto y ofrece soporte acotado y validado por versión para KMIP
2.0 y KMIP 3.0. Incluye TTLV, JSON y XML; TLS 1.3 con
autenticación mutua; conexiones TLS directas y HTTPS/HTTP 1.1; operaciones de
cliente y las operaciones servidor-cliente aprobadas; y capacidades
equivalentes en Rust, C, Java y Python.

Consulta el [estado del proyecto](docs/project-status.md) para ver el snapshot
documentado y el [índice de documentación](docs/README.md) para las guías de
uso, arquitectura y verificación.

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
