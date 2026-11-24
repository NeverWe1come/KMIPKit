# 04. Multi-Language Bindings Reference (Rust · C · Java · Python)

KMIPKit guarantees **1:1 functional parity** across Rust, C, Java, and Python. All four adapters are verified against a shared versioned API manifest (`verification/api/`).

---

## 1. Feature Parity & Runtime Conventions

| Aspect | Rust (`kmipkit`) | C (`kmipkit.h`) | Java (`org.kmipkit`) | Python (`kmipkit`) |
| :--- | :--- | :--- | :--- | :--- |
| **Underlying Core** | Native Rust | `libkmipkit_ffi` (C11 ABI) | JNI -> `libkmipkit_ffi` | CFFI -> `libkmipkit_ffi` |
| **Resource Cleanup** | RAII `Drop` + `Zeroize` | Explicit `kmipkit_*_free()` | `AutoCloseable` (`try-with-resources`) | Context managers (`with ... as ...`) |
| **Error Model** | `Result<T, ClientError>` | `kmipkit_status_t` + `kmipkit_last_error_get()` | Typed exceptions (`ProtocolException`, `TransportException`, `ServerResultException`) | Typed exceptions (`KmipProtocolError`, `KmipTransportError`, `KmipServerResultError`) |
| **Thread Safety** | `&mut Client` (serialized per worker) | 1 client handle per thread (or external mutex) | Synchronized per `Client` instance | Synchronized per `Client` instance |
| **Fluent Builders** | Yes | Yes (`kmipkit_fluent.h`) | Yes (`org.kmipkit.fluent.*`) | Yes (`client.create_symmetric_key()`) |

---

## 2. Rust Facade (`kmipkit`)

### Error Handling with Delivery State

```rust
use kmipkit::client::{ClientError, ClientErrorCategory};
use kmipkit::transport::RequestDeliveryState;

match client.activate(key_id.clone()).send() {
    Ok(resp) => println!("Activated: {:?}", resp.unique_identifier()),
    Err(err) => match err.category() {
        ClientErrorCategory::Validation => {
            // Guaranteed RequestDeliveryState::NotSent — safe to fix input and retry
            eprintln!("Local validation error (not sent)");
        }
        ClientErrorCategory::Transport => {
            if err.delivery_state() == Some(RequestDeliveryState::NotSent) {
                eprintln!("Connection failed before write — safe to retry");
            } else {
                eprintln!("Ambiguous network failure — query key state before retrying!");
            }
        }
        ClientErrorCategory::ServerResult => {
            eprintln!("KMIP server returned Operation Failed: {err}");
        }
        ClientErrorCategory::Protocol => {
            eprintln!("Protocol decoding/framing error: {err}");
        }
    },
}
```

### Preflight Validation (Zero Network Calls)

```rust
let report = request.validation_report();
if !report.is_valid() {
    for issue in report.issues() {
        eprintln!("Preflight issue: {issue:?}");
    }
}
```

---

## 3. Stable C11 ABI (`bindings/c/`)

### ABI Versioning & Error Inspection

All versioned C structs require `struct_size` and `abi_version` initialization:

```c
#include "kmipkit.h"

void inspect_last_error(void) {
    kmipkit_error_snapshot_v1_t snap = {0};
    snap.struct_size = sizeof(snap);
    snap.abi_version = 1;

    if (kmipkit_last_error_get_v1(&snap) == KMIPKIT_OK) {
        printf("status=%u category=%u cause=%u delivery=%u\n",
               snap.status, snap.category, snap.cause, snap.delivery_state);
    }
    kmipkit_last_error_clear();
}
```

### CMake Consumer & Linking

```cmake
# Local repository consumer build:
# cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"

# External C11 target linking:
add_executable(my_hsm_app main.c)
target_include_directories(my_hsm_app PRIVATE /path/to/KMIPKit/bindings/c/include)
target_link_libraries(my_hsm_app PRIVATE /path/to/libkmipkit_ffi.so)
```

### Memory Ownership Rules
1. **Never** call `free()` from `stdlib.h` on any pointer returned by `kmipkit_*`. Always call the matching `kmipkit_*_free()` destructor.
2. Input buffers passed to `kmipkit_*` are borrowed only for the duration of the function call.
3. All secret-bearing byte buffers returned by KMIPKit are zeroized inside `kmipkit_*_free()`.

---

## 4. Java Bindings (`bindings/java/`)

### Gradle / Maven Setup

```kotlin
// build.gradle.kts
dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0")
}
```

### Batch Execution & Authentication in Java

```java
import org.kmipkit.credentials.*;
import org.kmipkit.extensions.*;
import org.kmipkit.protocol.KmipValues;

try (Client client = Client.create(config, transport);
     Authentication auth = Authentication.usernameAndPassword("svc-payments", SecretText.fromChars(passwordChars))) {

    var batch = ClientBatch.builder()
        .authentication(auth)
        .addItem(ClientBatchItem.createSymmetricKey(KmipValues.CryptographicAlgorithm.AES, 256))
        .addItem(ClientBatchItem.activate(UniqueIdentifier.idPlaceholder()))
        .build();

    try (ClientBatchResponse response = client.execute(batch)) {
        String keyId = response.item(0).asCreate().uniqueIdentifier().orElseThrow();
        System.out.println("Created and activated in 1 RTT: " + keyId);
    }
}
```

---

## 5. Python Bindings (`bindings/python/`)

### Installation & Native Library Loading

```bash
pip install kmipkit
# Or load a custom-built native library explicitly:
export KMIPKIT_NATIVE_LIBRARY=/opt/kmipkit/libkmipkit_ffi.so
```

### HTTPS + JSON + Async Poll Example in Python

```python
from kmipkit import (
    Client,
    ClientConfiguration,
    RequestOptions,
    TransportConfiguration,
    WireEncoding,
)

transport = (
    TransportConfiguration.https("https://kms.internal.example.com:8443/kmip")
    .with_client_identity_pem_files("/etc/kmip/client.crt", "/etc/kmip/client.key")
    .with_trust_ca_pem_file("/etc/kmip/ca.crt")
    .build()
)

config = ClientConfiguration.defaults().with_wire_encoding(WireEncoding.JSON)

with Client(config, transport) as client:
    # Override encoding to XML for a single call
    xml_opts = RequestOptions().with_wire_encoding(WireEncoding.XML)
    versions = client.discover_versions(options=xml_opts)
    print("Supported KMIP versions:", versions.protocol_versions)
```
