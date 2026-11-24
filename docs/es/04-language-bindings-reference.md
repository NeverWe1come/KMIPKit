# 04. Referencia de Bindings Multi-Lenguaje (Rust · C · Java · Python)

KMIPKit garantiza **paridad funcional 1:1** entre Rust, C, Java y Python. Los cuatro adaptadores se verifican automáticamente contra un manifiesto de API versionado compartido (`verification/api/`).

---

## 1. Matriz de Paridad y Convenciones de Runtime

| Aspecto | Rust (`kmipkit`) | C (`kmipkit.h`) | Java (`org.kmipkit`) | Python (`kmipkit`) |
| :--- | :--- | :--- | :--- | :--- |
| **Núcleo Subyacente** | Rust Nativo | `libkmipkit_ffi` (ABI C11) | JNI -> `libkmipkit_ffi` | CFFI -> `libkmipkit_ffi` |
| **Liberación de Recursos** | RAII `Drop` + `Zeroize` | `kmipkit_*_free()` explícito | `AutoCloseable` (`try-with-resources`) | Context managers (`with ... as ...`) |
| **Modelo de Errores** | `Result<T, ClientError>` | `kmipkit_status_t` + `kmipkit_last_error_get_v1()` | Excepciones tipadas (`ProtocolException`, `TransportException`, `ServerResultException`) | Excepciones tipadas (`KmipProtocolError`, `KmipTransportError`, `KmipServerResultError`) |
| **Seguridad de Hilos** | `&mut Client` (serializado por worker) | 1 handle cliente por hilo (o mutex externo) | Sincronizado por instancia `Client` | Sincronizado por instancia `Client` |
| **Fluent Builders** | Sí | Sí (`kmipkit_fluent.h`) | Sí (`org.kmipkit.fluent.*`) | Sí (`client.create_symmetric_key()`) |

---

## 2. Fachada Rust (`kmipkit`)

### Manejo de Errores con Estado de Entrega (`RequestDeliveryState`)

```rust
use kmipkit::client::{ClientError, ClientErrorCategory};
use kmipkit::transport::RequestDeliveryState;

match client.activate(key_id.clone()).send() {
    Ok(resp) => println!("Clave activada: {:?}", resp.unique_identifier()),
    Err(err) => match err.category() {
        ClientErrorCategory::Validation => {
            // Garantizado RequestDeliveryState::NotSent — seguro corregir y reintentar
            eprintln!("Error de validación local (no enviado)");
        }
        ClientErrorCategory::Transport => {
            if err.delivery_state() == Some(RequestDeliveryState::NotSent) {
                eprintln!("Fallo de conexión antes de escribir — seguro reintentar");
            } else {
                eprintln!("Fallo de red ambiguo — consultar estado en servidor antes de reintentar");
            }
        }
        ClientErrorCategory::ServerResult => {
            eprintln!("El servidor KMIP devolvió Operation Failed: {err}");
        }
        ClientErrorCategory::Protocol => {
            eprintln!("Error de decodificación/framing de protocolo: {err}");
        }
    },
}
```

### Validación Preflight (Cero Llamadas de Red)

```rust
let report = request.validation_report();
if !report.is_valid() {
    for issue in report.issues() {
        eprintln!("Problema preflight: {issue:?}");
    }
}
```

---

## 3. ABI C11 Estable (`bindings/c/`)

### Versionado ABI e Inspección de Errores

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

### Integración con CMake

```cmake
find_package(KMIPKit REQUIRED)
add_executable(my_hsm_app main.c)
target_link_libraries(my_hsm_app PRIVATE KMIPKit::kmipkit_ffi)
```

### Reglas de Propiedad de Memoria
1. **Nunca** llames a `free()` de `stdlib.h` sobre punteros devueltos por `kmipkit_*`. Usa siempre el destructor `kmipkit_*_free()` correspondiente.
2. Los buffers de entrada pasados a `kmipkit_*` solo se toman prestados durante la llamada.
3. Todos los buffers con secretos devueltos por KMIPKit se zeroizan automáticamente dentro de `kmipkit_*_free()`.

---

## 4. Bindings Java (`bindings/java/`)

```kotlin
// build.gradle.kts
dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0")
}
```

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
        System.out.println("Creada y activada en 1 RTT: " + keyId);
    }
}
```

---

## 5. Bindings Python (`bindings/python/`)

```bash
pip install kmipkit
export KMIPKIT_NATIVE_LIB=/opt/kmipkit/libkmipkit_ffi.so
```

```python
from kmipkit import Client, ClientConfiguration, RequestOptions, TransportConfiguration, WireEncoding

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
    print("Versiones KMIP soportadas:", versions.protocol_versions)
```
