# 04. Sprach-Bindings-Referenz (Rust · C · Java · Python)

KMIPKit garantiert **1:1 Funktionsparität** zwischen Rust, C, Java und Python, verifiziert gegen ein gemeinsames API-Manifest (`verification/api/`).

---

## 1. Funktionsparität & Laufzeitkonventionen

| Aspekt | Rust (`kmipkit`) | C (`kmipkit.h`) | Java (`org.kmipkit`) | Python (`kmipkit`) |
| :--- | :--- | :--- | :--- | :--- |
| **Kern-Basis** | Natives Rust | `libkmipkit_ffi` (C11 ABI) | JNI `->` `libkmipkit_ffi` | CFFI `->` `libkmipkit_ffi` |
| **Speicherverwaltung** | RAII `Drop` + `Zeroize` | Explizites `kmipkit_*_free()` | `AutoCloseable` (`try-with-resources`) | Context Manager (`with ... as ...`) |
| **Fehlermodell** | `Result<T, ClientError>` | `kmipkit_status_t` + `kmipkit_last_error_get_v1()` | Typisierte Exceptions | Typisierte Exceptions (`KmipProtocolError`, etc.) |
| **Thread-Sicherheit** | `&mut Client` | 1 Client-Handle pro Thread | Pro `Client`-Instanz synchronisiert | Pro `Client`-Instanz synchronisiert |
| **Fluent Builders** | Ja | Ja (`kmipkit_fluent.h`) | Ja (`org.kmipkit.fluent.*`) | Ja (`client.create_symmetric_key()`) |

---

## 2. Rust-Fassade (`kmipkit`)

### Fehlerbehandlung mit `RequestDeliveryState`

```rust
use kmipkit::client::{ClientError, ClientErrorCategory};
use kmipkit::transport::RequestDeliveryState;

match client.activate(key_id.clone()).send() {
    Ok(resp) => println!("Aktiviert: {:?}", resp.unique_identifier()),
    Err(err) => match err.category() {
        ClientErrorCategory::Validation => eprintln!("Lokaler Validierungsfehler (nicht gesendet)"),
        ClientErrorCategory::Transport => {
            if err.delivery_state() == Some(RequestDeliveryState::NotSent) {
                eprintln!("Verbindungsfehler vor Sendung — Retry sicher");
            } else {
                eprintln!("Mehrdeutiger Netzwerkfehler — Zustand erst abfragen!");
            }
        }
        ClientErrorCategory::ServerResult => eprintln!("KMIP-Serverfehler: {err}"),
        ClientErrorCategory::Protocol => eprintln!("Protokollfehler: {err}"),
    },
}
```

### Preflight-Validierung (Ohne Netzwerkanfragen)

```rust
let report = request.validation_report();
if !report.is_valid() {
    for issue in report.issues() {
        eprintln!("Preflight-Problem: {issue:?}");
    }
}
```

---

## 3. Stabile C11-ABI (`bindings/c/`)

### ABI-Versionierung & Fehler-Snapshots

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

### CMake-Consumer & Verlinkung

```cmake
# Lokaler Repository-Consumer-Build:
# cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"

# Externe C11-Anwendung verlinken:
add_executable(my_hsm_app main.c)
target_include_directories(my_hsm_app PRIVATE /pfad/zu/KMIPKit/bindings/c/include)
target_link_libraries(my_hsm_app PRIVATE /pfad/zu/libkmipkit_ffi.so)
```

### Speicherbesitzregeln
1. **Niemals** `free()` aus `stdlib.h` für von `kmipkit_*` zurückgegebene Zeiger aufrufen; stets `kmipkit_*_free()` nutzen.
2. Eingabepuffer werden nur für die Dauer des Funktionsaufrufs geliehen.
3. Alle geheimen Bytepuffer werden beim Freigeben genullt (Zeroization).

---

## 4. Java-Bindings (`bindings/java/`)

### Gradle / Maven & nativer Override

```kotlin
dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0")
}
```

Expliziter Pfad zur JNI-Bibliothek über Property:
```bash
java -Dkmipkit.nativeLibrary=/opt/kmipkit/lib/libkmipkit_jni.so -jar app.jar
```

### Batch-Ausführung & Authentifizierung

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
        System.out.println("In 1 RTT erstellt und aktiviert: " + keyId);
    }
}
```

---

## 5. Python-Bindings (`bindings/python/`, Python 3.12)

### Installation & Laden der nativen Bibliothek

```bash
pip install kmipkit
# Oder expliziten Pfad zur kompilierten nativen Bibliothek setzen:
export KMIPKIT_NATIVE_LIBRARY=/opt/kmipkit/libkmipkit_ffi.so
```

### HTTPS + JSON Beispiel in Python

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
    xml_opts = RequestOptions().with_wire_encoding(WireEncoding.XML)
    versions = client.discover_versions(options=xml_opts)
    print("Unterstützte KMIP-Versionen:", versions.protocol_versions)
```
