# 01. Schnelleinstieg

Dieser Leitfaden beschreibt den Build von **KMIPKit** mit dem per Prüfsumme gepinnten OpenSSL 3.5.9 sowie den vollständigen Schlüssel-Lebenszyklus (`Create` -> `Activate` -> `Encrypt` -> `Decrypt` -> `Revoke` -> `Destroy`).

---

## 1. Gepinntes OpenSSL bauen & Workspace prüfen

```bash
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1

cargo check --workspace --all-targets --all-features
cargo test --workspace
cargo test --manifest-path kmip-proxy/Cargo.toml
```

---

## 2. Lokalen KMIP-Testserver starten

```bash
cargo run --manifest-path kmip-proxy/Cargo.toml -- \
  --listen 127.0.0.1:5696 \
  --provider mock
```

---

## 3. Vollständiges Rust-Beispiel (mTLS + TTLV)

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
    let transport_config = TransportConfig::builder(Endpoint::raw_tls("kms.internal.example.com", 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file("/etc/kmip/client.crt"),
            PrivateKeyInput::from_pem_file("/etc/kmip/client.key"),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file("/etc/kmip/ca.crt"),
        ]))
        .timeouts(TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::from_secs(15))))
        .build()?;

    let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;
    let client_config = ClientConfiguration::new(registry)
        .with_protocol_version(ProtocolVersion::V2_1)
        .with_wire_encoding(WireEncoding::Ttlv);

    let mut client = Client::new(client_config, transport_config)?;

    let created = client
        .create_symmetric_key()
        .algorithm(CryptographicAlgorithm::Aes)
        .length(256)
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("app-dek-prod")
        .send()?;
    let key_id = created.unique_identifier().expect("Schlüssel-ID");

    client.activate(key_id.clone()).send()?;
    let enc = client.encrypt(key_id.clone(), b"Vertrauliche Daten").send()?;
    let dec = client.decrypt(key_id.clone(), enc.data().unwrap()).send()?;
    assert_eq!(dec.data().unwrap(), b"Vertrauliche Daten");

    client.revoke(key_id.clone(), RevocationReasonCode::CessationOfOperation).send()?;
    client.destroy(key_id).send()?;
    Ok(())
}
```
