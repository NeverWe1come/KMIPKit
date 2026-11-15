# KMIPKIT-0053 External Consumer Quickstart

The four external applications compile against the public Rust, C, Java, and
Python package surfaces. They construct a typed attribute, managed object and
key material, cryptographic parameters, and demo Authentication; locally
validate and prepare a Ping client batch; then parse a synthetic successful
Ping response and inspect its generic TTLV tree. The response is constructed
in memory for this example. **There is no KMIP server required or contacted**, so the
quickstart demonstrates package behavior and parsing, not transport
interoperability or server acceptance. Demo values are non-secret placeholders.

## Prerequisites

- A KMIPKit checkout and network access only when uncached build dependencies
  must be downloaded.
- Rust and Cargo with toolchain 1.94 (`rustup toolchain install 1.94`).
- On Windows, 64-bit Visual Studio C++ Build Tools, PowerShell, and
  `vswhere.exe` at its standard Visual Studio Installer path.
- On Linux or macOS, a C compiler (`cc`), JDK 17, Maven, and a POSIX shell.
- Python 3.12 and `uv`.

## Run the consumers

From the repository root, run the platform runner. It builds the native
components, compiles each external package consumer, and runs the applications.
The Java package is installed in Maven's user-local repository; its consumer
uses the JNI library built from this checkout. Python resolves the local
editable binding with the checked-in consumer lock file.

```powershell
& .\tests\consumers\0053-parity\verify.ps1
```

```sh
./tests/consumers/0053-parity/verify.sh
```

The automated guide/source/runner wiring check can be run independently:

```powershell
python -B -m unittest discover -s tests/docs -p test_0053_executable_consumer_examples.py -v
```

## What each application demonstrates

- **Rust:** `Attribute<typed::Comment>`, `ManagedObject::OpaqueObject`,
  `KeyMaterial`, `CryptographicParametersBuilder`, `Authentication` attached
  to `ClientBatch`, `ClientRequest::validation_report`, a Ping batch item, and
  `ResponseMessage::try_from_ttlv` with generic `with_ttlv` access.
- **C:** the corresponding public ABI constructors for a typed attribute,
  opaque managed object, key material, Cryptographic Parameters, Credential
  and Authentication; `kmipkit_client_batch_set_authentication`, Ping
  validation, and `kmipkit_response_message_from_ttlv` followed by response
  item TTLV access.
- **Java:** `TypedAttribute`, `ManagedObject`, `KeyMaterial`,
  `CryptographicParameters`, `Authentication` attached to `ClientBatch`, Ping
  validation, and `ResponseMessage.fromTtlv` plus its response-item TTLV.
- **Python:** `TypedAttribute`, `ManagedObject`, `KeyMaterial`,
  `CryptographicParameters`, `Authentication` attached to `client.Batch`, Ping
  validation, and `ResponseMessage.from_ttlv` plus its response-item TTLV.

Each external consumer now calls `Client.execute` for Ping over a configured
TLS 1.3/mTLS transport and checks its typed result and generic Response Payload.
The runner starts a loopback-only fixture responder from disposable test
certificates; it is limited to Ping and does not count as interoperability
evidence or as a KMIP server implementation. The separate synthetic response
continues to exercise the public parser and generic TTLV accessors.

These examples do not independently demonstrate complete operation-checked
response-reader parity. Typed foreign readers for Cancel, Poll, Process, and
Query Async are implemented and tracked as complete in
[converged task T078](tasks.md); broader parity evidence is in the operation
binding matrix and response tests.

## Runnable sources

- [Rust consumer](../../tests/consumers/0053-parity/rust/src/main.rs)
- [C consumer](../../tests/consumers/0053-parity/c/main.c)
- [Java consumer](../../tests/consumers/0053-parity/java/src/main/java/Consumer.java)
- [Python consumer](../../tests/consumers/0053-parity/python/main.py)
- [Consumer package declarations and runner notes](../../tests/consumers/0053-parity/README.md)
- [Windows runner](../../tests/consumers/0053-parity/verify.ps1)
- [POSIX runner](../../tests/consumers/0053-parity/verify.sh)

## API guides

- English: [Language packages](../../docs/user-guide/en/language-packages.md),
  [client execution](../../docs/user-guide/en/client-execution.md),
  [typed bindings and response data](../../docs/user-guide/en/typed-bindings-and-response-data.md),
  and [Query and Ping](../../docs/user-guide/en/query-ping.md).
- Español: [Paquetes de idiomas](../../docs/user-guide/es/paquetes-de-idiomas.md),
  [ejecución del cliente](../../docs/user-guide/es/ejecucion-cliente.md),
  [objetos gestionados y material de clave](../../docs/user-guide/es/objetos-gestionados-y-material-de-clave.md),
  and [operaciones Query y Ping](../../docs/user-guide/es/operaciones-query-ping.md).
