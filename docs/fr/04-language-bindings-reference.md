# 04. Référence des Bindings Multi-Langages (Rust · C · Java · Python)

KMIPKit garantit une **parité fonctionnelle 1:1** entre Rust, C, Java et Python. Tous les adaptateurs sont vérifiés par rapport au manifeste d'API partagé (`verification/api/`).

---

## 1. Parité Fonctionnelle & Conventions d'Exécution

| Aspect | Rust (`kmipkit`) | C (`kmipkit.h`) | Java (`org.kmipkit`) | Python (`kmipkit`) |
| :--- | :--- | :--- | :--- | :--- |
| **Noyau Sous-Jacent** | Rust Natif | `libkmipkit_ffi` (ABI C11) | JNI `->` `libkmipkit_ffi` | CFFI `->` `libkmipkit_ffi` |
| **Gestion Mémoire** | RAII `Drop` + `Zeroize` | Destructeurs `kmipkit_*_free()` | `AutoCloseable` (`try-with-resources`) | Context managers (`with ... as ...`) |
| **Modèle d'Erreurs** | `Result<T, ClientError>` | `kmipkit_status_t` + `kmipkit_last_error_get_v1()` | Exceptions typées (`ProtocolException`, `TransportException`, `ServerResultException`) | Exceptions typées (`KmipProtocolError`, `KmipTransportError`, `KmipServerResultError`) |
| **Sécurité Thread** | `&mut Client` | 1 handle client par thread | Synchronisé par instance `Client` | Synchronisé par instance `Client` |
| **Fluent Builders** | Oui | Oui (`kmipkit_fluent.h`) | Oui (`org.kmipkit.fluent.*`) | Oui (`client.create_symmetric_key()`) |

---

## 2. Façade Rust (`kmipkit`)

### Gestion des Erreurs avec `RequestDeliveryState`

```rust
use kmipkit::client::{ClientError, ClientErrorCategory};
use kmipkit::transport::RequestDeliveryState;

match client.activate(key_id.clone()).send() {
    Ok(resp) => println!("Activé : {:?}", resp.unique_identifier()),
    Err(err) => match err.category() {
        ClientErrorCategory::Validation => {
            eprintln!("Erreur de validation locale (jamais envoyée sur le fil)");
        }
        ClientErrorCategory::Transport => {
            if err.delivery_state() == Some(RequestDeliveryState::NotSent) {
                eprintln!("Connexion échouée avant écriture — réessai sûr");
            } else {
                eprintln!("État réseau ambigu — vérifier l'état sur le serveur avant réessai !");
            }
        }
        ClientErrorCategory::ServerResult => eprintln!("Erreur serveur KMIP : {err}"),
        ClientErrorCategory::Protocol => eprintln!("Erreur de protocole : {err}"),
    },
}
```

### Validation Preflight (Zéro Appel Réseau)

```rust
let report = request.validation_report();
if !report.is_valid() {
    for issue in report.issues() {
        eprintln!("Problème preflight : {issue:?}");
    }
}
```

---

## 3. ABI C11 Stable (`bindings/c/`)

### Versionnage ABI & Snapshots d'Erreur

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

### Intégration & Consommateur CMake

```cmake
# Build du consommateur local du dépôt :
# cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"

# En-têtes et liaison pour une application C11 externe :
add_executable(my_hsm_app main.c)
target_include_directories(my_hsm_app PRIVATE /chemin/vers/KMIPKit/bindings/c/include)
target_link_libraries(my_hsm_app PRIVATE /chemin/vers/libkmipkit_ffi.so)
```

### Règles de Propriété Mémoire
1. **Ne jamais** appeler `free()` de `stdlib.h` sur un pointeur retourné par `kmipkit_*`. Toujours utiliser `kmipkit_*_free()`.
2. Les tampons d'entrée passés à `kmipkit_*` ne sont empruntés que pour la durée de l'appel.
3. Tous les tampons contenant des secrets retournés par KMIPKit sont zéroïsés dans `kmipkit_*_free()`.

---

## 4. Bindings Java (`bindings/java/`)

### Configuration Gradle / Maven & Override Natif

```kotlin
dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0")
}
```

Pour forcer une bibliothèque JNI explicite en environnement restreint :
```bash
java -Dkmipkit.nativeLibrary=/opt/kmipkit/lib/libkmipkit_jni.so -jar app.jar
```

### Exécution en Lot & Authentification Java

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
        System.out.println("Créée et activée en 1 RTT : " + keyId);
    }
}
```

---

## 5. Bindings Python (`bindings/python/`, Python 3.12)

### Installation & Chargement Natif

```bash
pip install kmipkit
# Ou spécifier une bibliothèque native compilée par chemin absolu :
export KMIPKIT_NATIVE_LIBRARY=/opt/kmipkit/libkmipkit_ffi.so
```

### Exemple HTTPS + JSON en Python

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
    print("Versions KMIP supportées :", versions.protocol_versions)
```
