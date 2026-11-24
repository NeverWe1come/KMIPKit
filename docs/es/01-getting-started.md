# 01. Inicio Rápido y Primeros Pasos

Esta guía explica cómo compilar **KMIPKit** con su proveedor criptográfico OpenSSL fijado por checksum y ejecutar tu primer ciclo de vida completo de claves KMIP (`Discover Versions` -> `Create` -> `Activate` -> `Encrypt` -> `Decrypt` -> `Revoke` -> `Destroy`) en **Rust, C, Java y Python**.

---

## 1. Requisitos Previos y OpenSSL Nativo

KMIPKit utiliza **OpenSSL 3.5.x** como proveedor criptográfico y TLS (`rustls-openssl`) unificado tanto en la biblioteca cliente como en `kmip-proxy`. Usar el script fijado del repositorio garantiza un comportamiento idéntico de TLS y PQC (`ML-DSA`) en Linux, macOS y Windows.

| Requisito | Versión Mínima | Notas |
| :--- | :--- | :--- |
| **Toolchain Rust** | `1.94+` (Edition 2024) | Workspace principal y `kmip-proxy` |
| **OpenSSL** | `3.5.9` (fijado) | Compilado con `verification/security/openssl/build.py` |
| **Compilador C / CMake** | C11 / CMake `3.20+` | Necesario para bindings C y biblioteca nativa JNI |
| **Java** | JDK `17+` | Bindings Java (Gradle / Maven) |
| **Python** | Python `3.10+` | Bindings CFFI |

### Compilar OpenSSL Fijado y Verificar el Workspace

```bash
# 1. Compilar OpenSSL estático verificado por checksum
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1

# 2. Verificar y ejecutar tests del workspace principal (8 crates)
cargo check --workspace --all-targets --all-features
cargo test --workspace

# 3. Verificar y ejecutar tests del workspace independiente KMIP Proxy
cargo check --manifest-path kmip-proxy/Cargo.toml --all-targets
cargo test --manifest-path kmip-proxy/Cargo.toml
```

---

## 2. Levantar un Servidor KMIP Local de Pruebas (Cero Configuración)

Para desarrollo local, arranca `kmipkit-proxy` con su `AutoPki` efímera e `InMemoryMockProvider`:

```bash
cargo run --manifest-path kmip-proxy/Cargo.toml -- \
  --listen 127.0.0.1:5696 \
  --provider mock
```

---

## 3. Inicio Rápido: Rust

Añade `kmipkit` a tu `Cargo.toml`:

```toml
[dependencies]
kmipkit = "1.0.0"
```

### Ciclo de Vida Completo (mTLS Raw TLS + TTLV)

```rust
use std::time::Duration;

use kmipkit::client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit::client::fluent::CryptographicUsageMask;
use kmipkit::client::{Client, WireEncoding};
use kmipkit::protocol::{CryptographicAlgorithm, ProtocolVersion, RevocationReasonCode};
use kmipkit::transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar transporte mTLS (TLS 1.3 obligatorio por defecto)
    let endpoint = Endpoint::raw_tls("kms.internal.example.com", 5696);
    let transport_config = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file("/etc/kmip/client.crt"),
            PrivateKeyInput::from_pem_file("/etc/kmip/client.key"),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file("/etc/kmip/ca.crt"),
        ]))
        .timeouts(
            TimeoutPolicy::default()
                .with_total(TimeoutLimit::Bounded(Duration::from_secs(15))),
        )
        .build()?;

    // 2. Configuración inmutable del cliente (KMIP 2.1, TTLV)
    let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;
    let client_config = ClientConfiguration::new(registry)
        .with_protocol_version(ProtocolVersion::V2_1)
        .with_wire_encoding(WireEncoding::Ttlv);

    let mut client = Client::new(client_config, transport_config)?;

    // 3. Crear clave simétrica AES-256 con la API Fluent
    let created = client
        .create_symmetric_key()
        .algorithm(CryptographicAlgorithm::Aes)
        .length(256)
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("app-data-encryption-key")
        .send()?;
    let key_id = created.unique_identifier().expect("ID de clave devuelto");
    println!("Clave AES-256 creada: {key_id}");

    // 4. Activar la clave
    client.activate(key_id.clone()).send()?;

    // 5. Cifrar y Descifrar datos
    let enc = client.encrypt(key_id.clone(), b"Payload de pago sensible").send()?;
    let dec = client.decrypt(key_id.clone(), enc.data().expect("ciphertext")).send()?;
    assert_eq!(dec.data().unwrap(), b"Payload de pago sensible");

    // 6. Revocar y Destruir
    client.revoke(key_id.clone(), RevocationReasonCode::CessationOfOperation).send()?;
    client.destroy(key_id).send()?;

    Ok(())
}
```

