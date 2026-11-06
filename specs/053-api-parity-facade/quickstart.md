# KMIPKIT-0053 External Consumer Quickstart

This quickstart runs four small applications from outside the language
bindings' source directories. Each declares only the corresponding KMIPKit
package as its KMIPKit dependency. These are package-surface and ownership
smoke checks: they construct a Ping request, confirm its KMIP operation value,
and create and navigate an empty generic TTLV Structure. They do not connect to
a KMIP server or demonstrate the complete typed response matrix.

## Prerequisites

- A checkout of KMIPKit and network access for any dependencies not already
  cached locally.
- Rust and Cargo, with the Rust 1.94 toolchain installed (`rustup toolchain
  install 1.94` if needed). The runner compiles the Rust consumer with this
  toolchain.
- On Windows: 64-bit Visual Studio C++ Build Tools, `vswhere.exe` at its
  standard Visual Studio Installer path, and PowerShell. The C and JNI builds
  use the MSVC x64 environment.
- On Linux or macOS: a C compiler (`cc`), a JDK 17 toolchain, Maven, and a POSIX
  shell. The JNI bridge must be buildable with the platform toolchain described
  by `bindings/java/native/build.sh`.
- Python 3.12 and `uv`. `uv` installs the editable Python binding from this
  checkout and builds its native extension.

## Run the consumers

From the repository root, use the runner for the current platform:

```powershell
& .\tests\consumers\0053-parity\verify.ps1
```

```sh
./tests/consumers/0053-parity/verify.sh
```

The scripts build the required native components, compile each standalone
consumer against the package interface, and run it. The Java package is
installed in Maven's user-local repository; its consumer runs with the JNI
library built from this checkout. The Python package is resolved through the
consumer's local editable source and locked dependencies.

## What each consumer checks

- **Rust:** depends only on `kmipkit`; creates Ping and Query request models,
  verifies Ping's operation value (`0x3b`), converts Query to a TTLV payload,
  and compile-checks the `Client::new` and `Client::execute` signatures through
  the facade.
- **C:** includes the public `kmipkit.h`, creates Ping through the C ABI,
  checks operation `0x3b`, creates a TTLV Structure and view, checks its child
  count, then releases all handles.
- **Java:** depends on the `org.kmipkit:kmipkit-java` package, creates Ping,
  checks operation `0x3b`, and uses try-with-resources for the TTLV Structure
  and borrowed view.
- **Python:** depends on the `kmipkit` package, creates Ping and checks
  operation `0x3b`, then uses context managers to close the TTLV Structure and
  its view.

The runnable sources are [Rust](../../tests/consumers/0053-parity/rust/src/main.rs),
[C](../../tests/consumers/0053-parity/c/main.c),
[Java](../../tests/consumers/0053-parity/java/src/main/java/Consumer.java), and
[Python](../../tests/consumers/0053-parity/python/main.py). The platform runners
and package declarations are under
[tests/consumers/0053-parity](../../tests/consumers/0053-parity/README.md).

## API guides

For broader usage and lifetime details, see the maintained language guides:

- [English: language packages](../../docs/user-guide/en/language-packages.md),
  [client execution](../../docs/user-guide/en/client-execution.md),
  [typed bindings and response data](../../docs/user-guide/en/typed-bindings-and-response-data.md),
  and [generic Query and Ping](../../docs/user-guide/en/query-ping.md).
- [Español: paquetes de idiomas](../../docs/user-guide/es/paquetes-de-idiomas.md),
  [ejecución del cliente](../../docs/user-guide/es/ejecucion-cliente.md),
  [objetos gestionados y material de clave](../../docs/user-guide/es/objetos-gestionados-y-material-de-clave.md),
  and [operaciones Query y Ping](../../docs/user-guide/es/operaciones-query-ping.md).
