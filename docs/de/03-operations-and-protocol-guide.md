# 03. KMIP 2.0 / 2.1 / 3.0 Protokoll- & Operationshandbuch

KMIPKit unterstützt standardmäßig **OASIS KMIP 2.1** sowie explizit **KMIP 2.0** und **KMIP 3.0** (einschließlich NIST Post-Quantum Cryptography).

---

## 1. Protokollversion & Wire-Encoding-Auswahl

```rust
use kmipkit::client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit::client::{RequestOptions, WireEncoding};
use kmipkit::protocol::ProtocolVersion;

let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;

let config = ClientConfiguration::new(registry)
    .with_protocol_version(ProtocolVersion::V3_0)
    .with_wire_encoding(WireEncoding::Ttlv);

// Einzelne Anfrage auf JSON über HTTPS umstellen
let json_opts = RequestOptions::default().with_wire_encoding(WireEncoding::Json);
```

| Wire Encoding | Content-Type (HTTPS) | Raw-TLS-Framing | Empfohlener Einsatz |
| :--- | :--- | :--- | :--- |
| **TTLV** (Standard) | `application/octet-stream` | 8-Byte-Header + 8-Byte-Padding | Native HSMs, minimale Latenz |
| **JSON** | `application/json` | Inkrementeller Klammer-Scanner | Cloud-Gateways, REST-Debugging |
| **XML** | `text/xml` | DTD-freier XML-Scanner | Enterprise-XML/SOAP-Systeme |

---

## 2. Vollständige Operationsmatrix (59 Operationen)

### Client-to-Server (C2S) Operationen

| Kategorie | Operationen | KMIP-Version |
| :--- | :--- | :--- |
| **Erkennung & Gesundheit** | `Discover Versions`, `Query`, `Ping`, `Interop`, `Log` | 2.0 / 2.1 / 3.0 |
| **Erstellung & Import** | `Create`, `Create Key Pair`, `Create Split Key`, `Register`, `Import`, `Derive Key`, `Join Split Key` | 2.0 / 2.1 / 3.0 |
| **Lebenszyklus & Rotation** | `Activate`, `Deactivate` (3.0), `Revoke`, `Destroy`, `Obliterate` (3.0), `Archive`, `Recover`, `Re-key`, `Re-key Key Pair`, `Re-Provision` | 2.0 / 2.1 / 3.0 |
| **Zertifikate** | `Certify`, `Re-certify`, `Validate` | 2.0 / 2.1 / 3.0 |
| **Abruf & Suche** | `Locate`, `Get`, `Export`, `Check`, `Obtain Lease`, `Get Usage Allocation` | 2.0 / 2.1 / 3.0 |
| **Attributverwaltung** | `Get Attributes`, `Get Attribute List`, `Set Attribute`, `Add Attribute`, `Modify Attribute`, `Adjust Attribute`, `Delete Attribute` | 2.0 / 2.1 / 3.0 |
| **Kryptographie** | `Encrypt`, `Decrypt`, `Sign`, `Signature Verify`, `MAC`, `MAC Verify`, `Hash`, `RNG Retrieve`, `RNG Seed` | 2.0 / 2.1 / 3.0 |
| **Post-Quantum KEM (3.0)** | `Encapsulate`, `Decapsulate` (`ML-KEM-512/768/1024`, `X25519MLKEM768`, `SECP256R1MLKEM768`) | **3.0** |
| **Sitzung & Zugriff** | `Login`, `Logout`, `Delegated Login`, `Create User` (3.0), `Create Group` (3.0), `Create Credential` (3.0) | 2.0 / 2.1 / 3.0 |
| **Richtlinien & Admin** | `Get Constraints`, `Set Constraints`, `Set Defaults`, `PKCS#11`, `Set Endpoint Role` | 2.0 / 2.1 / 3.0 |
| **Asynchrone Steuerung** | `Poll`, `Cancel`, `Process`, `Query Asynchronous Requests` | 2.0 / 2.1 / 3.0 |

### Server-to-Client (S2C) Operationen

| Operation | Zweck | Transport-Unterstützung |
| :--- | :--- | :--- |
| **`Discover Versions`** | Server fragt unterstützte Client-Versionen ab | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Query`** | Server fragt Client-Fähigkeiten & Namespaces ab | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Notify`** | Server sendet Attribut- oder Statusänderungs-Meldungen | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Put`** | Server sendet proaktiv neue/ersetzte Schlüssel oder Zertifikate | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Set Endpoint Role`** | Verhandelt Rollenumkehr auf bestehender TLS-Verbindung | Raw TLS Role-Reversed Stream |

---

## 3. Atomare Multi-Operation-Batches & `ID Placeholder`

Führen Sie abhängige Operationen in einem einzigen Roundtrip über `ClientBatch` und `UniqueIdentifier::id_placeholder()` aus:

```rust
use kmipkit::client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit::protocol::{ActivateRequest, CreateRequest, GetRequest, ObjectType, UniqueIdentifier};

let mut batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::create(
    CreateRequest::builder(ObjectType::SymmetricKey).build()?,
)));

// Item 2: Schlüssel aus Item 1 über ID Placeholder aktivieren
batch.push(ClientBatchItem::new(ClientRequest::activate(
    ActivateRequest::new(UniqueIdentifier::id_placeholder()),
)));

// Item 3: Aktivierten Schlüssel im selben atomaren Batch abrufen
batch.push(ClientBatchItem::new(ClientRequest::get(
    GetRequest::new(UniqueIdentifier::id_placeholder()),
)));

let outcome = client.execute(&batch)?;
assert_eq!(outcome.response().items().len(), 3);
```

---

## 4. Asynchrone Operationen (`Poll`, `Cancel`, `Process`)

Antwortet ein KMIP-Server mit `ResultStatus::OperationPending`, stellt KMIPKit explizite Methoden bereit:

```rust
use kmipkit::protocol::{CancelRequest, PollRequest};

let poll_outcome = client.poll(&PollRequest::new(correlation_value.clone()))?;
let cancel_outcome = client.cancel(&CancelRequest::new(correlation_value))?;
```

---

## 5. KMIP 3.0 Post-Quanten-Kryptographie (`ML-KEM` & `ML-DSA`)

Mit `ProtocolVersion::V3_0` unterstützt KMIPKit NIST FIPS 203 (`ML-KEM`), FIPS 204 (`ML-DSA`) und FIPS 205 (`SLH-DSA`):

```rust
let kp = client
    .create_key_pair()
    .algorithm(CryptographicAlgorithm::MlKem768)
    .send()?;

let encap = client.encapsulate(kp.public_key_unique_identifier().unwrap().clone()).send()?;
let decap = client
    .decapsulate(
        kp.private_key_unique_identifier().unwrap().clone(),
        encap.ciphertext().unwrap(),
    )
    .send()?;
```

---

## 6. Vendor-Extensions & Kritikalitätsprüfung

KMIPKit implementiert die OASIS-Regeln für **Message Extensions**:
* **Unbekannte nicht-kritische Extensions (`Criticality Indicator = false`):** unverändert im TTLV-Baum erhalten.
* **Unbekannte kritische Extensions (`Criticality Indicator = true`):** mit Protokollfehler abgelehnt.
* **Registrierte Vendor-Extensions:** über `ExtensionDefinition` in `ClientExtensionRegistry` konsistent über Rust, C, Java und Python validierbar.