---

## 4. Inicio Rápido: C (ABI C11 Estable)

```c
#include <stdio.h>
#include "kmipkit.h"
#include "kmipkit_fluent.h"

int main(void) {
    uint32_t major = 0, minor = 0, patch = 0, abi = 0;
    kmipkit_version_get(&major, &minor, &patch, &abi);
    printf("KMIPKit C ABI v%u.%u.%u (ABI rev %u)\n", major, minor, patch, abi);

    kmipkit_transport_config_t *transport = NULL;
    if (kmipkit_transport_config_new_raw_tls(
            "kms.internal.example.com", 5696,
            "/etc/kmip/client.crt", "/etc/kmip/client.key", "/etc/kmip/ca.crt",
            &transport) != KMIPKIT_OK) {
        return 1;
    }

    kmipkit_client_t *client = NULL;
    kmipkit_status_t st = kmipkit_client_new_default(transport, &client);
    kmipkit_transport_config_free(transport);
    if (st != KMIPKIT_OK) return 1;

    kmipkit_discover_versions_response_t *resp = NULL;
    if (kmipkit_client_discover_versions(client, &resp) == KMIPKIT_OK) {
        printf("Discover Versions completado.\n");
        kmipkit_discover_versions_response_free(resp);
    }

    kmipkit_client_free(client);
    return 0;
}
```

---

## 5. Inicio Rápido: Java (JNI)

```java
import org.kmipkit.extensions.*;
import org.kmipkit.protocol.KeyUsage;
import org.kmipkit.protocol.KmipValues;
import java.nio.file.Path;
import java.time.Duration;

public class KmipQuickstart {
    public static void main(String[] args) throws Exception {
        try (TransportConfiguration transport = TransportConfiguration.rawTls("kms.internal.example.com", 5696)
                .clientIdentityPemFiles(Path.of("/etc/kmip/client.crt"), Path.of("/etc/kmip/client.key"))
                .trustCaPemFile(Path.of("/etc/kmip/ca.crt"))
                .totalTimeout(Duration.ofSeconds(15))
                .build();
             ClientConfiguration config = ClientConfiguration.defaults().withWireEncoding(WireEncoding.TTLV);
             Client client = Client.create(config, transport)) {

            var created = client.createSymmetricKey()
                .algorithm(KmipValues.CryptographicAlgorithm.AES)
                .length(256)
                .usageMask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
                .name("java-service-dek")
                .execute();

            String keyId = created.uniqueIdentifier().orElseThrow();
            client.activate(keyId).execute();
            System.out.println("Clave activada: " + keyId);
        }
    }
}
```

---

## 6. Inicio Rápido: Python (CFFI)

```python
from kmipkit import Client, ClientConfiguration, TransportConfiguration, WireEncoding
from kmipkit.protocol import CryptographicAlgorithm, KeyUsage

transport = (
    TransportConfiguration.raw_tls("kms.internal.example.com", 5696)
    .with_client_identity_pem_files("/etc/kmip/client.crt", "/etc/kmip/client.key")
    .with_trust_ca_pem_file("/etc/kmip/ca.crt")
    .with_total_timeout_ms(15_000)
    .build()
)

config = ClientConfiguration.defaults().with_wire_encoding(WireEncoding.TTLV)

with Client(config, transport) as client:
    created = (
        client.create_symmetric_key()
        .algorithm(CryptographicAlgorithm.AES)
        .length(256)
        .usage_mask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
        .name("python-ml-pipeline-key")
        .execute()
    )
    key_id = created.unique_identifier
    client.activate(key_id).execute()

    encrypted = client.encrypt(key_id, b"Hola KMIP 2.1!").execute()
    decrypted = client.decrypt(key_id, encrypted.data).execute()
    assert decrypted.data == b"Hola KMIP 2.1!"
```
