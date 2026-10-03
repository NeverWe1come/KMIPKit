# KMIPKit

KMIPKit is a security focused, open source client library for **OASIS KMIP
2.1**. Its core is written in Rust and exposed through a stable C ABI so that
the same implementation can serve multiple language ecosystems.

The repository is currently in its private design and foundation phase. No
production ready release exists yet.

## Planned 1.0 scope

- Complete KMIP 2.1 client initiated operations using TTLV.
- Raw TLS and HTTPS transports, both with TLS 1.3 and mutual TLS.
- Synchronous Rust client and KMIP protocol asynchronous operations.
- High level, typed protocol, and generic TTLV APIs.
- Rust, C, Java, and Python support with functional parity.
- Traceability from every applicable normative requirement to code and tests.
- Generic support for vendor extensions and a future extension SDK.

Server initiated operations are planned for 1.1. JSON and XML encodings are
planned for later compatible releases.

## Rust workspace

The repository contains a Cargo workspace with seven crates. See the
[Rust workspace guide](docs/development/rust-workspace.md) for the PowerShell
commands and local development workflow.

To inspect and compile the workspace:

```powershell
cargo metadata --format-version 1 --no-deps
cargo check --workspace --all-targets --all-features
```

The crates are structural scaffolding at this stage; KMIP operations and
transport behavior are implemented through approved specifications.

## Documentation

Start with [the documentation index](docs/README.md). AI agents and
contributors must read [AGENTS.md](AGENTS.md) before making changes.

## Español

KMIPKit será una biblioteca cliente Open Source para OASIS KMIP 2.1. Tendrá
un núcleo en Rust, una ABI C estable y adaptadores idiomáticos para varios
lenguajes. La versión 1.0 cubrirá todas las operaciones iniciadas por el
cliente mediante TTLV y ofrecerá APIs de alto nivel, tipada y TTLV genérica.

Consulta [el índice de documentación](docs/README.md) para conocer el diseño,
la metodología y el roadmap.

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
