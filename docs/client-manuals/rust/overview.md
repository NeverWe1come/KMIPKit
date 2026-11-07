# KMIPKit Rust Client Manual

`kmipkit` provides a production-grade, memory-safe client implementation for **OASIS KMIP 2.1** with support for raw TTLV over TLS 1.3 and HTTPS/1.1.

---

## 1. Installation and OS Requirements

Add `kmipkit` to your `Cargo.toml`:

```toml
[dependencies]
kmipkit = "1.0.0"
```

### Supported Platforms
- **Linux**: `x86_64` and `aarch64` (glibc 2.28+ / musl)
- **macOS**: `x86_64` (Intel) and `aarch64` (Apple Silicon), macOS 12+
- **Windows**: `x86_64` MSVC, Windows 10 / Server 2019+
- **Toolchain**: Rust Edition 2024, MSRV 1.94+

---

## 2. Quickstart

Create an authenticated client, provision an AES-256 symmetric key, execute authenticated encryption, and securely destroy the key:

```rust
use kmipkit::client::Client;
use kmipkit::config::{ClientConfig, TlsConfig};
use kmipkit::models::{SymmetricAlgorithm, CryptographicUsage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure mutual TLS
    let tls = TlsConfig::builder()
        .ca_certificate_file("/etc/kmip/ca.pem")
        .client_certificate_file("/etc/kmip/client.crt")
        .client_key_file("/etc/kmip/client.key")
        .build()?;

    let config = ClientConfig::builder()
        .endpoint("kms.internal.corp:5696")
        .tls(tls)
        .build()?;

    let client = Client::connect(config)?;

    // 2. Discover supported protocol versions
    let versions = client.discover_versions().execute()?;
    println!("Server versions: {:?}", versions);

    // 3. Create AES-256 symmetric key
    let key_id = client
        .create_symmetric_key()
        .name("database-column-encryption")
        .algorithm(SymmetricAlgorithm::Aes)
        .length_bits(256)
        .usage(CryptographicUsage::ENCRYPT | CryptographicUsage::DECRYPT)
        .execute()?;

    println!("Created key ID: {}", key_id);

    // 4. Activate the key
    client.activate(&key_id).execute()?;

    // 5. Encrypt data on the server
    let plaintext = b"Confidential banking transaction payload";
    let ciphertext = client
        .encrypt(&key_id)
        .plaintext(plaintext)
        .execute()?;

    // 6. Decrypt data on the server
    let decrypted = client
        .decrypt(&key_id)
        .ciphertext(&ciphertext)
        .execute()?;

    assert_eq!(decrypted, plaintext);

    // 7. Destroy key
    client.destroy(&key_id).execute()?;
    println!("Key successfully revoked and destroyed.");

    Ok(())
}
```

---

## 3. Connection and Transport Configuration

KMIPKit supports both traditional **Raw TTLV over TLS 1.3** and **TTLV over HTTPS/1.1**:

### Raw TLS (Port 5696)
```rust
use kmipkit::config::{ClientConfig, TlsConfig, TransportMode};

let config = ClientConfig::builder()
    .endpoint("hsm.cluster.local:5696")
    .transport_mode(TransportMode::RawTls)
    .tls(
        TlsConfig::builder()
            .ca_certificate_file("/etc/pki/ca-chain.pem")
            .client_certificate_file("/etc/pki/tls.crt")
            .client_key_file("/etc/pki/tls.key")
            .server_name("hsm.cluster.local")
            .build()?
    )
    .build()?;
```

### HTTPS / HTTP 1.1 (Port 5697)
```rust
let config = ClientConfig::builder()
    .endpoint("https://vault.corp.internal:5697/kmip")
    .transport_mode(TransportMode::Https)
    .tls(tls_config)
    .build()?;
```

---

## 4. Client Method Catalog (Three API Levels)

`kmipkit` exposes three distinct abstraction tiers:

1. **High-Level Builders**: Idiomatic ergonomic methods with strong compile-time validation.
2. **Typed Protocol**: Direct strongly-typed `RequestMessage` and `ResponseMessage` payloads for advanced parameters.
3. **Generic TTLV**: Raw structural TTLV nodes for vendor extensions and custom attributes.

### High-Level Operations Overview

| Operation Category | Methods |
| :--- | :--- |
| **Discovery** | `discover_versions()`, `query()`, `ping()` |
| **Object Lifecycle** | `create_symmetric_key()`, `create_key_pair()`, `register()`, `activate()`, `revoke()`, `destroy()`, `archive()`, `recover()` |
| **Cryptographic** | `encrypt()`, `decrypt()`, `sign()`, `signature_verify()`, `mac()`, `mac_verify()`, `rng_retrieve()` |
| **Key Transfer & Derivation**| `get()`, `get_key_attributes()`, `derive_key()`, `split_key()`, `join_split_key()` |
| **Attributes** | `get_attributes()`, `get_attribute_list()`, `add_attribute()`, `modify_attribute()`, `delete_attribute()` |
| **Batching & Async** | `batch()`, `poll()`, `cancel()` |

---

## 5. Error Handling and Secret Zeroization

### Error Categories and Delivery States
`kmipkit::Error` provides exact delivery state tracking:

```rust
use kmipkit::error::{Error, DeliveryState};

match client.encrypt(&key_id).plaintext(b"data").execute() {
    Ok(ciphertext) => println!("Success: {} bytes", ciphertext.len()),
    Err(Error::KmipResponse { status, reason, message }) => {
        eprintln!("KMIP Server Failure: {:?} ({:?}): {}", status, reason, message);
    }
    Err(Error::Transport { source, delivery_state }) => match delivery_state {
        DeliveryState::NotSent => eprintln!("Transport failed before dispatch. Safe to retry."),
        DeliveryState::PossiblySent => eprintln!("Network failed in-flight. Do NOT retry non-idempotent calls!"),
        DeliveryState::ResponseStarted => eprintln!("Server processed request; connection interrupted."),
    },
    Err(err) => eprintln!("Client error: {}", err),
}
```

### Secret Zeroization
All cryptographic keys and sensitive responses implement the `zeroize::Zeroize` trait. Sensitive key material is wiped from memory upon `Drop`.

---

## 6. Local Testing with `kmipkit-test-server`

Test your application logic without an external HSM:

```rust
#[cfg(test)]
mod tests {
    use kmipkit_test_server::{TestServer, ServerConfig};
    use kmipkit::client::Client;

    #[test]
    fn test_create_and_encrypt() {
        // Start ephemeral in-memory KMIP 2.1 server with auto-generated PKI
        let server = TestServer::start_ephemeral().expect("server start");
        let client = Client::connect(server.client_config()).expect("client connect");

        let key_id = client.create_symmetric_key()
            .name("unit-test-key")
            .length_bits(256)
            .execute()
            .unwrap();

        assert!(!key_id.is_empty());
    }
}
```
