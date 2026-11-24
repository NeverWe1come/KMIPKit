# 01. Getting Started & Quickstart

This guide walks you through building **KMIPKit** with its checksum-pinned OpenSSL cryptographic provider and running your first complete KMIP key lifecycle (`Discover Versions` -> `Create` -> `Activate` -> `Encrypt` -> `Decrypt` -> `Revoke` -> `Destroy`) in **Rust, C, Java, and Python**.

---

## 1. Prerequisites & Native OpenSSL Setup

KMIPKit uses **OpenSSL 3.5.x** as its unified cryptographic and `rustls-openssl` TLS provider across both the client library and `kmip-proxy`. Using the repository's pinned build script guarantees identical TLS and PQC (`ML-DSA`) behavior on Linux, macOS, and Windows.

| Requirement | Minimum Version | Notes |
| :--- | :--- | :--- |
| **Rust Toolchain** | `1.94+` (Edition 2024) | Core workspace & `kmip-proxy` |
| **OpenSSL** | `3.5.9` (pinned) | Built via `verification/security/openssl/build.py` |
| **C Compiler / CMake** | C11 / CMake `3.20+` | Required for C bindings & JNI native library |
| **Java** | JDK `17+` | Gradle / Maven Java bindings |
| **Python** | Python `3.10+` | CFFI bindings |

### Build the Pinned OpenSSL & Verify the Workspace

```bash
# 1. Build checksum-pinned static OpenSSL
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1

# 2. Check and test the core 8-crate Rust workspace
cargo check --workspace --all-targets --all-features
cargo test --workspace

# 3. Check and test the standalone KMIP Proxy workspace
cargo check --manifest-path kmip-proxy/Cargo.toml --all-targets
cargo test --manifest-path kmip-proxy/Cargo.toml
```

---

## 2. Launching a Local Test KMIP Server (Zero Config)

For local development, start the built-in `kmipkit-proxy` with its ephemeral `AutoPki` and `InMemoryMockProvider` (or the standalone `kmipkit-test-server` crate):

```bash
# Start a local KMIP 2.1/3.0 mock server on 127.0.0.1:5696
cargo run --manifest-path kmip-proxy/Cargo.toml -- \
  --listen 127.0.0.1:5696 \
  --provider mock
```

---

## 3. Quickstart: Rust

Add `kmipkit` to your `Cargo.toml`:

```toml
[dependencies]
kmipkit = "1.0.0"
```

### Complete Key Lifecycle (mTLS Raw TLS + TTLV)

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
    // 1. Configure mTLS transport (TLS 1.3 enforced by default)
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

    // 2. Build immutable client configuration (KMIP 2.1, TTLV encoding)
    let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;
    let client_config = ClientConfiguration::new(registry)
        .with_protocol_version(ProtocolVersion::V2_1)
        .with_wire_encoding(WireEncoding::Ttlv);

    let mut client = Client::new(client_config, transport_config)?;

    // 3. Create an AES-256 symmetric key using the Fluent Builder API
    let created = client
        .create_symmetric_key()
        .algorithm(CryptographicAlgorithm::Aes)
        .length(256)
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("app-data-encryption-key")
        .send()?;
    let key_id = created.unique_identifier().expect("server returned key ID");
    println!("Created AES-256 key: {key_id}");

    // 4. Activate the key
    client.activate(key_id.clone()).send()?;

    // 5. Encrypt & Decrypt data
    let enc = client.encrypt(key_id.clone(), b"Sensitive payment payload").send()?;
    let dec = client.decrypt(key_id.clone(), enc.data().expect("ciphertext")).send()?;
    assert_eq!(dec.data().unwrap(), b"Sensitive payment payload");

    // 6. Revoke & Destroy cleanly
    client.revoke(key_id.clone(), RevocationReasonCode::CessationOfOperation).send()?;
    client.destroy(key_id).send()?;

    Ok(())
}
```

---

## 4. Quickstart: C (Stable C11 ABI)

Build the native library (`libkmipkit_ffi.so` / `.dylib` / `.dll`) and link against `bindings/c/include/kmipkit.h`:

```c
#include <stdio.h>
#include <string.h>
#include "kmipkit.h"
#include "kmipkit_fluent.h"

int main(void) {
    uint32_t major = 0, minor = 0, patch = 0, abi = 0;
    kmipkit_version_get(&major, &minor, &patch, &abi);
    printf("KMIPKit C ABI v%u.%u.%u (ABI rev %u)\n", major, minor, patch, abi);

    // 1. Build validated mTLS transport configuration
    kmipkit_transport_config_t *transport = NULL;
    kmipkit_status_t st = kmipkit_transport_config_new_raw_tls(
        "kms.internal.example.com", 5696,
        "/etc/kmip/client.crt", "/etc/kmip/client.key", "/etc/kmip/ca.crt",
        &transport
    );
    if (st != KMIPKIT_OK) {
        fprintf(stderr, "Transport config failed (status=%d)\n", st);
        return 1;
    }

    // 2. Construct client
    kmipkit_client_t *client = NULL;
    st = kmipkit_client_new_default(transport, &client);
    kmipkit_transport_config_free(transport);
    if (st != KMIPKIT_OK) {
        fprintf(stderr, "Client init failed (status=%d)\n", st);
        return 1;
    }

    // 3. Execute Discover Versions
    kmipkit_discover_versions_response_t *resp = NULL;
    st = kmipkit_client_discover_versions(client, &resp);
    if (st == KMIPKIT_OK) {
        printf("Discover Versions succeeded!\n");
        kmipkit_discover_versions_response_free(resp);
    }

    kmipkit_client_free(client);
    return 0;
}
```

---

## 5. Quickstart: Java (JNI)

Add the Maven/Gradle dependency (`org.kmipkit:kmipkit-java:1.0.0`) and use `try-with-resources` for deterministic native handle cleanup:

```java
import org.kmipkit.extensions.*;
import org.kmipkit.fluent.*;
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
             ClientConfiguration config = ClientConfiguration.defaults()
                .withWireEncoding(WireEncoding.TTLV);
             Client client = Client.create(config, transport)) {

            // Create and activate an AES-256 key
            var created = client.createSymmetricKey()
                .algorithm(KmipValues.CryptographicAlgorithm.AES)
                .length(256)
                .usageMask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
                .name("java-service-dek")
                .execute();

            String keyId = created.uniqueIdentifier().orElseThrow();
            client.activate(keyId).execute();
            System.out.println("Activated key: " + keyId);
        }
    }
}
```

---

## 6. Quickstart: Python (CFFI)

Install the wheel (`pip install kmipkit`) and use context managers to guarantee zeroized cleanup:

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

    encrypted = client.encrypt(key_id, b"Hello KMIP 2.1!").execute()
    decrypted = client.decrypt(key_id, encrypted.data).execute()
    assert decrypted.data == b"Hello KMIP 2.1!"
    print(f"Verified roundtrip with key {key_id}")
```
